use crate::{event::EventQueue, layer::Layer};

use super::chunk_manager::ChunkManager;

pub struct ChunkLayer {
    chunk_manager: ChunkManager,

}

impl Layer for ChunkLayer {
    fn id(&self) -> u32 {
        0
    }

    fn on_attach(&mut self) {
        
    }
    
    fn on_detach(&mut self) {
        
    }

    fn on_update(&mut self, event_queue: &mut EventQueue) {
        
    }
}