use std::sync::{Arc, OnceLock};

use cgmath::Vector2;
use chunk_generator::GenerationStage;
use chunk_part::{chunk_part_mesher::MeshingOutput, ChunkPart};
use dynamic_chunk_mesh::DynamicChunkMesh;
use wgpu::util::DeviceExt;

use crate::block::model::FacePacked;

use super::PARTS_PER_CHUNK;

pub mod dynamic_chunk_mesh;
pub mod chunk_map;
pub mod chunk_part;
pub mod chunk_mesh_map;
pub mod chunk_manager;
pub mod chunk_generator;

pub struct Chunk {
    pub position: Vector2<i32>,
    pub parts: [ChunkPart; PARTS_PER_CHUNK],
    pub generation_stage: GenerationStage,
    pub generation_scheduled: bool,
}

impl Chunk {
    pub fn new_air(chunk_position: Vector2<i32>) -> Self {
        Self {
            position: chunk_position,
            parts: std::array::from_fn(|_| ChunkPart::new_air()),
            generation_stage: GenerationStage::Empty,
            generation_scheduled: false,
        }
    }
    // pub fn insert_meshed_chunk_part(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, meshing_data: MeshingOutput, mesh_queue: &mut std::collections::VecDeque<(Vector2<i32>, usize)>) {
    //     let faces_size = meshing_data.faces.len() as u32;
    //     let chunk_part_index = meshing_data.chunk_part_index;
    //     let chunk_position = meshing_data.chunk_position;

    //     let mut needs_resizing = false;
    //     while self.mesh.face_bucket_elements[chunk_part_index] < faces_size {
    //         self.mesh.face_bucket_elements[chunk_part_index] *= 2;
    //         needs_resizing = true;
    //     }

    //     if needs_resizing {
    //         self.mesh.resize(device);
    //         for i in 0..PARTS_PER_CHUNK {
    //             if i == chunk_part_index { continue; }
    //             let chunk_part = &mut self.parts[i];
    //             if chunk_part.meshing_scheduled { continue; }
    //             chunk_part.meshed = false;
    //             chunk_part.meshing_scheduled = true;
    //             mesh_queue.push_front((chunk_position, i));
    //         }
    //     }

    //     let indirect_buffer_offset = (chunk_part_index * std::mem::size_of::<wgpu::util::DrawIndexedIndirectArgs>()) as u64;
    //     let face_buffer_offset = ((0..chunk_part_index).map(|i| self.mesh.face_bucket_elements[i]).sum::<u32>() * std::mem::size_of::<FacePacked>() as u32) as u64;
    //     if meshing_data.faces_num > 0 {
    //         queue.write_buffer(self.mesh.face_buffer(), face_buffer_offset, bytemuck::cast_slice(&meshing_data.faces));
    //     }
    //     queue.write_buffer(self.mesh.indirect_buffer(), indirect_buffer_offset, self.mesh.create_indirect_args(meshing_data.faces_num, chunk_part_index).as_bytes());

    //     let part = &mut self.parts[chunk_part_index];
    //     part.meshing_scheduled = false;
    //     part.meshed = true;
    // }
}
static CHUNK_TRANSLATION_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

pub struct ChunkTranslation {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl ChunkTranslation {
    pub fn new(device: &wgpu::Device, position: Vector2<i32>) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ChunkTranslation_buffer"),
            contents: bytemuck::cast_slice(&[position.x, position.y]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        
        let bind_group_layout = CHUNK_TRANSLATION_BIND_GROUP_LAYOUT.get_or_init(|| Self::create_bind_group_layout(device));

        let bind_group = Self::create_bind_group(device, &buffer, bind_group_layout);

        Self {
            bind_group,
            buffer,
        }
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ChunkTranslation"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: wgpu::BufferBindingType::Uniform,
                },
                visibility: wgpu::ShaderStages::VERTEX,
            }]
        })
    }

    fn create_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer, bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ChunkTranslation_bind_group"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding())
            }],
            layout: bind_group_layout 
        })
    }

    // grabs the layout from global variable, initializing it if needed
    pub fn get_or_init_bind_group_layout(device: &wgpu::Device) -> &wgpu::BindGroupLayout {
        CHUNK_TRANSLATION_BIND_GROUP_LAYOUT.get_or_init(|| {
            Self::create_bind_group_layout(device)
        })
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

}

impl Drop for ChunkTranslation {
    fn drop(&mut self) {
        self.buffer.destroy();
    }
}
