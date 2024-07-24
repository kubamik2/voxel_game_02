use std::{sync::{Arc, Mutex}};

use cgmath::Vector2;
use hashbrown::{hash_map::{Keys, Values}, HashMap};

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
}