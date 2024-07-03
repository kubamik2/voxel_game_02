use cgmath::Vector3;

use crate::{block::block_pallet::{BlockPallet, BlockPalletItemId}, world::{CHUNK_HEIGHT, CHUNK_PARTS_PER_REGION}};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;
pub const CHUNK_SIZE_F64: f64 = CHUNK_SIZE as f64;

pub struct ChunkPart {
    block_pallet: BlockPallet,
    block_layers: BlockLayers,
}

pub struct BlockLayers([BlockLayer; CHUNK_PARTS_PER_REGION]);

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


    pub fn set_block_id(&mut self, position: Vector3<usize>, block_pallet_id: BlockPalletItemId) {
        assert!(position.x < CHUNK_SIZE && position.y < CHUNK_SIZE && position.z < CHUNK_SIZE);
        let layer = &mut self.0[position.y];
        match layer {
            BlockLayer::Uncompressed(block_ids) => {
                block_ids[position.x + position.z * CHUNK_SIZE] = block_pallet_id;
            },
            BlockLayer::Compressed(block_id) => {
                let block_id = *block_id;
                if block_id == block_pallet_id {
                    return;
                }
                let mut raw_block_ids = Box::new([block_id; Self::LAYER_SIZE]);
                raw_block_ids[position.x + position.z * CHUNK_SIZE] = block_pallet_id;
                *layer = BlockLayer::Uncompressed(raw_block_ids);
            }
        }
    }
}

// TODO maybe add indexing
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