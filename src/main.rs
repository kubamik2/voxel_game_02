#![feature(variant_count)]
use block::{asset_loader::{BlockList, BlockMap}, model::{BlockModelVariants, QuadRaw}};
use state::State;
use std::sync::{Arc, Mutex};

mod state;
mod game_window;
mod settings;
mod camera;
mod relative_vector;
mod world;
mod block;
mod texture;
mod collision;
mod interval;
mod render_thread;
mod renderable;
mod thread_work_dispatcher;

lazy_static::lazy_static! {
    pub static ref BASE_MODELS: block::asset_loader::BaseQuadBlockModels = block::asset_loader::load_models("./assets/models").unwrap();
    static ref _TEMP: (BlockMap, BlockList, BlockModelVariants, Vec<QuadRaw>) = block::asset_loader::load_blocks("./assets/blocks", &BASE_MODELS).unwrap();

    pub static ref BLOCK_MAP: BlockMap = _TEMP.0.clone();
    pub static ref BLOCK_LIST: BlockList = _TEMP.1.clone();
    pub static ref BLOCK_MODEL_VARIANTS: BlockModelVariants = _TEMP.2.clone();
    pub static ref QUADS: Vec<QuadRaw> = _TEMP.3.clone();
    pub static ref OBSTRUCTS_LIGHT_CACHE: Box<[bool]> = {
        let mut v = vec![];
        for info in BLOCK_LIST.iter() {
            v.push(info.properties().obstructs_light);
        }
        v.into_boxed_slice()
    };
}

fn main() -> anyhow::Result<()> {
    State::run("./settings.json")
}
