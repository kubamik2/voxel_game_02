#![feature(variant_count, float_next_up_down, downcast_unchecked)]
use block::{asset_loader::{BlockList, BlockMap}, model::{block_model_variant::BlockModelVariants, QuadRaw}, Block};
use cgmath::Vector3;
use hashbrown::HashMap;
use game::Game;
use world::structure::Structure;

mod game;
mod game_window;
mod settings;
mod camera;
mod global_vector;
mod world;
mod block;
mod texture;
mod collision;
mod interval;
mod render_thread;
mod thread_work_dispatcher;
mod gui;
mod layer;
mod event;
mod typemap;
mod keybinds;
mod utils;
mod shader;

lazy_static::lazy_static! {
    pub static ref BASE_MODELS: block::asset_loader::BaseCuboidBlockModels = block::asset_loader::load_models("./assets/models").unwrap();
    static ref _TEMP: (BlockMap, BlockList, BlockModelVariants, Vec<QuadRaw>) = block::asset_loader::load_blocks("./assets/blocks", &BASE_MODELS).unwrap();

    pub static ref BLOCK_MAP: BlockMap = _TEMP.0.clone();
    pub static ref BLOCK_LIST: BlockList = _TEMP.1.clone();
    pub static ref BLOCK_MODEL_VARIANTS: BlockModelVariants = _TEMP.2.clone();
    pub static ref QUADS: Vec<QuadRaw> = _TEMP.3.clone();
    // pub static ref OBSTRUCTS_LIGHT_CACHE: bitmaps::Bitmap<1024> = {
    //     let mut bitmap = bitmaps::Bitmap::new();

    //     for (i, info) in BLOCK_LIST.iter().enumerate() {
    //         bitmap.set(i, info.properties().light_obstruction);
    //     }
    //     bitmap
    // };

    pub static ref STRUCTURES: HashMap<String, Structure> = {
        let mut structures = HashMap::new();
        let oak_log: Block = BLOCK_MAP.get("oak_log").unwrap().clone().into();
        let oak_leaves: Block = BLOCK_MAP.get("oak_leaves").unwrap().clone().into();

        structures.insert("tree".to_string(), Structure {
            blocks: vec![
                (Vector3::new(0, 0, 0), oak_log.clone()),
                (Vector3::new(0, 1, 0), oak_log.clone()),
                (Vector3::new(0, 2, 0), oak_log.clone()),
                (Vector3::new(0, 3, 0), oak_leaves.clone()),
                (Vector3::new(1, 3, 0), oak_leaves.clone()),
                (Vector3::new(0, 3, 1), oak_leaves.clone()),
                (Vector3::new(1, 3, 1), oak_leaves.clone()),
                (Vector3::new(-1, 3, 0), oak_leaves.clone()),
                (Vector3::new(0, 3, -1), oak_leaves.clone()),
                (Vector3::new(-1, 3, -1), oak_leaves.clone()),
                (Vector3::new(1, 3, -1), oak_leaves.clone()),
                (Vector3::new(-1, 3, 1), oak_leaves.clone()),
                (Vector3::new(0, 4, 0), oak_leaves.clone()),
                (Vector3::new(1, 4, 0), oak_leaves.clone()),
                (Vector3::new(0, 4, 1), oak_leaves.clone()),
                (Vector3::new(1, 4, 1), oak_leaves.clone()),
                (Vector3::new(-1, 4, 0), oak_leaves.clone()),
                (Vector3::new(0, 4, -1), oak_leaves.clone()),
                (Vector3::new(-1, 4, -1), oak_leaves.clone()),
                (Vector3::new(1, 4, -1), oak_leaves.clone()),
                (Vector3::new(-1, 4, 1), oak_leaves.clone()),
                (Vector3::new(0, 5, 0), oak_leaves.clone()),
            ]
        });

        structures
    };
}

fn main() -> anyhow::Result<()> {
    Game::run("./settings.json")
}
