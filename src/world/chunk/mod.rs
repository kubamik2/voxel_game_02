use std::{fmt::Debug, ops::{Index, IndexMut}, sync::OnceLock};

use cgmath::{Vector2, Vector3};
use chunk_generator::GenerationStage;
use chunk_part::{chunk_part_position::ChunkPartPosition, ChunkPart, CHUNK_SIZE};
use wgpu::util::DeviceExt;

use crate::{block::{light::LightLevel, Block}, chunk_position::ChunkPosition, BLOCK_LIST};

use super::{CHUNK_HEIGHT, PARTS_PER_CHUNK};

pub mod dynamic_chunk_mesh;
pub mod chunk_map;
pub mod chunk_part;
pub mod chunk_mesh_map;
pub mod chunk_manager;
pub mod chunk_generator;
pub mod chunks3x3;
pub mod chunk_renderer;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    pub position: Vector2<i32>,
    pub parts: [ChunkPart; PARTS_PER_CHUNK],
    pub generation_stage: GenerationStage,
    pub highest_blocks: HighestBlockPositions,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct HighestBlockPosition {
    pub y: u8,
    pub chunk_part_index: u8,
}

impl Default for HighestBlockPosition {
    #[inline]
    fn default() -> Self {
        Self { y: 0, chunk_part_index: 0 }
    }
}

use serde_big_array::BigArray;
#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct HighestBlockPositions(
    #[serde(with = "BigArray")]
    [HighestBlockPosition; CHUNK_SIZE * CHUNK_SIZE]
);

impl Default for HighestBlockPositions {
    #[inline]
    fn default() -> Self {
        Self([HighestBlockPosition::default(); CHUNK_SIZE * CHUNK_SIZE])
    }
}

impl Index<Vector2<u8>> for HighestBlockPositions {
    type Output = HighestBlockPosition;
    #[inline]
    fn index(&self, index: Vector2<u8>) -> &Self::Output {
        &self.0[index.x as usize + index.y as usize * CHUNK_SIZE]
    }
}

impl IndexMut<Vector2<u8>> for HighestBlockPositions {
    #[inline]
    fn index_mut(&mut self, index: Vector2<u8>) -> &mut Self::Output {
        &mut self.0[index.x as usize + index.y as usize * CHUNK_SIZE]
    }
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
            highest_blocks: HighestBlockPositions::default(),
        }
    }

    #[inline]
    fn get_chunk_part_and_chunk_part_position(&self, position: Vector3<usize>) -> Option<(&ChunkPart, Vector3<usize>)> {
        if position.y >= CHUNK_HEIGHT { return None; }
        let chunk_part_index = position.y.div_euclid(CHUNK_SIZE);
        let chunk_part_position = position.map(|f| f.rem_euclid(CHUNK_SIZE));

        Some((&self.parts[chunk_part_index], chunk_part_position))
    }

    #[inline]
    fn get_chunk_part_mut_and_chunk_part_position(&mut self, position: Vector3<usize>) -> Option<(&mut ChunkPart, Vector3<usize>)> {
        if position.y >= CHUNK_HEIGHT { return None; }
        let chunk_part_index = position.y.div_euclid(CHUNK_SIZE);
        let chunk_part_position = position.map(|f| f.rem_euclid(CHUNK_SIZE));

        Some((&mut self.parts[chunk_part_index], chunk_part_position))
    }

    #[inline]
    fn get_chunk_part_index_and_chunk_part_position(position: Vector3<usize>) -> Option<(usize, Vector3<usize>)> {
        if position.y >= CHUNK_HEIGHT { return None; }
        let chunk_part_index = position.y.div_euclid(CHUNK_SIZE);
        let chunk_part_position = position.map(|f| f.rem_euclid(CHUNK_SIZE));

        Some((chunk_part_index, chunk_part_position))
    }

    #[inline]
    pub fn get_block(&self, position: ChunkPosition) -> &Block {
        let chunk_part = &self.parts[position.chunk_part_index()];
        chunk_part.get_block(position.chunk_part_position())
    }

    #[inline]
    pub fn set_block(&mut self, position: ChunkPosition, block: Block) {
        let highest_block_position = &mut self.highest_blocks[position.chunk_part_position().xz()];
        if block.is_air() && position.chunk_part_index() as u8 == highest_block_position.chunk_part_index && position.chunk_part_position().y == highest_block_position.y {
            Self::find_new_highest_block(&self.parts, position, highest_block_position);
        } else if (position.chunk_part_index() as u8 == highest_block_position.chunk_part_index && position.chunk_part_position().y > highest_block_position.y)
        || (position.chunk_part_index() as u8 > highest_block_position.chunk_part_index) {
            *highest_block_position = HighestBlockPosition { chunk_part_index: position.chunk_part_index() as u8, y: position.chunk_part_position().y };
        }
        let chunk_part = &mut self.parts[position.chunk_part_index()];
        chunk_part.set_block(position.chunk_part_position(), block);
    }

    #[inline]
    fn find_new_highest_block(parts: &[ChunkPart; PARTS_PER_CHUNK], position: ChunkPosition, prev_highest_block_position: &mut HighestBlockPosition) {
        let chunk_part = &parts[position.chunk_part_index()];
        for y in (0..prev_highest_block_position.y).rev() {
            let chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(Vector3::new(position.chunk_part_position().x as u32, y as u32, position.chunk_part_position().z as u32)) };
            if !chunk_part.get_block(chunk_part_position).is_air() {
                *prev_highest_block_position = HighestBlockPosition { chunk_part_index: prev_highest_block_position.chunk_part_index, y };
                return;
            }
        }

        for chunk_part_index in 0..prev_highest_block_position.chunk_part_index {
            let chunk_part = &parts[chunk_part_index as usize];
            for y in (0..CHUNK_SIZE as u8).rev() {
                let chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(Vector3::new(position.chunk_part_position().x as u32, y as u32, position.chunk_part_position().z as u32)) };
                if !chunk_part.get_block(chunk_part_position).is_air() {
                    *prev_highest_block_position = HighestBlockPosition { chunk_part_index, y };
                    return;
                }
            }
        }

        *prev_highest_block_position = HighestBlockPosition { chunk_part_index: 0, y: 0 };
    }

    #[inline]
    pub fn get_light_level(&self, position: ChunkPosition) -> LightLevel {
        let chunk_part = &self.parts[position.chunk_part_index()];
        chunk_part.get_light_level(position.chunk_part_position())
    }

    #[inline]
    pub fn set_light_level(&mut self, position: ChunkPosition, light_level: LightLevel) {
        let chunk_part = &mut self.parts[position.chunk_part_index()];
        chunk_part.set_light_level(position.chunk_part_position(), light_level);
    }

    #[inline]
    pub fn get_block_light_level(&self, position: ChunkPosition) -> u8 {
        let chunk_part = &self.parts[position.chunk_part_index()];
        chunk_part.get_block_light_level(position.chunk_part_position())
    }

    #[inline]
    pub fn set_block_light_level(&mut self, position: ChunkPosition, level: u8) {
        let chunk_part = &mut self.parts[position.chunk_part_index()];
        chunk_part.set_block_light_level(position.chunk_part_position(), level);
    }

    #[inline]
    pub fn get_sky_light_level(&self, position: ChunkPosition) -> u8 {
        let chunk_part = &self.parts[position.chunk_part_index()];
        chunk_part.get_sky_light_level(position.chunk_part_position())
    }

    #[inline]
    pub fn set_sky_light_level(&mut self, position: ChunkPosition, level: u8) {
        let chunk_part = &mut self.parts[position.chunk_part_index()];
        chunk_part.set_sky_light_level(position.chunk_part_position(), level);
    }

    #[inline]
    pub fn compress_parts(&mut self) {
        for part in self.parts.iter_mut() {
            part.compress();
        }
    }

    #[inline]
    pub fn clean_up_parts(&mut self) {
        for part in self.parts.iter_mut() {
            part.block_pallet.clean_up();
        }
    }

    #[inline]
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
