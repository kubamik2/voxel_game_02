use std::sync::Arc;

use crate::{application::Application, event::{EventReader, Events}, layer::Layer, render_thread::RenderArgs, world::chunk::dynamic_chunk_mesh::DynamicChunkMesh};

use super::game_logic_layer::ChunkUpdateRenderMesh;

pub struct ChunkRenderingLayer {
    meshes: Arc<[DynamicChunkMesh]>,
    chunk_update_mesh_reader: EventReader<ChunkUpdateRenderMesh>,
}

impl Layer for ChunkRenderingLayer {
    fn on_render(&mut self, events: &mut crate::event::Events, application: &mut Application) {
        for event in self.chunk_update_mesh_reader.read(&events).cloned() {
            let meshes = event.meshes;
            self.meshes = meshes;
        }

        let render_args = RenderArgs {
            meshes: self.meshes.clone(),
            surface: application.surface.clone(),
            surface_config: application.surface_config.clone(),
            window: application.game_window.window_arc(),
        };
        application.render_thread.render(render_args);
    }
}

impl ChunkRenderingLayer {
    pub fn new(events: &Events) -> Self {
        Self {
            meshes: Arc::new([]),
            chunk_update_mesh_reader: EventReader::new(events),
        }
    }
}