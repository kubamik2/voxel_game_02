pub mod chunk_part_mesher;
pub mod expanded_chunk_part;
pub mod chunk_part_position;

use std::ops::{Index, IndexMut};

use cgmath::Vector3;
use chunk_part_position::ChunkPartPosition;

use crate::{block::{block_pallet::{BlockPallet, BlockPalletItemId}, light::{LightLevel, LightNode}, Block}, BLOCK_LIST};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;
pub const CHUNK_SIZE_F64: f64 = CHUNK_SIZE as f64;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_I16: i16 = CHUNK_SIZE as i16;
pub const CHUNK_SIZE_U32: u32 = CHUNK_SIZE as u32;
pub const INVERSE_CHUNK_SIZE: f32 = 1.0 / CHUNK_SIZE_F32;


#[derive(Clone)]
pub struct ChunkPart {
    pub block_pallet: BlockPallet,
    pub block_layers: BlockLayers,
    pub light_level_layers: LightLevelLayers,
    pub added_light_emitters: Vec<Vector3<usize>>,
    pub removed_light_emitters: Vec<Vector3<usize>>,
}

impl ChunkPart {
    pub fn new_air() -> Self {
        let block_pallet = BlockPallet::new_air();
        let block_layers = BlockLayers::new_uncompressed();
        let light_level_layers = LightLevelLayers::new_uncompressed();
        Self { block_layers, block_pallet, light_level_layers, added_light_emitters: vec![], removed_light_emitters: vec![] }
    }

    #[inline]
    fn position_in_bounds(local_position: Vector3<usize>) -> bool {
        local_position.x < CHUNK_SIZE && local_position.y < CHUNK_SIZE && local_position.z < CHUNK_SIZE
    }

    #[inline]
    pub fn set_block(&mut self, local_position: ChunkPartPosition, block: Block) {
        let new_block_light_level = block.properties().emitted_light;

        let old_block_pallet_id = self.block_layers.get_block_pallet_id(local_position);
        let block_pallet_item = self.block_pallet.get_mut(old_block_pallet_id).unwrap();
        block_pallet_item.count += 1;

        let block_pallet_id = if let Some((id, item)) = self.block_pallet.find_item_mut(&block) {
            item.count += 1;
            id
        } else {
            self.block_pallet.insert_block(block)
        };

        self.block_layers.set_block_pallet_id(local_position, block_pallet_id);
        
        self.set_block_light_level(local_position, new_block_light_level);
        self.set_sky_light_level(local_position, 0);
    }

    #[inline]
    pub fn set_block_pallet_id(&mut self, local_position: ChunkPartPosition, block_pallet_id: BlockPalletItemId) {
        let old_block_pallet_id = self.block_layers[local_position];
        if block_pallet_id == old_block_pallet_id { return; } 

        self.block_pallet.get_mut(&block_pallet_id).unwrap().count += 1;

        let old_block_pallet_item = self.block_pallet.get_mut(&old_block_pallet_id).unwrap();
        assert!(old_block_pallet_item.count > 0);
        old_block_pallet_item.count -= 1;

        self.block_layers.set_block_pallet_id(local_position, block_pallet_id);
    }

    #[inline]
    pub fn get_block(&self, position: ChunkPartPosition) -> &Block {
        let block_pallet_id = self.block_layers.get_block_pallet_id(position);
        &self.block_pallet.get(block_pallet_id).unwrap().block
    }

    #[inline]
    pub fn get_block_light_level(&self, position: ChunkPartPosition) -> u8 {
        let light_level = self.light_level_layers.get_light_level(position);
        light_level.get_block()
    }

    #[inline]
    pub fn set_block_light_level(&mut self, position: ChunkPartPosition, level: u8) {
        let mut light_level = *self.light_level_layers.get_light_level(position);
        light_level.set_block(level);
        self.light_level_layers.set_light_level(position, light_level);
    }

    #[inline]
    pub fn get_sky_light_level(&self, position: ChunkPartPosition) -> u8 {
        let light_level = self.light_level_layers.get_light_level(position);
        light_level.get_sky()
    }

    #[inline]
    pub fn set_sky_light_level(&mut self, position: ChunkPartPosition, level: u8) {
        let mut light_level = *self.light_level_layers.get_light_level(position);
        light_level.set_sky(level);
        self.light_level_layers.set_light_level(position, light_level);
    }

    #[inline]
    pub fn get_light_level(&self, position: ChunkPartPosition) -> LightLevel {
        *self.light_level_layers.get_light_level(position)
    }


    #[inline]
    pub fn set_light_level(&mut self, position: ChunkPartPosition, light_level: LightLevel) {
        self.light_level_layers.set_light_level(position, light_level);
    }

    #[inline]
    pub fn compress(&mut self) {
        self.block_layers.compress();
        self.light_level_layers.compress();
    }
}

#[derive(Clone)]
pub struct BlockLayers(pub [BlockLayer; CHUNK_SIZE]);

impl BlockLayers {
    const LAYER_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE;
    pub fn new_compressed() -> Self {
        Self(std::array::from_fn(|_| BlockLayer::Compressed(0)))
    }

    pub fn new_uncompressed() -> Self {
        Self(std::array::from_fn(|_| BlockLayer::Uncompressed(Box::new([0; Self::LAYER_SIZE]))))
    }


    #[inline]
    pub fn compress(&mut self) {
        for layer in self.0.iter_mut() {
            if layer.can_be_compressed() {
                let first_block_id = if let BlockLayer::Uncompressed(block_ids) = layer {
                    block_ids[0]
                } else { continue; };
                *layer = BlockLayer::Compressed(first_block_id);
            }
        }
    }

    #[inline]
    pub fn uncompress(&mut self) {
        for layer in self.0.iter_mut() {
            if let BlockLayer::Compressed(block_pallet_id) = layer {
                *layer = BlockLayer::Uncompressed(Box::new([*block_pallet_id; CHUNK_SIZE * CHUNK_SIZE]));
            }
        }
    }

    #[inline]
    pub fn set_block_pallet_id(&mut self, position: ChunkPartPosition, block_pallet_id: BlockPalletItemId) {
        let layer = &mut self.0[position.y as usize];
        match layer {
            BlockLayer::Uncompressed(block_ids) => {
                block_ids[position.x as usize + position.z as usize * CHUNK_SIZE] = block_pallet_id;
            },
            BlockLayer::Compressed(block_id) => {
                let block_id = *block_id;
                if block_id == block_pallet_id {
                    return;
                }
                let mut raw_block_ids = Box::new([block_id; Self::LAYER_SIZE]);
                raw_block_ids[position.x as usize + position.z as usize * CHUNK_SIZE] = block_pallet_id;
                *layer = BlockLayer::Uncompressed(raw_block_ids);
            }
        }
    }

    #[inline]
    pub fn get_block_pallet_id(&self, position: ChunkPartPosition) -> &BlockPalletItemId {
        &self[position]
    }
}

impl Index<ChunkPartPosition> for BlockLayers {
    type Output = BlockPalletItemId;
    #[inline]
    fn index(&self, index: ChunkPartPosition) -> &Self::Output {
        let layer = &self.0[index.y as usize];
        match layer {
            BlockLayer::Uncompressed(blocks) => &blocks[index.x as usize + index.z as usize * CHUNK_SIZE],
            BlockLayer::Compressed(block) => block,
        }
    }
}

#[derive(Clone)]
pub enum BlockLayer {
    Uncompressed(Box<[BlockPalletItemId; CHUNK_SIZE * CHUNK_SIZE]>),
    Compressed(BlockPalletItemId)
}

impl BlockLayer {
    #[inline]
    pub fn can_be_compressed(&self) -> bool {
        match self {
            Self::Uncompressed(block_ids) => {
                let first_block_id = block_ids[0];
                for block_id in block_ids.iter().skip(1) {
                    if first_block_id != *block_id {
                        return false;
                    }
                }
                true
            },
            Self::Compressed(_) => false
        }
    }
}

#[derive(Clone)]
pub struct LightLevelLayers([LightLevelLayer; CHUNK_SIZE]);

impl LightLevelLayers {
    #[inline]
    pub fn new_uncompressed() -> Self {
        Self(std::array::from_fn(|_| LightLevelLayer::Uncompressed(Box::new([LightLevel::new(0, 0).unwrap(); CHUNK_SIZE * CHUNK_SIZE]))))
    }

    #[inline]
    pub fn new_compressed() -> Self {
        Self(std::array::from_fn(|_| LightLevelLayer::Compressed(LightLevel::new(0, 0).unwrap())))
    }

    #[inline]
    pub fn compress(&mut self) {
        for layer in self.0.iter_mut() {
            if layer.can_be_compressed() {
                let first_light_level = if let LightLevelLayer::Uncompressed(light_levels) = layer {
                    light_levels[0]
                } else { continue; };
                *layer = LightLevelLayer::Compressed(first_light_level);
            }
        }
    }

    #[inline]
    pub fn uncompress(&mut self) {
        for layer in self.0.iter_mut() {
            if let LightLevelLayer::Compressed(light_level) = layer {
                *layer = LightLevelLayer::Uncompressed(Box::new([*light_level; CHUNK_SIZE * CHUNK_SIZE]))
            }
        }
    }

    #[inline]
    pub fn set_light_level(&mut self, position: ChunkPartPosition, light_level: LightLevel) {
        let layer = &mut self.0[position.y as usize];
        match layer {
            LightLevelLayer::Uncompressed(light_levels) => {
                light_levels[position.x as usize + position.z as usize * CHUNK_SIZE] = light_level;
            },
            LightLevelLayer::Compressed(layer_light_level) => {
                let layer_light_level = *layer_light_level;
                if layer_light_level == light_level {
                    return;
                }
                let mut raw_light_levels = Box::new([layer_light_level; CHUNK_SIZE * CHUNK_SIZE]);
                raw_light_levels[position.x as usize + position.z as usize * CHUNK_SIZE] = light_level;
                *layer = LightLevelLayer::Uncompressed(raw_light_levels);
            }
        }
    }

    #[inline]
    pub fn get_light_level(&self, position: ChunkPartPosition) -> &LightLevel {
        let layer = &self.0[position.y as usize];
        match layer {
            LightLevelLayer::Uncompressed(light_levels) => {
                &light_levels[position.x as usize + position.z as usize * CHUNK_SIZE]
            },
            LightLevelLayer::Compressed(layer_light_level) => {
                layer_light_level
            }
        }
    }
}

impl Index<ChunkPartPosition> for LightLevelLayers {
    type Output = LightLevel;
    #[inline]
    fn index(&self, index: ChunkPartPosition) -> &Self::Output {
        let layer = &self.0[index.y as usize];
        match layer {
            LightLevelLayer::Uncompressed(light_levels) => {
                &light_levels[index.x as usize + index.z as usize * CHUNK_SIZE]
            },
            LightLevelLayer::Compressed(light_level) => {
                light_level
            }
        }
    }
}

#[derive(Clone)]
pub enum LightLevelLayer {
    Uncompressed(Box<[LightLevel; CHUNK_SIZE * CHUNK_SIZE]>),
    Compressed(LightLevel)
}

impl LightLevelLayer {
    #[inline]
    pub fn can_be_compressed(&self) -> bool {
        match self {
            Self::Uncompressed(light_levels) => {
                let first_light_level = light_levels[0];
                for light_level in light_levels.iter().skip(1) {
                    if first_light_level != *light_level {
                        return false;
                    }
                }
                true
            },
            Self::Compressed(_) => false
        }
    }
}
