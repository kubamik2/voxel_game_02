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

        Self { buffer }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}