use chunk::{chunk_manager::ChunkManager, chunk_part::CHUNK_SIZE, chunk_renderer::ChunkRenderer};
use player::Player;

use crate::settings::Settings;

pub mod chunk;
pub mod structure;
pub mod player;

pub const CHUNK_HEIGHT: usize = CHUNK_SIZE * PARTS_PER_CHUNK;
pub const PARTS_PER_CHUNK: usize = 12;

pub struct World {
    pub chunk_manager: ChunkManager,
    pub chunk_renderer: ChunkRenderer,
    pub player: Player,
}

impl World {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration, settings: &Settings) -> anyhow::Result<Self> {
        let chunk_renderer = ChunkRenderer::new(device, queue, surface_config)?;

        Ok(Self {
            chunk_manager: ChunkManager::new(settings.render_distance, 16, 12),
            chunk_renderer,
            player: Player::new(),
        })
    }
}
