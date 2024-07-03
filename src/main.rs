#![feature(variant_count)]
use block::{asset_loader::{BlockList, BlockMap}, model::{BlockModelVariants, QuadRaw}};
use state::State;
use std::sync::{Arc, Mutex};

mod state;
mod game_window;
mod setttings;
mod camera;
mod relative_vector;
mod world;
mod block;
mod texture;
mod collision;

lazy_static::lazy_static! {
    pub static ref BASE_MODELS: block::asset_loader::BaseQuadBlockModels = block::asset_loader::load_models("./assets/models").unwrap();
    static ref _TEMP: (BlockMap, BlockList, BlockModelVariants, Vec<QuadRaw>) = block::asset_loader::load_blocks("./assets/blocks", &BASE_MODELS).unwrap();

    pub static ref BLOCK_MAP: Arc<Mutex<BlockMap>> = Arc::new(Mutex::new(_TEMP.0.clone()));
    pub static ref BLOCK_LIST: Arc<Mutex<BlockList>> = Arc::new(Mutex::new(_TEMP.1.clone()));
    pub static ref BLOCK_MODEL_VARIANTS: Arc<Mutex<BlockModelVariants>> = Arc::new(Mutex::new(_TEMP.2.clone()));
    pub static ref QUADS: Arc<Mutex<Vec<QuadRaw>>> = Arc::new(Mutex::new(_TEMP.3.clone()));
}

fn main() -> anyhow::Result<()> {
    State::run("./settings.json")
}