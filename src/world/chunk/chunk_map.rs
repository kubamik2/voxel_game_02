use std::sync::Arc;

use cgmath::Vector2;
use hashbrown::{hash_map::{Keys, Values, ValuesMut}, HashMap};

use crate::{block::{light::LightLevel, Block}, chunk_position::ChunkPosition, global_vector::GlobalVecU};

use super::{chunk_generator::GenerationStage, Chunk};
use parking_lot::RwLock;

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ChunkMap(HashMap<Vector2<i32>, Arc<Chunk>>);

impl ChunkMap {
    #[inline]
    pub fn insert(&mut self, chunk: Chunk) -> Option<Arc<Chunk>> {
        self.insert_arc(Arc::new(chunk))
    }

    #[inline]
    pub fn insert_arc(&mut self, chunk_arc: Arc<Chunk>) -> Option<Arc<Chunk>> {
        self.0.insert(chunk_arc.position, chunk_arc)
    }

    #[inline]
    pub fn update_chunk(&mut self, chunk_arc: Arc<Chunk>) {
        let Some(old_chunk) = self.borrow_chunk(&chunk_arc.position) else { return; };
        if old_chunk.last_update < chunk_arc.last_update {
            self.insert_arc(chunk_arc);
        }
    }

    #[inline]
    pub fn get_chunk(&self, position: &Vector2<i32>) -> Option<Arc<Chunk>> {
        self.0.get(position).cloned()
    }
    
    #[inline]
    pub fn borrow_chunk(&self, position: &Vector2<i32>) -> Option<&Chunk> {
        self.0.get(position).map(|f| f.as_ref())
    }

    #[inline]
    pub fn borrow_mut_chunk(&mut self, position: &Vector2<i32>) -> Option<&mut Chunk> {
        self.0.get_mut(position).map(Arc::make_mut)
    }

    #[inline]
    pub fn contains_position(&self, position: &Vector2<i32>) -> bool {
        self.0.contains_key(position)
    }

    #[inline]
    pub fn iter_chunks(&self) -> Values<Vector2<i32>, Arc<Chunk>> {
        self.0.values()
    }

    #[inline]
    pub fn iter_mut_chunks(&mut self) -> ValuesMut<Vector2<i32>, Arc<Chunk>> {
        self.0.values_mut()
    }

    #[inline]
    pub fn positions(&self) -> Keys<Vector2<i32>, Arc<Chunk>> {
        self.0.keys()
    }

    #[inline]
    pub fn remove(&mut self, position: &Vector2<i32>) -> Option<Arc<Chunk>> {
        self.0.remove(position)
    }

    #[inline]
    pub fn get_block(&self, position: GlobalVecU) -> Option<&Block> {
        let chunk = self.borrow_chunk(&position.chunk.xz())?;
        let position = ChunkPosition::try_from(position).ok()?;
        Some(chunk.get_block(position))
    }

    #[inline]
    pub fn set_block(&mut self, position: GlobalVecU, block: Block) {
        let Some(chunk) = self.borrow_mut_chunk(&position.chunk.xz()) else { return; };
        let Ok(position) = ChunkPosition::try_from(position) else { return; };
        chunk.set_block(position, block);
    }

    #[inline]
    pub fn get_light_level(&self, position: GlobalVecU) -> Option<LightLevel> {
        let chunk = self.borrow_chunk(&position.chunk.xz())?;
        let position = ChunkPosition::try_from(position).ok()?;
        Some(chunk.get_light_level(position))
    }

    #[inline]
    pub fn set_light_level(&mut self, position: GlobalVecU, light_level: LightLevel) {
        let Some(chunk) = self.borrow_mut_chunk(&position.chunk.xz()) else { return; };
        let Ok(position) = ChunkPosition::try_from(position) else { return; };
        chunk.set_light_level(position, light_level);
    }

    #[inline]
    pub fn is_chunk_surrounded_by_chunks_at_least_at_stage(&self, position: Vector2<i32>, stage: GenerationStage) -> bool {
        let position: Vector2<i32> = position;
        for z in -1..=1 {
            for x in -1..=1 {
                if x == 0 && z == 0 { continue; }
                let Some(chunk) = self.borrow_chunk(&(position + Vector2::new(x, z))) else { return false; };
                if (chunk.generation_stage as u8) < (stage as u8) { return false; }
            }
        }

        true
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ChunkMapLock(Arc<RwLock<ChunkMap>>);

impl ChunkMapLock {
    #[inline]
    pub fn read(&self) -> parking_lot::lock_api::RwLockReadGuard<parking_lot::RawRwLock, ChunkMap> {
        self.0.read()
    }

    #[inline]
    pub fn write(&self) -> parking_lot::lock_api::RwLockWriteGuard<parking_lot::RawRwLock, ChunkMap> {
        self.0.write()
    }
}
