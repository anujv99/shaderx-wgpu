use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct UniformBuffer {
  pub buffer: wgpu::Buffer,
  pub bind_group: wgpu::BindGroup,
  pub bind_group_layout: wgpu::BindGroupLayout,
  pub binding: u32,
}

#[derive(Debug)]
pub struct UniformBufferCreateDesc<'a, T: NoUninit> {
  pub device: &'a wgpu::Device,
  pub data: &'a T,
  pub binding: u32,
}

impl UniformBuffer {
  pub fn new<T: NoUninit>(create_desc: &UniformBufferCreateDesc<T>) -> Self {
    let device = create_desc.device;
    let data = create_desc.data;

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("uniform buffer"),
      contents: bytemuck::cast_slice(&[*data]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: create_desc.binding,
          visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ],
      label: Some("uniform buffer bind group layout"),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("uniform buffer bind group"),
      entries: &[
        wgpu::BindGroupEntry {
          binding: create_desc.binding,
          resource: buffer.as_entire_binding(),
        },
      ],
      layout: &bind_group_layout,
    });

    Self {
      buffer,
      bind_group,
      bind_group_layout,
      binding: create_desc.binding,
    }
  }

  pub fn update<T: NoUninit>(&self, queue: &wgpu::Queue, data: &T) {
    queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[*data]));
  }
}

