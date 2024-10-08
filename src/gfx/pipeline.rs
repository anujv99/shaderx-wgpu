
#[derive(Debug)]
pub struct Pipeline {
  pub pipeline: wgpu::RenderPipeline,
}

#[derive(Debug)]
pub struct PipelineCreateDesc<'a> {
  pub device: &'a wgpu::Device,
  pub config: &'a wgpu::SurfaceConfiguration,
  pub shader_source: &'a str,
  pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
}

impl Pipeline {
  pub fn new(create_desc: &PipelineCreateDesc) -> Self {
    let device = create_desc.device;
    let config = create_desc.config;
    let shader_source = create_desc.shader_source;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Shader"),
      source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Pipeline Layout"),
      bind_group_layouts: create_desc.bind_group_layouts,
      push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: config.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
      cache: None,
    });

    Self { pipeline }
  }
}
