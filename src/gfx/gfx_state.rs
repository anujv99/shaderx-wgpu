use std::sync::Arc;
use wgpu::util::RenderEncoder;
use winit::window::Window;

use super::{pipeline::{Pipeline, PipelineCreateDesc}, uniform_buffer::{UniformBuffer, UniformBufferCreateDesc}};
use bytemuck::{Pod, Zeroable};
use web_time::{SystemTime, UNIX_EPOCH, Duration};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CommonUniformBuffer {
  pub time: f32,
  pub delta_time: f32,
  // padding
  pub padding: [f32; 2],
}

#[derive(Debug)]
pub struct GfxState {
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  size: winit::dpi::PhysicalSize<u32>,
  surface: wgpu::Surface<'static>,
  window: Arc<Window>,
  limits: wgpu::Limits,

  surface_configured: bool,
  pipeline: Option<Pipeline>,

  last_frame_time: Duration,
  common_buffer_data: CommonUniformBuffer,
  common_buffer: UniformBuffer,
}

impl GfxState {
  pub async fn new(window: Arc<Window>) -> Self {
    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      #[cfg(not(target_arch = "wasm32"))]
        backends: wgpu::Backends::PRIMARY,
      #[cfg(target_arch = "wasm32")]
        backends: wgpu::Backends::GL,
      ..Default::default()
    });

    let surface = instance.create_surface(window.clone()).expect("[gfx] failed to create surface");

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::default(),
      compatible_surface: Some(&surface),
      force_fallback_adapter: false,
    }).await.expect("[gfx] failed to create adapter");

    let limits = {
      #[cfg(not(target_arch = "wasm32"))] {
        wgpu::Limits::default()
      }
      #[cfg(target_arch = "wasm32")] {
        wgpu::Limits::downlevel_webgl2_defaults()
      }
    };

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
      label: None,
      required_features: wgpu::Features::empty(),
      required_limits: limits.clone(),
      memory_hints: Default::default(),
    }, None).await.expect("[gfx] failed to create device");

    device.on_uncaptured_error(Box::new(move |err| {
      log::error!("[gfx] uncaptured error: {:?}", err);
    }));

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats
      .iter()
      .copied()
      .find(|f| f.is_srgb())
      .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: surface_caps.present_modes[0],
      alpha_mode: surface_caps.alpha_modes[0],
      desired_maximum_frame_latency: 2,
      view_formats: vec![],
    };

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let common_buffer_data = CommonUniformBuffer {
      time: 0.0,
      delta_time: 0.0,
      padding: [0.0; 2],
    };

    let common_buffer = UniformBuffer::new(&UniformBufferCreateDesc {
      device: &device,
      binding: 0,
      data: &common_buffer_data,
    });

    Self {
      device,
      queue,
      config,
      size,
      surface,
      window,
      limits,
      surface_configured: false,
      pipeline: None,
      common_buffer,
      common_buffer_data,
      last_frame_time: current_time,
    }
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    if !self.surface_configured {
      return Ok(());
    }

    // update common buffer
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let delta_time = (current_time - self.last_frame_time).as_secs_f32();
    self.last_frame_time = current_time;
    self.common_buffer_data.time += delta_time;
    self.common_buffer_data.delta_time = delta_time;
    self.common_buffer.update(&self.queue, &self.common_buffer_data);

    // setup render target
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("render encoder"),
    });

    // render pass
    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::RED),
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
      });

      render_pass.set_bind_group(self.common_buffer.binding, &self.common_buffer.bind_group, &[]);

      if self.pipeline.is_some() {
        let pipeline = self.pipeline.as_ref().unwrap();
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.draw(0..3, 0..1);
      }
    }

    // submit
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = std::cmp::min(new_size.width, self.limits.max_texture_dimension_2d);
      self.config.height = std::cmp::min(new_size.height, self.limits.max_texture_dimension_2d);
      self.surface.configure(&self.device, &self.config);
      self.surface_configured = true;
    }
  }

  pub fn update_shader(&mut self, shader_source: &str) {
    self.pipeline = Some(Pipeline::new(&PipelineCreateDesc {
      device: &self.device,
      config: &self.config,
      shader_source,
      bind_group_layouts: &vec![&self.common_buffer.bind_group_layout],
    }));
  }

  pub async fn compile_shader(&self, shader_source: &str) -> wgpu::CompilationInfo {
    let shader_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("temp shader"),
      source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let result = shader_module.get_compilation_info().await;
    log::info!("{:?}", result);

    result
  }
}
