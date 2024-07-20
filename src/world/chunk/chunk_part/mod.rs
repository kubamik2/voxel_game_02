pub mod chunk_part_mesher;
pub mod expanded_chunk_part;

use std::ops::Index;

use rand::Rng;

use crate::{block::{block_pallet::{BlockPallet, BlockPalletItemId}, Block}, world::{CHUNK_HEIGHT, PARTS_PER_CHUNK}, BLOCK_MAP};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;
pub const CHUNK_SIZE_F64: f64 = CHUNK_SIZE as f64;



pub struct ChunkPart {
    pub block_pallet: BlockPallet,
    pub block_layers: BlockLayers,
}

impl ChunkPart {
    pub fn new_random() -> Self {
        let block_pallet = BlockPallet::new_air();
        let block_layers = BlockLayers::new_uncompressed();
        let mut chunk_part = Self { block_layers, block_pallet };
        let cobblestone_id = chunk_part.block_pallet.insert_count(BLOCK_MAP.get("cobblestone").unwrap().clone().into(), 0);
        let dirt_id = chunk_part.block_pallet.insert_count(BLOCK_MAP.get("dirt").unwrap().clone().into(), 0);

        let mut rng = rand::thread_rng();
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if rng.gen_bool(0.5) { continue; }
                    if rng.gen_bool(0.5) {
                        chunk_part.set_block_pallet_id((x, y, z), cobblestone_id);
                    } else {
                        chunk_part.set_block_pallet_id((x, y, z), dirt_id);
                    }
                }
            }
        }
        chunk_part.set_block_pallet_id((0, 0, 0), 0);

        chunk_part.block_pallet.clean_up();
        chunk_part.block_layers.compress();

        chunk_part
    }

    pub fn new_air() -> Self {
        let block_pallet = BlockPallet::new_air();
        let block_layers = BlockLayers::new_uncompressed();
        Self { block_layers, block_pallet }
    }

    pub fn set_block(&mut self, index: (usize, usize, usize), block: Block) {
        let old_block_pallet_id = self.block_layers[index];
        let block_pallet_item = self.block_pallet.get_mut(&old_block_pallet_id).unwrap();
        assert!(block_pallet_item.count > 0);
        block_pallet_item.count += 1;

        let block_pallet_id = if let Some((id, item)) = self.block_pallet.find_item_mut(&block) {
            item.count += 1;
            id
        } else {
            self.block_pallet.insert_block(block)
        };

        self.block_layers.set_block_pallet_id(index, block_pallet_id)
    }

    pub fn set_available_block(&mut self, index: (usize, usize, usize), block: Block) {
        let (block_pallet_id, item) = self.block_pallet.find_item_mut(&block).unwrap();
        let old_block_pallet_id = self.block_layers[index];
        if block_pallet_id == old_block_pallet_id { return; }

        item.count += 1;
        self.block_layers.set_block_pallet_id(index, block_pallet_id);
        let old_item = self.block_pallet.get_mut(&old_block_pallet_id).unwrap();
        assert!(old_item.count > 0);
        old_item.count -= 1;
    }

    pub fn set_block_pallet_id(&mut self, index: (usize, usize, usize), block_pallet_id: BlockPalletItemId) {
        let old_block_pallet_id = self.block_layers[index];
        if block_pallet_id == old_block_pallet_id { return; } 

        self.block_pallet.get_mut(&block_pallet_id).unwrap().count += 1;

        let old_block_pallet_item = self.block_pallet.get_mut(&old_block_pallet_id).unwrap();
        assert!(old_block_pallet_item.count > 0);
        old_block_pallet_item.count -= 1;

        self.block_layers.set_block_pallet_id(index, block_pallet_id);
    }
}

impl Index<(usize, usize, usize)> for ChunkPart {
    type Output = Block;
    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        let block_pallet_id = self.block_layers[index];
        let block_pallet_item = self.block_pallet.get(&block_pallet_id).unwrap();
        &block_pallet_item.block
    }
}

pub struct BlockLayers([BlockLayer; CHUNK_SIZE]);

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


    pub fn set_block_pallet_id(&mut self, index: (usize, usize, usize), block_pallet_id: BlockPalletItemId) {
        assert!(index.0 < CHUNK_SIZE && index.1 < CHUNK_SIZE && index.2 < CHUNK_SIZE);
        let layer = &mut self.0[index.1];
        match layer {
            BlockLayer::Uncompressed(block_ids) => {
                block_ids[index.0 + index.2 * CHUNK_SIZE] = block_pallet_id;
            },
            BlockLayer::Compressed(block_id) => {
                let block_id = *block_id;
                if block_id == block_pallet_id {
                    return;
                }
                let mut raw_block_ids = Box::new([block_id; Self::LAYER_SIZE]);
                raw_block_ids[index.0 + index.2 * CHUNK_SIZE] = block_pallet_id;
                *layer = BlockLayer::Uncompressed(raw_block_ids);
            }
        }
    }
}

impl Index<(usize, usize, usize)> for BlockLayers {
    type Output = BlockPalletItemId;
    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        let layer = &self.0[index.1];
        match layer {
            BlockLayer::Uncompressed(blocks) => &blocks[index.0 + index.2 * CHUNK_SIZE],
            BlockLayer::Compressed(block) => block,
        }
    }
}

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
