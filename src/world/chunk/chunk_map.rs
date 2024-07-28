use std::{sync::{Arc, Mutex}};

use cgmath::Vector2;
use hashbrown::{hash_map::{Keys, Values}, HashMap};

use crate::{block::{light::LightLevel, Block}, relative_vector::RelVec3, world::PARTS_PER_CHUNK};

use super::{chunk_generator::GenerationStage, chunk_part::chunk_part_mesher::MeshingOutput, Chunk};

pub struct ChunkMap {
    chunks: HashMap<Vector2<i32>, Arc<Chunk>>
}

impl ChunkMap {
    #[inline]
    pub fn new() -> Self {
        Self { chunks: HashMap::new() }
    }

    #[inline]
    pub fn insert(&mut self, position: Vector2<i32>, chunk: Chunk) -> Option<Arc<Chunk>> {
        self.chunks.insert(position, Arc::new(chunk))
    }

    #[inline]
    pub fn insert_arc(&mut self, position: Vector2<i32>, chunk_arc: Arc<Chunk>) -> Option<Arc<Chunk>> {
        self.chunks.insert(position, chunk_arc)
    }

    #[inline]
    pub fn get(&self, position: Vector2<i32>) -> Option<&Arc<Chunk>> {
        self.chunks.get(&position)
    }

    #[inline]
    pub fn get_mut(&mut self, position: Vector2<i32>) -> Option<&mut Arc<Chunk>> {
        self.chunks.get_mut(&position)
    }

    #[inline]
    pub fn contains_key(&self, position: Vector2<i32>) -> bool {
        self.chunks.contains_key(&position)
    }

    #[inline]
    pub fn values(&self) -> Values<Vector2<i32>, Arc<Chunk>> {
        self.chunks.values()
    }

    #[inline]
    pub fn is_chunk_surrounded_by_chunks_at_least_at_stage(&self, position: Vector2<i32>, stage: GenerationStage) -> bool {
        let position: Vector2<i32> = position;
        for z in -1..=1 {
            for x in -1..=1 {
                if x == 0 && z == 0 { continue; }
                let Some(chunk) = self.chunks.get(&(position + Vector2::new(x, z))) else { return false; };
                if (chunk.generation_stage as u8) < (stage as u8) { return false; }
            }
        }

        true
    }

    #[inline]
    pub fn positions(&self) -> Keys<Vector2<i32>, Arc<Chunk>> {
        self.chunks.keys()
    }

    #[inline]
    pub fn remove(&mut self, position: Vector2<i32>) -> Option<Arc<Chunk>> {
        self.chunks.remove(&position)
    }

    #[inline]
    pub fn get_light_level(&self, position: RelVec3) -> Option<&LightLevel> {
        if position.chunk_pos.y.is_negative() || position.chunk_pos.y >= PARTS_PER_CHUNK as i32 { return None; }
        let Some(chunk) = self.get(position.chunk_pos.xz()) else { return None; };
        let local_position = position.local_pos().map(|f| f.floor() as usize);
        Some(chunk.parts[position.chunk_pos.y as usize].light_level_layers.get_light_level(local_position))
    }

    #[inline]
    pub fn set_light_level(&mut self, position: RelVec3, light_level: LightLevel) {
        if position.chunk_pos.y.is_negative() || position.chunk_pos.y >= PARTS_PER_CHUNK as i32 { return; }
        let Some(chunk) = self.get_mut(position.chunk_pos.xz()) else { return; };
        assert!(Arc::strong_count(chunk) == 1);
        let chunk = Arc::make_mut(chunk);
        let local_position = position.local_pos().map(|f| f.floor() as usize);
        chunk.parts[position.chunk_pos.y as usize].light_level_layers.set_light_level(local_position, light_level);
    }

    #[inline]
    pub fn get_block(&self, position: RelVec3) -> Option<&Block> {
        if position.chunk_pos.y.is_negative() || position.chunk_pos.y >= PARTS_PER_CHUNK as i32 { return None; }
        let Some(chunk) = self.get(position.chunk_pos.xz()) else { return None; };
        let local_position = position.local_pos().map(|f| f.floor() as usize);
        chunk.parts[position.chunk_pos.y as usize].get_block(local_position)
    }

    #[inline]
    pub fn set_block(&mut self, position: RelVec3, block: Block) {
        if position.chunk_pos.y.is_negative() || position.chunk_pos.y >= PARTS_PER_CHUNK as i32 { return; }
        let Some(chunk) = self.get_mut(position.chunk_pos.xz()) else { return; };
        assert!(Arc::strong_count(chunk) == 1);
        let chunk = Arc::make_mut(chunk);
        let local_position = position.local_pos().map(|f| f.floor() as usize);
        chunk.parts[position.chunk_pos.y as usize].set_block(local_position, block);
    }

    #[inline]
    pub fn get_chunk(&self, position: RelVec3) -> Option<&Chunk> {
        if position.chunk_pos.y.is_negative() || position.chunk_pos.y >= PARTS_PER_CHUNK as i32 { return None; }
        let Some(chunk) = self.get(position.chunk_pos.xz()) else { return None; };
        Some(chunk)
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, position: RelVec3) -> Option<&mut Chunk> {
        if position.chunk_pos.y.is_negative() || position.chunk_pos.y >= PARTS_PER_CHUNK as i32 { return None; }
        let Some(chunk) = self.get_mut(position.chunk_pos.xz()) else { return None; };
        assert!(Arc::strong_count(chunk) == 1);
        Some(Arc::make_mut(chunk))
    }
}