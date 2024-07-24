use std::sync::{Arc, OnceLock};

use cgmath::{Vector2, Vector3};
use chunk_generator::GenerationStage;
use chunk_part::{chunk_part_mesher::MeshingOutput, ChunkPart, CHUNK_SIZE};
use dynamic_chunk_mesh::DynamicChunkMesh;
use wgpu::util::DeviceExt;

use crate::block::{model::FacePacked, Block};

use super::PARTS_PER_CHUNK;

pub mod dynamic_chunk_mesh;
pub mod chunk_map;
pub mod chunk_part;
pub mod chunk_mesh_map;
pub mod chunk_manager;
pub mod chunk_generator;
pub mod area;

#[derive(Clone)]
pub struct Chunk {
    pub position: Vector2<i32>,
    pub parts: [ChunkPart; PARTS_PER_CHUNK],
    pub generation_stage: GenerationStage,
}

impl Chunk {
    pub fn new_air(chunk_position: Vector2<i32>) -> Self {
        Self {
            position: chunk_position,
            parts: std::array::from_fn(|_| ChunkPart::new_air()),
            generation_stage: GenerationStage::Empty,
        }
    }

    pub fn get_block(&self, position: Vector3<usize>) -> Option<&Block> {
        let chunk_part_index = position.y / CHUNK_SIZE;
        if chunk_part_index >= PARTS_PER_CHUNK { return None; }

        let position_in_chunk_part = position.map(|f| f.rem_euclid(CHUNK_SIZE));
        let part = &self.parts[chunk_part_index];
        Some(part.get_block(position_in_chunk_part))
    }

    pub fn set_block(&mut self, position: Vector3<usize>, block: Block) {
        let chunk_part_index = position.y / CHUNK_SIZE;
        if chunk_part_index >= PARTS_PER_CHUNK { return; }

        let position_in_chunk = position.map(|f| f.rem_euclid(CHUNK_SIZE));
        let part = &mut self.parts[chunk_part_index];

        part.set_block(position_in_chunk.into(), block);
    }
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
