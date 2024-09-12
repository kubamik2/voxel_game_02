use cgmath::{Vector2, Vector3};

use crate::{block::{block_pallet::{BlockPallet, BlockPalletItemId}, light::LightLevel, Block}, world::{chunk::{chunk_map::ChunkMap, chunk_part::{chunk_part_position::ChunkPartPosition, ChunkPart, CHUNK_SIZE_U32}}, PARTS_PER_CHUNK}};

use super::CHUNK_SIZE;

#[derive(Clone, Debug)]
pub struct ExpandedChunkPart {
    pub block_pallet_ids: [BlockPalletItemId; Self::SIZE * Self::SIZE * Self::SIZE],
    pub light_levels: [LightLevel; Self::SIZE * Self::SIZE * Self::SIZE],
    pub block_pallet: BlockPallet
}

impl ExpandedChunkPart {
    const SIZE: usize = (CHUNK_SIZE + 2);

    #[inline]
    fn convert_index(index: (u32, u32, u32)) -> usize {
        index.0 as usize + index.2 as usize * Self::SIZE + index.1 as usize * Self::SIZE * Self::SIZE
    }
    
    #[inline]
    pub fn index_inner_block_pallet_id(&self, index: (u32, u32, u32)) -> &BlockPalletItemId {
        self.index_block_pallet_id((index.0 + 1, index.1 + 1, index.2 + 1))
    }

    #[inline]
    pub fn index_mut_inner_block_pallet_id(&mut self, index: (u32, u32, u32)) -> &mut BlockPalletItemId {
        self.index_mut_block_pallet_id((index.0 + 1, index.1 + 1, index.2 + 1))
    }

    #[inline]
    pub fn index_block_pallet_id(&self, index: (u32, u32, u32)) -> &BlockPalletItemId {
        &self.block_pallet_ids[Self::convert_index(index)]
    }

    #[inline]
    pub fn index_mut_block_pallet_id(&mut self, index: (u32, u32, u32)) -> &mut BlockPalletItemId {
        &mut self.block_pallet_ids[Self::convert_index(index)]
    }

    #[inline]
    pub fn index_inner_block(&self, index: (u32, u32, u32)) -> &Block {
        let id = self.index_inner_block_pallet_id(index);
        &self.block_pallet.get(id).unwrap().block
    }

    #[inline]
    pub fn index_block(&self, index: (u32, u32, u32)) -> &Block {
        let id = self.index_block_pallet_id(index);
        &self.block_pallet.get(id).unwrap().block
    }

    #[inline]
    pub fn index_light_level(&self, index: (u32, u32, u32)) -> &LightLevel {
        &self.light_levels[Self::convert_index(index)]
    }

    #[inline]
    pub fn index_inner_light_level(&self, index: (u32, u32, u32)) -> &LightLevel {
        self.index_light_level((index.0 + 1, index.1 + 1, index.2 + 1))
    }

    #[inline]
    pub fn index_mut_light_level(&mut self, index: (u32, u32, u32)) -> &mut LightLevel {
        &mut self.light_levels[Self::convert_index(index)]
    }

    #[inline]
    pub fn index_mut_inner_light_level(&mut self, index: (u32, u32, u32)) -> &mut LightLevel {
        self.index_mut_light_level((index.0 + 1, index.1 + 1, index.2 + 1))
    }

    pub fn new(chunk_map: &ChunkMap, chunk_pos: Vector2<i32>, chunk_part_index: usize) -> Option<Self> {
        // let now = std::time::Instant::now();
        let Some(chunk) = chunk_map.get_chunk(chunk_pos) else { return None; };
        let Some(chunk_part) = chunk.parts.get(chunk_part_index) else { return None; };

        let mut expanded_chunk_part = Self {
            block_pallet_ids: std::array::from_fn(|_| 0),
            light_levels: std::array::from_fn(|_| LightLevel::new(0, 0).unwrap()),
            block_pallet: chunk_part.block_pallet.clone()
        };

        for y in 0..CHUNK_SIZE as u32 {
            for z in 0..CHUNK_SIZE as u32 {
                for x in 0..CHUNK_SIZE as u32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3::new(x, y, z)) };
                    *expanded_chunk_part.index_mut_inner_block_pallet_id((x, y, z)) = chunk_part.block_layers[position];
                    *expanded_chunk_part.index_mut_inner_light_level((x, y, z)) = chunk_part.light_level_layers[position];
                }
            }
        }

        #[inline]
        fn get_id_or_insert(expanded_chunk_part: &mut ExpandedChunkPart, chunk_part: &ChunkPart, position: ChunkPartPosition) -> BlockPalletItemId {
            let block = chunk_part.get_block(position);
            match expanded_chunk_part.block_pallet.get_block_pallet_id(block) {
                Some(id) => id,
                None => expanded_chunk_part.block_pallet.insert_block(block.clone())
            }
        }
        

        // +x
        if let Some(chunk) = chunk_map.get_chunk(Vector2::new(chunk_pos.x + 1, chunk_pos.y)) {
            let chunk_part = &chunk.parts[chunk_part_index];

            for y in 0..CHUNK_SIZE_U32 {
                for z in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: 0, y: y, z: z }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE_U32 + 1, y + 1, z + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((CHUNK_SIZE_U32 + 1, y + 1, z + 1)) = chunk_part.light_level_layers[position];
                }
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                for z in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: 0, y: 0, z: z }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1, z + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1, z + 1)) = chunk_part.light_level_layers[position];
                }
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                for z in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: 0, y: CHUNK_SIZE_U32 - 1, z: z }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE_U32 + 1, 0, z + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((CHUNK_SIZE_U32 + 1, 0, z + 1)) = chunk_part.light_level_layers[position];
                }
            }
        }

        // -x
        if let Some(chunk) = chunk_map.get_chunk(Vector2::new(chunk_pos.x - 1, chunk_pos.y)) {
            let chunk_part = &chunk.parts[chunk_part_index];

            for y in 0..CHUNK_SIZE_U32 {
                for z in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: CHUNK_SIZE_U32 - 1, y: y, z: z }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((0, y + 1, z + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((0, y + 1, z + 1)) = chunk_part.light_level_layers[position];
                }
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                for z in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: CHUNK_SIZE_U32 - 1, y: 0, z: z }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((0, CHUNK_SIZE_U32 + 1, z + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((0, CHUNK_SIZE_U32 + 1, z + 1)) = chunk_part.light_level_layers[position];
                }
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                for z in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: CHUNK_SIZE_U32 - 1, y: CHUNK_SIZE_U32 - 1, z: z }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((0, 0, z + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((0, 0, z + 1)) = chunk_part.light_level_layers[position];
                }
            }
        }

        // +z
        if let Some(chunk) = chunk_map.get_chunk(Vector2::new(chunk_pos.x, chunk_pos.y + 1)) {
            let chunk_part = &chunk.parts[chunk_part_index];

            for y in 0..CHUNK_SIZE_U32 {
                for x in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: x, y: y, z: 0 }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, y + 1, CHUNK_SIZE_U32 + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((x + 1, y + 1, CHUNK_SIZE_U32 + 1)) = chunk_part.light_level_layers[position];
                }
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                for x in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: x, y: 0, z: 0 }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((x + 1, CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1)) = chunk_part.light_level_layers[position];
                }
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                for x in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: x, y: CHUNK_SIZE_U32 - 1, z: 0 }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, 0, CHUNK_SIZE_U32 + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((x + 1, 0, CHUNK_SIZE_U32 + 1)) = chunk_part.light_level_layers[position];
                }
            }
        }

        // -z
        if let Some(chunk) = chunk_map.get_chunk(Vector2::new(chunk_pos.x, chunk_pos.y - 1)) {
            let chunk_part = &chunk.parts[chunk_part_index];

            for y in 0..CHUNK_SIZE_U32 {
                for x in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: x, y: y, z: CHUNK_SIZE_U32 - 1 }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, y + 1, 0)) = id;
                    *expanded_chunk_part.index_mut_light_level((x + 1, y + 1, 0)) = chunk_part.light_level_layers[position];
                }
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                for x in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: x, y: 0, z: CHUNK_SIZE_U32 - 1 }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, CHUNK_SIZE_U32 + 1, 0)) = id;
                    *expanded_chunk_part.index_mut_light_level((x + 1, CHUNK_SIZE_U32 + 1, 0)) = chunk_part.light_level_layers[position];
                }
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                for x in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: x, y: CHUNK_SIZE_U32 - 1, z: CHUNK_SIZE_U32 - 1 }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, 0, 0)) = id;
                    *expanded_chunk_part.index_mut_light_level((x + 1, 0, 0)) = chunk_part.light_level_layers[position];
                }
            }
        }

        // +y
        if chunk_part_index < PARTS_PER_CHUNK - 1 {
            let chunk_part = &chunk.parts[chunk_part_index + 1];
            for z in 0..CHUNK_SIZE_U32 {
                for x in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: x, y: 0, z: z }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, CHUNK_SIZE_U32 + 1, z + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((x + 1, CHUNK_SIZE_U32 + 1, z + 1)) = chunk_part.light_level_layers[position];
                }
            }
        }

        // -y
        if chunk_part_index > 0 {
            let chunk_part = &chunk.parts[chunk_part_index - 1];
            for z in 0..CHUNK_SIZE_U32 {
                for x in 0..CHUNK_SIZE_U32 {
                    let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: x, y: CHUNK_SIZE_U32 - 1, z: z }) };
                    let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                    *expanded_chunk_part.index_mut_block_pallet_id((x + 1, 0, z + 1)) = id;
                    *expanded_chunk_part.index_mut_light_level((x + 1, 0, z + 1)) = chunk_part.light_level_layers[position];
                }
            }
        }

        // corner +x +z
        if let Some(chunk) = chunk_map.get_chunk(chunk_pos.map(|f| f + 1)) {
            let chunk_part = &chunk.parts[chunk_part_index];
            for y in 0..CHUNK_SIZE_U32 {
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: 0, y: y, z: 0 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE_U32 + 1, y + 1, CHUNK_SIZE_U32 + 1)) = id;
                *expanded_chunk_part.index_mut_light_level((CHUNK_SIZE_U32 + 1, y + 1, CHUNK_SIZE_U32 + 1)) = chunk_part.light_level_layers[position];
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: 0, y: 0, z: 0 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1)) = id;
                *expanded_chunk_part.index_mut_light_level((CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1)) = chunk_part.light_level_layers[position];
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: 0, y: CHUNK_SIZE_U32 - 1, z: 0 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE_U32 + 1, 0, CHUNK_SIZE_U32 + 1)) = id;
                *expanded_chunk_part.index_mut_light_level((CHUNK_SIZE_U32 + 1, 0, CHUNK_SIZE_U32 + 1)) = chunk_part.light_level_layers[position];
            }
        }

        // corner -x +z
        if let Some(chunk) = chunk_map.get_chunk(Vector2::new(chunk_pos.x - 1, chunk_pos.y + 1)) {
            let chunk_part = &chunk.parts[chunk_part_index];
            for y in 0..CHUNK_SIZE_U32 {
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: CHUNK_SIZE_U32 - 1, y: y, z: 0 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((0, y + 1, CHUNK_SIZE_U32 + 1)) = id;
                *expanded_chunk_part.index_mut_light_level((0, y + 1, CHUNK_SIZE_U32 + 1)) = chunk_part.light_level_layers[position];
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: CHUNK_SIZE_U32 - 1, y: 0, z: 0 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((0, CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1)) = id;
                *expanded_chunk_part.index_mut_light_level((0, CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1)) = chunk_part.light_level_layers[position];
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: CHUNK_SIZE_U32 - 1, y: CHUNK_SIZE_U32 - 1, z: 0 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((0, 0, CHUNK_SIZE_U32 + 1)) = id;
                *expanded_chunk_part.index_mut_light_level((0, 0, CHUNK_SIZE_U32 + 1)) = chunk_part.light_level_layers[position];
            }
        }

        // corner +x -z
        if let Some(chunk) = chunk_map.get_chunk(Vector2::new(chunk_pos.x + 1, chunk_pos.y - 1)) {
            let chunk_part = &chunk.parts[chunk_part_index];
            for y in 0..CHUNK_SIZE_U32 {
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: 0, y: y, z: CHUNK_SIZE_U32 - 1 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE_U32 + 1, y + 1, 0)) = id;
                *expanded_chunk_part.index_mut_light_level((CHUNK_SIZE_U32 + 1, y + 1, 0)) = chunk_part.light_level_layers[position];
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: 0, y: 0, z: CHUNK_SIZE_U32 - 1 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1, 0)) = id;
                *expanded_chunk_part.index_mut_light_level((CHUNK_SIZE_U32 + 1, CHUNK_SIZE_U32 + 1, 0)) = chunk_part.light_level_layers[position];
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: 0, y: CHUNK_SIZE_U32 - 1, z: CHUNK_SIZE_U32 - 1 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((CHUNK_SIZE_U32 + 1, 0, 0)) = id;
                *expanded_chunk_part.index_mut_light_level((CHUNK_SIZE_U32 + 1, 0, 0)) = chunk_part.light_level_layers[position];
            }
        }

        // corner -x -z
        if let Some(chunk) = chunk_map.get_chunk(chunk_pos.map(|f| f - 1)) {
            let chunk_part = &chunk.parts[chunk_part_index];
            for y in 0..CHUNK_SIZE_U32 {
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: CHUNK_SIZE_U32 - 1, y: y, z: CHUNK_SIZE_U32 - 1 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((0, y + 1, 0)) = id;
                *expanded_chunk_part.index_mut_light_level((0, y + 1, 0)) = chunk_part.light_level_layers[position];
            }

            if chunk_part_index < PARTS_PER_CHUNK - 1 {
                let chunk_part = &chunk.parts[chunk_part_index + 1];
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: CHUNK_SIZE_U32 - 1, y: 0, z: CHUNK_SIZE_U32 - 1 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((0, CHUNK_SIZE_U32 + 1, 0)) = id;
                *expanded_chunk_part.index_mut_light_level((0, CHUNK_SIZE_U32 + 1, 0)) = chunk_part.light_level_layers[position];
            }

            if chunk_part_index > 0 {
                let chunk_part = &chunk.parts[chunk_part_index - 1];
                let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x: CHUNK_SIZE_U32 - 1, y: CHUNK_SIZE_U32 - 1, z: CHUNK_SIZE_U32 - 1 }) };
                let id = get_id_or_insert(&mut expanded_chunk_part, chunk_part, position);
                *expanded_chunk_part.index_mut_block_pallet_id((0, 0, 0)) = id;
                *expanded_chunk_part.index_mut_light_level((0, 0, 0)) = chunk_part.light_level_layers[position];
            }
        }

        // dbg!(now.elapsed());
        Some(expanded_chunk_part)
    }
}
