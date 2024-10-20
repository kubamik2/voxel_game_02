use std::sync::Arc;

use crate::{event::{EventManager, EventReader}, game::Game, layer::Layer, render_thread::RenderArgs, world::chunk::dynamic_chunk_mesh::DynamicChunkMesh, GLOBAL_RESOURCES};

use super::game_logic_layer::ChunkUpdateRenderMesh;

pub struct ChunkRenderingLayer {
    meshes: Arc<[DynamicChunkMesh]>,
    chunk_update_mesh_reader: EventReader<ChunkUpdateRenderMesh>,
}

impl Layer for ChunkRenderingLayer {
    fn on_render(&mut self, events: &crate::event::EventManager, game: &mut Game) {
        for event in self.chunk_update_mesh_reader.read().cloned() {
            let meshes = event.meshes;
            self.meshes = meshes;
        }
        game.egui_full_output = game.egui_winit_state.egui_ctx().end_frame();
        game.render_thread.execute_queued_renders(RenderArgs {
            egui_context: game.egui_winit_state.egui_ctx().clone(),
            egui_full_output: game.egui_full_output.clone(),
            surface: game.surface.clone(),
            surface_config: game.surface_config.clone(),
            window: game.game_window.window_arc(),
        });
        game.last_render_instant = std::time::Instant::now();
    }
}

impl ChunkRenderingLayer {
    pub fn new() -> Self {
        let event_manager = (*GLOBAL_RESOURCES).get::<EventManager>().unwrap();
        Self {
            meshes: Arc::new([]),
            chunk_update_mesh_reader: EventReader::new(event_manager),
        }
    }
}
