use cgmath::{Vector2, Vector3};

use crate::block::Block;

pub struct Structure {
    pub blocks: Vec<(Vector3<i32>, Block)>,
}

impl Structure {
    pub fn blocks(&self) -> &[(Vector3<i32>, Block)] {
        &self.blocks
    }
}