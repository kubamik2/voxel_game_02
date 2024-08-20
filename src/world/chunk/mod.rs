use std::{fmt::Debug, sync::OnceLock};

use cgmath::{Vector2, Vector3};
use chunk_generator::GenerationStage;
use chunk_part::{ChunkPart, CHUNK_SIZE};
use wgpu::util::DeviceExt;

use crate::block::Block;

use super::PARTS_PER_CHUNK;

pub mod dynamic_chunk_mesh;
pub mod chunk_map;
pub mod chunk_part;
pub mod chunk_mesh_map;
pub mod chunk_manager;
pub mod chunk_generator;
pub mod area;
pub mod chunk_renderer;

#[derive(Clone)]
pub struct Chunk {
    pub position: Vector2<i32>,
    pub parts: [ChunkPart; PARTS_PER_CHUNK],
    pub generation_stage: GenerationStage,
    pub highest_blocks: [(u8, u8); CHUNK_SIZE * CHUNK_SIZE],
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
        .field("position", &self.position)
        .finish()
    }
}

impl Chunk {
    pub fn new_air(chunk_position: Vector2<i32>) -> Self {
        Self {
            position: chunk_position,
            parts: std::array::from_fn(|_| ChunkPart::new_air()),
            generation_stage: GenerationStage::Empty,
            highest_blocks: std::array::from_fn(|_| (0, 0)),
        }
    }

    pub fn get_block_chunk_local(&self, position: Vector3<usize>) -> Option<&Block> {
        let chunk_part_index = position.y.div_euclid(CHUNK_SIZE);
        if chunk_part_index >= PARTS_PER_CHUNK { return None; }

        let position_in_chunk_part = position.map(|f| f.rem_euclid(CHUNK_SIZE));
        let part = &self.parts[chunk_part_index];
        part.get_block(position_in_chunk_part)
    }

    pub fn set_block_chunk_local(&mut self, position: Vector3<usize>, block: Block) {
        let chunk_part_index = position.y.div_euclid(CHUNK_SIZE);
        if chunk_part_index >= PARTS_PER_CHUNK { return; }

        let position_in_chunk = position.map(|f| f.rem_euclid(CHUNK_SIZE));
        let part = &mut self.parts[chunk_part_index];

        part.set_block(position_in_chunk.into(), block);
    }

    pub fn compress_parts(&mut self) {
        for part in self.parts.iter_mut() {
            part.block_layers.compress();
            part.light_level_layers.compress();
        }
    }

    pub fn clean_up_parts(&mut self) {
        for part in self.parts.iter_mut() {
            part.block_pallet.clean_up();
        }
    }

    pub fn maintain_parts(&mut self) {
        self.compress_parts();
        self.clean_up_parts();
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
