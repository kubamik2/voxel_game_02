use std::sync::Arc;

use wgpu::util::DeviceExt;

#[derive(Clone)]
pub struct IndexBuffer {
    buffer: Arc<wgpu::Buffer>,
}

impl IndexBuffer {
    pub const FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;

    pub fn new_init(device: &wgpu::Device, indices: &[u32]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("IndexBuffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { buffer: Arc::new(buffer) }
    }

    pub fn new_size(device: &wgpu::Device, size: u64) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("IndexBuffer"),
            mapped_at_creation: false,
            size,
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { buffer: Arc::new(buffer) }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}