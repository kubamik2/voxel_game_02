use chunk::chunk_part::CHUNK_SIZE;

pub mod chunk;
pub mod structure;

pub const CHUNK_HEIGHT: usize = CHUNK_SIZE * PARTS_PER_CHUNK;
pub const PARTS_PER_CHUNK: usize = 12;

pub struct World {
    
}

impl World {
    pub fn new() -> Self {
        Self::validate();
        todo!()
    }

    fn validate() {
        assert!(CHUNK_SIZE > 0);
        assert!(PARTS_PER_CHUNK > 0);
        assert!(CHUNK_HEIGHT > 0);
    }
}
