use std::sync::Arc;

use cgmath::{Deg, Matrix4, Point3, Vector3};
use wgpu::util::DeviceExt;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);

pub trait Camera {
    fn z_near(&self) -> f32;
    fn z_far(&self) -> f32;
    fn camera_position(&self) -> Point3<f32>;
    fn camera_direction(&self) -> Vector3<f32>;
    fn fovy(&self) -> Deg<f32>;
    fn build_view_projection_matrix(&self, aspect: f32) -> Matrix4<f32> {
        let view = Matrix4::look_to_rh(self.camera_position(), self.camera_direction(), UP);
        let projection = cgmath::perspective(self.fovy(), aspect, self.z_near(), self.z_far());
        
        OPENGL_TO_WGPU_MATRIX * projection * view
    }
}

#[derive(Clone)]
pub struct ViewProjection {
    view_projection: [[f32; 4]; 4],
    buffer: Arc<wgpu::Buffer>,
}

impl ViewProjection {
    pub fn new(device: &wgpu::Device) -> Self {
        use cgmath::SquareMatrix;
        let view_projection: [[f32; 4]; 4] = Matrix4::identity().into();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ViewProjection_buffer"),
            contents: bytemuck::cast_slice(&view_projection),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        Self { view_projection, buffer: Arc::new(buffer) }
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.view_projection));
    }

    pub fn update_from_matrix(&mut self, matrix: Matrix4<f32>) {
        self.view_projection = matrix.into();
    }

    pub fn create_bind_group_layout(device: &wgpu::Device, binding: u32) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ViewProjection_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Uniform
                    },
                    visibility: wgpu::ShaderStages::VERTEX,
                    count: None
                }
            ]
        })
    }
    
    pub fn bind_group_layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            count: None,
            ty: wgpu::BindingType::Buffer {
                has_dynamic_offset: false,
                min_binding_size: None,
                ty: wgpu::BufferBindingType::Uniform,
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