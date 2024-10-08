use std::sync::Arc;

use cgmath::{Vector2, Vector3};
use hashbrown::{hash_map::{Keys, Values, ValuesMut}, HashMap};

use crate::{block::{light::LightLevel, Block}, chunk_position::ChunkPosition, global_vector::GlobalVecU, world::PARTS_PER_CHUNK};

use super::{chunk_generator::GenerationStage, chunk_part::ChunkPart, Chunk};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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
    pub fn get_chunk(&self, position: Vector2<i32>) -> Option<&Chunk> {
        self.chunks.get(&position).map(|f| f.as_ref())
    }

    #[inline]
    pub fn get_mut_chunk(&mut self, position: Vector2<i32>) -> Option<&mut Chunk> {
        let chunk = self.chunks.get_mut(&position)?;
        assert!(Arc::strong_count(chunk) == 1);
        Some(Arc::make_mut(chunk))
    }

    #[inline]
    pub fn get_chunk_part(&self, chunk_part_position: Vector3<i32>) -> Option<&ChunkPart> {
        if chunk_part_position.y.is_negative() || chunk_part_position.y >= PARTS_PER_CHUNK as i32 { return None; }
        let chunk = self.get_chunk(chunk_part_position.xz())?;
        Some(&chunk.parts[chunk_part_position.y as usize])
    }

    #[inline]
    pub fn get_mut_chunk_part(&mut self, chunk_part_position: Vector3<i32>) -> Option<&mut ChunkPart> {
        if chunk_part_position.y.is_negative() || chunk_part_position.y >= PARTS_PER_CHUNK as i32 { return None; }
        let chunk = self.get_mut_chunk(chunk_part_position.xz())?;
        Some(&mut chunk.parts[chunk_part_position.y as usize])
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
    pub fn values_mut(&mut self) -> ValuesMut<Vector2<i32>, Arc<Chunk>> {
        self.chunks.values_mut()
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
    pub fn get_block(&self, position: GlobalVecU) -> Option<&Block> {
        let chunk = self.get_chunk(position.chunk.xz())?;
        let position = ChunkPosition::try_from(position).ok()?;
        Some(chunk.get_block(position))
    }

    #[inline]
    pub fn set_block(&mut self, position: GlobalVecU, block: Block) {
        let Some(chunk) = self.get_mut_chunk(position.chunk.xz()) else { return; };
        let Ok(position) = ChunkPosition::try_from(position) else { return; };
        chunk.set_block(position, block);
    }

    #[inline]
    pub fn get_light_level(&self, position: GlobalVecU) -> Option<LightLevel> {
        let chunk = self.get_chunk(position.chunk.xz())?;
        let position = ChunkPosition::try_from(position).ok()?;
        Some(chunk.get_light_level(position))
    }

    #[inline]
    pub fn set_light_level(&mut self, position: GlobalVecU, light_level: LightLevel) {
        let Some(chunk) = self.get_mut_chunk(position.chunk.xz()) else { return; };
        let Ok(position) = ChunkPosition::try_from(position) else { return; };
        chunk.set_light_level(position, light_level);
    }
}
