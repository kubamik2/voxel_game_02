use chunk::{chunk_manager::ChunkManager, chunk_part::CHUNK_SIZE};
use player::Player;

use crate::settings::Settings;

pub mod chunk;
pub mod structure;
pub mod player;

pub const CHUNK_HEIGHT: usize = CHUNK_SIZE * PARTS_PER_CHUNK;
pub const PARTS_PER_CHUNK: usize = 12;

pub struct World {
    pub chunk_manager: ChunkManager,
    pub player: Player,
}

impl World {
    pub fn new(settings: &Settings) -> Self {
        Self {
            chunk_manager: ChunkManager::new(settings.render_distance, 16, 12),
            player: Player::new(),
        }
    }
}
