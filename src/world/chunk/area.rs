use std::{mem::MaybeUninit, sync::Arc};

use cgmath::{Vector2, Vector3};

use crate::{block::Block, world::structure::Structure, world::PARTS_PER_CHUNK};

use super::{chunk_map::ChunkMap, chunk_part::CHUNK_SIZE, Chunk};

pub struct Area {
    pub chunks: [Arc<Chunk>; 9]
}

impl Area {
    #[inline]
    fn chunk_index(offset: Vector2<i32>) -> Option<usize> {
        let index = offset.x + offset.y * 3 + 4;
        if index < 0 || index > 8 { return None; }
        Some(index as usize)
    }

    #[inline]
    pub fn get_chunk(&self, offset: Vector2<i32>) -> Option<&Chunk> {
        let Some(index) = Self::chunk_index(offset) else { return None; };
        Some(&self.chunks[index])
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, offset: Vector2<i32>) -> Option<&mut Chunk>{
        let Some(index) = Self::chunk_index(offset) else { return None; };
        let chunk = &mut self.chunks[index];

        assert!(Arc::strong_count(chunk) == 1);

        let chunk_mut = Arc::make_mut(chunk);
        Some(chunk_mut)
    }

    #[inline]
    pub fn get_block(&self, position: Vector3<i32>) -> Option<&Block> {
        let chunk_part_index = position.y / CHUNK_SIZE as i32;
        if chunk_part_index < 0 || chunk_part_index >= PARTS_PER_CHUNK as i32 { return None; }

        let chunk_offset = position.xz().map(|f| f.div_euclid(CHUNK_SIZE as i32));
        let Some(chunk) = self.get_chunk(chunk_offset) else { return None; };

        let position_in_chunk_part = position.map(|f| f.rem_euclid(CHUNK_SIZE as i32) as usize);
        let chunk_part = &chunk.parts[chunk_part_index as usize];
        Some(chunk_part.get_block(position_in_chunk_part))
    }

    #[inline]
    pub fn set_block(&mut self, position: Vector3<i32>, block: Block) {
        let chunk_part_index = position.y / CHUNK_SIZE as i32;
        if chunk_part_index < 0 || chunk_part_index >= PARTS_PER_CHUNK as i32 { return; }

        let chunk_offset = position.xz().map(|f| f.div_euclid(CHUNK_SIZE as i32));
        let Some(chunk) = self.get_chunk_mut(chunk_offset) else { return; };

        let position_in_chunk = position.map(|f| f.rem_euclid(CHUNK_SIZE as i32) as usize);
        let chunk_part = &mut chunk.parts[chunk_part_index as usize];

        chunk_part.set_block(position_in_chunk.into(), block);
    }

    pub fn insert_structure(&mut self, structure: &Structure, origin_point: Vector3<i32>) {
        for (position, block) in structure.blocks().iter().cloned() {
            self.set_block(position + origin_point, block);
        }
    }

    pub fn new(chunk_map: &mut ChunkMap, center_chunk_position: Vector2<i32>) -> Option<Self> {
        for z in -1..=1 {
            for x in -1..=1 {
                if !chunk_map.contains_key(center_chunk_position + Vector2::new(x, z)) { return None; }
            }
        }
        
        let mut chunks: [MaybeUninit<Arc<Chunk>>; 9] = std::array::from_fn(|_| MaybeUninit::uninit());
        let mut index = 0;
        for z in -1..=1 {
            for x in -1..=1 {
                chunks[index] = MaybeUninit::new(chunk_map.remove(center_chunk_position + Vector2::new(x, z)).unwrap());
                index += 1;
            }
        }

        let chunks = unsafe { std::mem::transmute::<_, [Arc<Chunk>; 9]>(chunks) };

        Some(Self { chunks })
    }
}