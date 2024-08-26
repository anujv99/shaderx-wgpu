use std::sync::Arc;
use winit::window::Window;

use super::pipeline::{Pipeline, PipelineCreateDesc};


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
    }
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    if !self.surface_configured {
      return Ok(());
    }

    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("render encoder"),
    });

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

      if self.pipeline.is_some() {
        let pipeline = self.pipeline.as_ref().unwrap();
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.draw(0..3, 0..1);
      }
    }

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
