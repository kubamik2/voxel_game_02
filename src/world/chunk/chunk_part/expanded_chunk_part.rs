use std::ops::{Index, IndexMut};

use cgmath::Vector2;

use crate::{block::{block_pallet::{BlockPallet, BlockPalletItemId}, Block}, world::{chunk::{chunk_map::ChunkMap, chunk_part::ChunkPart}, PARTS_PER_CHUNK}};

use super::CHUNK_SIZE;

#[derive(Clone, Debug)]
pub struct ExpandedChunkPart {
    pub block_pallet_ids: [BlockPalletItemId; Self::SIZE * Self::SIZE * Self::SIZE],
    pub block_pallet: BlockPallet
}

impl ExpandedChunkPart {
    const SIZE: usize = (CHUNK_SIZE + 2);

    #[inline]
    fn convert_index(index: (usize, usize, usize)) -> usize {
        index.0 + index.2 * Self::SIZE + index.1 * Self::SIZE * Self::SIZE
    }
    
    #[inline]
    pub fn index_inner_block_pallet_id(&self, index: (usize, usize, usize)) -> &BlockPalletItemId {
        self.index_block_pallet_id((index.0 + 1, index.1 + 1, index.2 + 1))
    }

    #[inline]
    pub fn index_mut_inner_block_pallet_id(&mut self, index: (usize, usize, usize)) -> &mut BlockPalletItemId {
        self.index_mut_block_pallet_id((index.0 + 1, index.1 + 1, index.2 + 1))
    }

    #[inline]
    pub fn index_block_pallet_id(&self, index: (usize, usize, usize)) -> &BlockPalletItemId {
        &self.block_pallet_ids[Self::convert_index(index)]
    }

    #[inline]
    pub fn index_mut_block_pallet_id(&mut self, index: (usize, usize, usize)) -> &mut BlockPalletItemId {
        &mut self.block_pallet_ids[Self::convert_index(index)]
    }

    #[inline]
    pub fn index_inner_block(&self, index: (usize, usize, usize)) -> &Block {
        let id = self.index_inner_block_pallet_id(index);
        &self.block_pallet.get(id).unwrap().block
    }

    #[inline]
    pub fn index_block(&self, index: (usize, usize, usize)) -> &Block {
        let id = self.index_block_pallet_id(index);
        &self.block_pallet.get(id).unwrap().block
    }

    pub fn new(chunk_map: &ChunkMap, chunk_pos: Vector2<i32>, chunk_part_index: usize) -> Option<Self> {
        let Some(chunk) = chunk_map.get(chunk_pos).map(|f| f.lock().unwrap()) else { return None; };
        let Some(chunk_part) = chunk.parts.get(chunk_part_index) else { return None; };

        let mut expanded_chunk_part = Self {
            block_pallet_ids: std::array::from_fn(|_| 0),
            block_pallet: chunk_part.block_pallet.clone()
        };

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    *expanded_chunk_part.index_mut_inner_block_pallet_id((x, y, z)) = chunk_part.block_layers[(x, y, z)];
                }
            }
        }

        #[inline]
        fn get_id_or_insert(expanded_chunk_part: &mut ExpandedChunkPart, chunk_part: &ChunkPart, index: (usize, usize, usize)) -> BlockPalletItemId {
            let block = &chunk_part[index];
            match expanded_chunk_part.block_pallet.get_block_pallet_id(block) {
                Some(id) => id,
                None => expanded_chunk_part.block_pallet.insert_block(block.clone())
            }
        }
        

        // +x
        if let Some(chunk) = chunk_map.get(Vector2::new(chunk_pos.x + 1, chunk_pos.y)).map(|f| f.lock().unwrap()) {
            let chunk_part = &chunk.parts[chunk_part_index];

            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (0, y, z));
                    *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE + 1, y + 1, z + 1)) = id;
                }
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                for z in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (0, 0, z));
                    *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE + 1, CHUNK_SIZE + 1, z + 1)) = id;
                }
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                for z in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (0, CHUNK_SIZE - 1, z));
                    *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE + 1, 0, z + 1)) = id;
                }
            }
        }

        // -x
        if let Some(chunk) = chunk_map.get(Vector2::new(chunk_pos.x - 1, chunk_pos.y)).map(|f| f.lock().unwrap()) {
            let chunk_part = &chunk.parts[chunk_part_index];

            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (CHUNK_SIZE - 1, y, z));
                    *expanded_chunk_part.index_mut_block_pallet_id((0, y + 1, z + 1)) = id;
                }
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                for z in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (CHUNK_SIZE - 1, 0, z));
                    *expanded_chunk_part.index_mut_block_pallet_id((0, CHUNK_SIZE + 1, z + 1)) = id;
                }
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                for z in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (CHUNK_SIZE - 1, CHUNK_SIZE - 1, z));
                    *expanded_chunk_part.index_mut_block_pallet_id((0, 0, z + 1)) = id;
                }
            }
        }

        // +z
        if let Some(chunk) = chunk_map.get(Vector2::new(chunk_pos.x, chunk_pos.y + 1)).map(|f| f.lock().unwrap()) {
            let chunk_part = &chunk.parts[chunk_part_index];

            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (x, y, 0));
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, y + 1, CHUNK_SIZE + 1)) = id;
                }
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                for x in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (x, 0, 0));
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, CHUNK_SIZE + 1, CHUNK_SIZE + 1)) = id;
                }
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                for x in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (x, CHUNK_SIZE - 1, 0));
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, 0, CHUNK_SIZE + 1)) = id;
                }
            }
        }

        // -z
        if let Some(chunk) = chunk_map.get(Vector2::new(chunk_pos.x, chunk_pos.y - 1)).map(|f| f.lock().unwrap()) {
            let chunk_part = &chunk.parts[chunk_part_index];

            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (x, y, CHUNK_SIZE - 1));
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, y + 1, 0)) = id;
                }
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                for x in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (x, 0, CHUNK_SIZE - 1));
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, CHUNK_SIZE + 1, 0)) = id;
                }
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                for x in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (x, CHUNK_SIZE - 1, CHUNK_SIZE - 1));
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, 0, 0)) = id;
                }
            }
        }

        // +y
        if chunk_part_index < PARTS_PER_CHUNK - 1 {
            let chunk_part = &chunk.parts[chunk_part_index + 1];
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (x, 0, z));
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, CHUNK_SIZE + 1, z + 1)) = id;
                }
            }
        }

        // -y
        if chunk_part_index > 0 {
            let chunk_part = &chunk.parts[chunk_part_index - 1];
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (x, CHUNK_SIZE - 1, z));
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, 0, z + 1)) = id;
                }
            }
        }

        // corner +x +z
        if let Some(chunk) = chunk_map.get(chunk_pos.map(|f| f + 1)).map(|f| f.lock().unwrap()) {
            let chunk_part = &chunk.parts[chunk_part_index];
            for y in 0..CHUNK_SIZE {
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (0, y, 0));
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE + 1, y + 1, CHUNK_SIZE + 1)) = id;
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (0, 0, 0));
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE + 1, CHUNK_SIZE + 1, CHUNK_SIZE + 1)) = id;
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (0, CHUNK_SIZE - 1, 0));
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE + 1, 0, CHUNK_SIZE + 1)) = id;
            }
        }

        // corner -x +z
        if let Some(chunk) = chunk_map.get(Vector2::new(chunk_pos.x - 1, chunk_pos.y + 1)).map(|f| f.lock().unwrap()) {
            let chunk_part = &chunk.parts[chunk_part_index];
            for y in 0..CHUNK_SIZE {
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (CHUNK_SIZE - 1, y, 0));
                *expanded_chunk_part.index_mut_block_pallet_id((0, y + 1, CHUNK_SIZE + 1)) = id;
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (CHUNK_SIZE - 1, 0, 0));
                *expanded_chunk_part.index_mut_block_pallet_id((0, CHUNK_SIZE + 1, CHUNK_SIZE + 1)) = id;
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (CHUNK_SIZE - 1, CHUNK_SIZE - 1, 0));
                *expanded_chunk_part.index_mut_block_pallet_id((0, 0, CHUNK_SIZE + 1)) = id;
            }
        }

        // corner +x -z
        if let Some(chunk) = chunk_map.get(Vector2::new(chunk_pos.x + 1, chunk_pos.y - 1)).map(|f| f.lock().unwrap()) {
            let chunk_part = &chunk.parts[chunk_part_index];
            for y in 0..CHUNK_SIZE {
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (0, y, CHUNK_SIZE - 1));
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE + 1, y + 1, 0)) = id;
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (0, 0, CHUNK_SIZE - 1));
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE + 1, CHUNK_SIZE + 1, 0)) = id;
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (0, CHUNK_SIZE - 1, CHUNK_SIZE - 1));
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE + 1, 0, 0)) = id;
            }
        }

        // corner -x -z
        if let Some(chunk) = chunk_map.get(chunk_pos.map(|f| f - 1)).map(|f| f.lock().unwrap()) {
            let chunk_part = &chunk.parts[chunk_part_index];
            for y in 0..CHUNK_SIZE {
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (CHUNK_SIZE - 1, y, CHUNK_SIZE - 1));
                *expanded_chunk_part.index_mut_block_pallet_id((0, y + 1, 0)) = id;
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (CHUNK_SIZE - 1, 0, CHUNK_SIZE - 1));
                *expanded_chunk_part.index_mut_block_pallet_id((0, CHUNK_SIZE + 1, 0)) = id;
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, (CHUNK_SIZE - 1, CHUNK_SIZE - 1, CHUNK_SIZE - 1));
                *expanded_chunk_part.index_mut_block_pallet_id((0, 0, 0)) = id;
            }
        }

        Some(expanded_chunk_part)
    }
}