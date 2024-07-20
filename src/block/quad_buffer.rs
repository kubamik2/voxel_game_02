use wgpu::util::DeviceExt;

use super::model::QuadRaw;

static QUAD_BUFFER_BIND_GROUP_LAYOUT: std::sync::OnceLock<wgpu::BindGroupLayout> = std::sync::OnceLock::new();

pub struct QuadBuffer {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl QuadBuffer {
    pub fn new(device: &wgpu::Device, quads: &[QuadRaw]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("QuadBuffer"),
            contents: bytemuck::cast_slice(quads),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
        });
        let bind_group_layout = Self::get_or_init_bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("QuadBuffer_bind_group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding())
                }
            ],
            layout: bind_group_layout
        });

        Self { buffer, bind_group }
    }

    pub fn get_or_init_bind_group_layout(device: &wgpu::Device) -> &wgpu::BindGroupLayout {
        QUAD_BUFFER_BIND_GROUP_LAYOUT.get_or_init(|| 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("QuadBuffer_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        count: Some(std::num::NonZeroU32::new(1).unwrap()),
                        ty: wgpu::BindingType::Buffer {
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            ty: wgpu::BufferBindingType::Storage { read_only: true }
                        },
                        visibility: wgpu::ShaderStages::VERTEX
                    }
                ]
            })
        )
        
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}