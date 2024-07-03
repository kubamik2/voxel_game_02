use chunk::chunk_part::CHUNK_SIZE;

pub mod chunk;

pub const CHUNK_HEIGHT: usize = CHUNK_SIZE * CHUNK_PARTS_PER_REGION;
pub const CHUNK_PARTS_PER_REGION: usize = 12;

pub struct World {
    
}

impl World {
    pub fn new() {
        Self::validate();
    }

    fn validate() {
        assert!(CHUNK_SIZE > 0);
        assert!(CHUNK_PARTS_PER_REGION > 0);
        assert!(CHUNK_HEIGHT > 0);
    }
}