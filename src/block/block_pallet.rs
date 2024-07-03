use super::Block;

pub type BlockPalletItemId = u16;

pub struct BlockPallet {
    items: Vec<Option<BlockPalletItem>>
}

pub struct BlockPalletItem {
    block: Block,
    count: u16,
}

impl BlockPallet {
    pub fn new_air() -> Self {
        todo!()
    }
}