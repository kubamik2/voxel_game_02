#![feature(variant_count, float_next_up_down)]
use block::{asset_loader::{BlockList, BlockMap}, model::{BlockModelVariants, QuadRaw}, Block};
use cgmath::Vector3;
use hashbrown::HashMap;
use state::State;
use world::structure::Structure;
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
mod gui;

lazy_static::lazy_static! {
    pub static ref BASE_MODELS: block::asset_loader::BaseQuadBlockModels = block::asset_loader::load_models("./assets/models").unwrap();
    static ref _TEMP: (BlockMap, BlockList, BlockModelVariants, Vec<QuadRaw>) = block::asset_loader::load_blocks("./assets/blocks", &BASE_MODELS).unwrap();

    pub static ref BLOCK_MAP: BlockMap = _TEMP.0.clone();
    pub static ref BLOCK_LIST: BlockList = _TEMP.1.clone();
    pub static ref BLOCK_MODEL_VARIANTS: BlockModelVariants = _TEMP.2.clone();
    pub static ref QUADS: Vec<QuadRaw> = _TEMP.3.clone();
    pub static ref OBSTRUCTS_LIGHT_CACHE: bitmaps::Bitmap<1024> = {
        // let mut v = vec![];
        let mut bitmap = bitmaps::Bitmap::new();

        for (i, info) in BLOCK_LIST.iter().enumerate() {
            bitmap.set(i, info.properties().obstructs_light);
            // v.push(info.properties().obstructs_light);
        }
        // v.into_boxed_slice()
        bitmap
    };

    pub static ref STRUCTURES: HashMap<String, Structure> = {
        let mut structures = HashMap::new();
        let cobblestone: Block = BLOCK_MAP.get("cobblestone").unwrap().clone().into();

        structures.insert("tree".to_string(), Structure {
            blocks: vec![
                (Vector3::new(0, 0, 0), cobblestone.clone()),
                (Vector3::new(0, 1, 0), cobblestone.clone()),
                (Vector3::new(0, 2, 0), cobblestone.clone()),
                (Vector3::new(0, 3, 0), cobblestone.clone()),
                (Vector3::new(1, 3, 0), cobblestone.clone()),
                (Vector3::new(0, 3, 1), cobblestone.clone()),
                (Vector3::new(1, 3, 1), cobblestone.clone()),
                (Vector3::new(-1, 3, 0), cobblestone.clone()),
                (Vector3::new(0, 3, -1), cobblestone.clone()),
                (Vector3::new(-1, 3, -1), cobblestone.clone()),
                (Vector3::new(1, 3, -1), cobblestone.clone()),
                (Vector3::new(-1, 3, 1), cobblestone.clone()),
                (Vector3::new(0, 4, 0), cobblestone.clone()),
                (Vector3::new(1, 4, 0), cobblestone.clone()),
                (Vector3::new(0, 4, 1), cobblestone.clone()),
                (Vector3::new(1, 4, 1), cobblestone.clone()),
                (Vector3::new(-1, 4, 0), cobblestone.clone()),
                (Vector3::new(0, 4, -1), cobblestone.clone()),
                (Vector3::new(-1, 4, -1), cobblestone.clone()),
                (Vector3::new(1, 4, -1), cobblestone.clone()),
                (Vector3::new(-1, 4, 1), cobblestone.clone()),
                (Vector3::new(0, 5, 0), cobblestone.clone()),
            ]
        });

        structures
    };
}

fn main() -> anyhow::Result<()> {
    State::run("./settings.json")
}
