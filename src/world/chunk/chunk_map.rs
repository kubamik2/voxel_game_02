use std::{collections::HashMap, sync::{Arc, Mutex}};

use cgmath::Vector2;

use super::{chunk_generator::GenerationStage, chunk_part::chunk_part_mesher::MeshingOutput, Chunk};

pub struct ChunkMap {
    chunks: HashMap<(i32, i32), Arc<Mutex<Chunk>>>
}

impl ChunkMap {
    pub fn new() -> Self {
        Self { chunks: HashMap::new() }
    }

    pub fn insert<T: Into<(i32, i32)>>(&mut self, position: T, chunk: Chunk) -> Option<Arc<Mutex<Chunk>>> {
        self.chunks.insert(position.into(), Arc::new(Mutex::new(chunk)))
    }

    pub fn get<T: Into<(i32, i32)>>(&self, position: T) -> Option<&Arc<Mutex<Chunk>>> {
        self.chunks.get(&position.into())
    }

    pub fn get_mut<T: Into<(i32, i32)>>(&mut self, position: T) -> Option<&mut Arc<Mutex<Chunk>>> {
        self.chunks.get_mut(&position.into())
    }

    pub fn contains_key<T: Into<(i32, i32)>>(&self, position: T) -> bool {
        self.chunks.contains_key(&position.into())
    }

    pub fn values(&self) -> std::collections::hash_map::Values<(i32, i32), Arc<Mutex<Chunk>>> {
        self.chunks.values()
    }

    pub fn is_chunk_surrounded_by_chunks_at_least_at_stage<T: Into<(i32, i32)>>(&self, position: T, stage: GenerationStage) -> bool {
        let position: (i32, i32) = position.into();
        for z in -1..=1 {
            for x in -1..=1 {
                if x == 0 && z == 0 { continue; }
                let Some(chunk_lock) = self.chunks.get(&(position.0 + x, position.1 + z)) else { return false; };
                let chunk = chunk_lock.lock().unwrap();
                if (chunk.generation_stage as u8) < (stage as u8) { return false; }
            }
        }

        true
    }
}