use std::collections::HashMap;

use cgmath::Vector2;

use super::{chunk_part::chunk_part_mesher::MeshingOutput, Chunk};

pub struct ChunkMap {
    chunks: HashMap<(i32, i32), Chunk>
}

impl ChunkMap {
    pub fn new() -> Self {
        Self { chunks: HashMap::new() }
    }

    pub fn insert<T: Into<(i32, i32)>>(&mut self, position: T, chunk: Chunk) -> Option<Chunk> {
        self.chunks.insert(position.into(), chunk)
    }

    pub fn get<T: Into<(i32, i32)>>(&self, position: T) -> Option<&Chunk> {
        self.chunks.get(&position.into())
    }

    pub fn get_mut<T: Into<(i32, i32)>>(&mut self, position: T) -> Option<&mut Chunk> {
        self.chunks.get_mut(&position.into())
    }

    pub fn contains_key<T: Into<(i32, i32)>>(&self, position: T) -> bool {
        self.chunks.contains_key(&position.into())
    }
}