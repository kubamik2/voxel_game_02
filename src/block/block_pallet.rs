use crate::world::chunk::chunk_part::CHUNK_SIZE;

use super::{block_state::BlockState, Block};

pub type BlockPalletItemId = u16;

#[derive(Debug, Clone)]
pub struct BlockPallet {
    items: Vec<Option<BlockPalletItem>>
}

#[derive(Debug, Clone)]
pub struct BlockPalletItem {
    pub block: Block,
    pub count: BlockPalletItemId,
}

impl BlockPallet {
    pub fn new_air() -> Self {
        let items = vec![Some(
            BlockPalletItem {
                block: Block::new(0, "air", BlockState::new()),
                count: (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE + 1) as BlockPalletItemId,
            },
        )];

        Self { items }
    }

    #[inline]
    fn get_lowest_free_id(&mut self) -> BlockPalletItemId {
        if let Some(block_pallet_id) = self.items.iter().position(|p| p.is_none()) {
            return block_pallet_id as BlockPalletItemId;
        }

        let block_pallet_id = self.items.len() as BlockPalletItemId;
        self.items.push(None);

        assert!(self.items.len() < BlockPalletItemId::MAX as usize);

        block_pallet_id
    }

    #[inline]
    pub fn insert_block_pallet_item(&mut self, block_pallet_item: BlockPalletItem) -> BlockPalletItemId {
        let block_pallet_id = self.get_lowest_free_id();
        self.items[block_pallet_id as usize] = Some(block_pallet_item);
        block_pallet_id
    }

    #[inline]
    pub fn insert_block(&mut self, block: Block) -> BlockPalletItemId {
        if let Some(block_pallet_id) = self.get_block_pallet_id(&block) {
            return block_pallet_id;
        }

        let block_pallet_id = self.get_lowest_free_id();
        let block_pallet_item = BlockPalletItem {
            block,
            count: 1,
        };
        self.items[block_pallet_id as usize] = Some(block_pallet_item);
        block_pallet_id
    }

    #[inline]
    pub fn insert_count(&mut self, block: Block, count: BlockPalletItemId) -> BlockPalletItemId {
        let block_pallet_id = self.get_lowest_free_id();
        let block_pallet_item = BlockPalletItem {
            block,
            count,
        };
        self.items[block_pallet_id as usize] = Some(block_pallet_item);
        block_pallet_id
    }

    #[inline]
    pub fn remove(&mut self, block_pallet_id: &BlockPalletItemId) -> Option<BlockPalletItem> {
        if (*block_pallet_id + 1) as usize == self.items.len() {
            return self.items.pop().flatten();
        }
        self.items[*block_pallet_id as usize].take()
    }

    #[inline]
    pub fn clean_up(&mut self) {
        let mut marked_for_deletion = vec![];

        for (id, item) in self.iter_mut() {
            if item.count == 0 { marked_for_deletion.push(id); }
        }

        for id in marked_for_deletion {
            self.remove(&(id as BlockPalletItemId));
        }
    }

    #[inline]
    pub fn get_block_pallet_id(&self, block: &Block) -> Option<BlockPalletItemId> {
        self.iter().find(|p| p.1.block == *block).map(|f| f.0)
    }

    #[inline]
    pub fn get(&self, block_pallet_id: &BlockPalletItemId) -> Option<&BlockPalletItem> {
        self.items.get(*block_pallet_id as usize).map(|f| f.as_ref()).flatten()
    }

    #[inline]
    pub fn get_mut(&mut self, id: &BlockPalletItemId) -> Option<&mut BlockPalletItem> {
        self.items.get_mut(*id as usize).map(|f| f.as_mut()).flatten()
    }

    #[inline]
    pub fn find_item(&self, block: &Block) -> Option<(BlockPalletItemId, &BlockPalletItem)> {
        self.iter().find(|p| p.1.block == *block)
    }

    #[inline]
    pub fn find_item_mut(&mut self, block: &Block) -> Option<(BlockPalletItemId, &mut BlockPalletItem)> {
        self.iter_mut().find(|p| p.1.block == *block)
    }

    #[inline]
    pub fn max_key(&self) -> Option<BlockPalletItemId> {
        self.ids().last()
    }

    #[inline]
    pub fn values(&self) -> BlockPalletValues {
        BlockPalletValues { inner: self.items.iter() }
    }

    #[inline]
    pub fn values_mut(&mut self) -> BlockPalletValuesMut {
        BlockPalletValuesMut { inner: self.items.iter_mut() }
    }

    #[inline]
    pub fn ids(&self) -> BlockPalletIds {
        BlockPalletIds { inner: self.items.iter().enumerate() }
    }

    #[inline]
    pub fn iter(&self) -> BlockPalletIter {
        BlockPalletIter { inner: self.items.iter().enumerate() }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> BlockPalletIterMut {
        BlockPalletIterMut { inner: self.items.iter_mut().enumerate() }
    }
}

pub struct BlockPalletValues<'a> {
    inner: std::slice::Iter<'a, Option<BlockPalletItem>>
}

impl<'a> Iterator for BlockPalletValues<'a> {
    type Item = &'a BlockPalletItem;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.inner.next() {
            if next.is_some() {
                return next.as_ref();
            }
        }
        None
    }
}

pub struct BlockPalletValuesMut<'a> {
    inner: std::slice::IterMut<'a, Option<BlockPalletItem>>
}

impl<'a> Iterator for BlockPalletValuesMut<'a> {
    type Item = &'a mut BlockPalletItem;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.inner.next() {
            if next.is_some() {
                return next.as_mut();
            }
        }
        None
    }
}


pub struct BlockPalletIds<'a> {
    inner: std::iter::Enumerate<std::slice::Iter<'a, Option<BlockPalletItem>>>
}

impl<'a> Iterator for BlockPalletIds<'a> {
    type Item = BlockPalletItemId;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id, next)) = self.inner.next() {
            if next.is_some() {
                return Some(id as Self::Item);
            }
        }
        None
    }
}


pub struct BlockPalletIter<'a> {
    inner: std::iter::Enumerate<std::slice::Iter<'a, Option<BlockPalletItem>>>
}

impl<'a> Iterator for BlockPalletIter<'a> {
    type Item = (BlockPalletItemId, &'a BlockPalletItem);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id, next)) = self.inner.next() {
            if next.is_some() {
                return next.as_ref().map(|f| (id as BlockPalletItemId, f));
            }
        }
        None
    }
}

pub struct BlockPalletIterMut<'a> {
    inner: std::iter::Enumerate<std::slice::IterMut<'a, Option<BlockPalletItem>>>
}

impl<'a> Iterator for BlockPalletIterMut<'a> {
    type Item = (BlockPalletItemId, &'a mut BlockPalletItem);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id, next)) = self.inner.next() {
            if next.is_some() {
                return next.as_mut().map(|f| (id as BlockPalletItemId, f));
            }
        }
        None
    }
}