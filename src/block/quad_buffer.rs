use wgpu::util::DeviceExt;

use super::model::QuadRaw;

pub struct QuadBuffer {
    buffer: wgpu::Buffer,
}

impl QuadBuffer {
    pub fn new(device: &wgpu::Device, quads: &[QuadRaw]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("QuadBuffer"),
            contents: bytemuck::cast_slice(quads),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer}
    }

    pub fn bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            count: None,
            ty: wgpu::BindingType::Buffer {
                has_dynamic_offset: false,
                min_binding_size: None,
                ty: wgpu::BufferBindingType::Storage { read_only: true }
            },
            visibility: wgpu::ShaderStages::VERTEX,
        }
    }

    pub fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}