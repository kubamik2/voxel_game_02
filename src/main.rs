#![feature(variant_count, float_next_up_down, downcast_unchecked, new_zeroed_alloc, portable_simd, trait_alias, mapped_lock_guards)]
use std::ops::Deref;

use block::{asset_loader::{BlockList, BlockMap}, model::{block_model_variant::BlockModelVariants, QuadRaw}, Block, BlockId};
use cgmath::Vector3;
use event::{EventManager, EventManagerBuilder};
use global_resources::{GlobalResources, GlobalResourcesBuilder};
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
mod chunk_position;
mod global_resources;

lazy_static::lazy_static! {
    pub static ref BASE_MODELS: block::asset_loader::BaseCuboidBlockModels = block::asset_loader::load_models("./assets/models").unwrap();
    static ref _TEMP: (BlockMap, BlockList, BlockModelVariants, Vec<QuadRaw>) = block::asset_loader::load_blocks("./assets/blocks", &BASE_MODELS).unwrap();

    pub static ref BLOCK_MAP: BlockMap = _TEMP.0.clone();
    pub static ref BLOCK_LIST: BlockList = _TEMP.1.clone();
    pub static ref BLOCK_MODEL_VARIANTS: BlockModelVariants = _TEMP.2.clone();
    pub static ref QUADS: Vec<QuadRaw> = _TEMP.3.clone();
    pub static ref AIR_ID: BlockId = *BLOCK_MAP.get("air").unwrap().id();

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

pub static GLOBAL_RESOURCES: std::sync::LazyLock<GlobalResources> = std::sync::LazyLock::new(|| 
    GlobalResourcesBuilder::default()
        .register_resource(
            EventManagerBuilder::default()
                .build()
        )
        .build()
);

fn main() -> anyhow::Result<()> {
    Game::run("./settings.json")
}
