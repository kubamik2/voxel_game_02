use std::sync::Arc;

use cgmath::{Vector2, Vector3};

use crate::{application::Application, event::{EventReader, Events}, game_window::{KeyboardInputEvent, MouseInputEvent, MouseMoveEvent}, interval::Interval, layer::Layer, settings::Settings, world::{chunk::{area::Area, chunk_part::CHUNK_SIZE_I32, dynamic_chunk_mesh::DynamicChunkMesh}, World}, BLOCK_MAP};

pub struct GameLogicLayer {
    world: World,
    interval_300hz: Interval,
    interval_60hz: Interval,
    keyboard_input_reader: EventReader<KeyboardInputEvent>,
    mouse_input_reader: EventReader<MouseInputEvent>,
    mouse_move_reader: EventReader<MouseMoveEvent>,
}

impl Layer for GameLogicLayer {
    fn on_update(&mut self, events: &mut Events, application: &mut Application) {
        self.interval_300hz.tick(|| {
            self.world.chunk_manager.update(&application.device);
        });

        self.interval_60hz.tick(|| {
            self.world.chunk_manager.insert_chunks_around_player(Vector2::new(0, 0));
            self.world.player.modify_block(&mut self.world.chunk_manager, BLOCK_MAP.get("torch").unwrap().clone().into());
            while let Some(changed_block_position) = self.world.chunk_manager.changed_blocks.pop() {
                let mut afflicted_chunk_parts = hashbrown::HashSet::new();
                let Some(chunk_part) = self.world.chunk_manager.chunk_map.get_mut_chunk_part(changed_block_position.chunk) else { continue; };
                
                let mut removed_light_emitters = std::mem::take(&mut chunk_part.removed_light_emitters);
                let mut added_light_emitters = std::mem::take(&mut chunk_part.added_light_emitters);

                let Some(mut area) = Area::new(&mut self.world.chunk_manager.chunk_map, changed_block_position.chunk.xz()) else { continue; };
                area.update_sky_light_at(changed_block_position.local().map(|f| f as i32) + Vector3::new(0, changed_block_position.chunk.y * CHUNK_SIZE_I32, 0), &mut afflicted_chunk_parts);
                area.update_block_light_at(changed_block_position.local().map(|f| f as i32) + Vector3::new(0, changed_block_position.chunk.y * CHUNK_SIZE_I32, 0), &mut afflicted_chunk_parts);
                
                while let Some(light_local_position) = removed_light_emitters.pop() {
                    let area_position = light_local_position.map(|f| f as i32) + Vector3::new(0, changed_block_position.chunk.y * CHUNK_SIZE_I32, 0);
                    area.remove_block_light_at(area_position, &mut afflicted_chunk_parts);
                }

                while let Some(light_local_position) = added_light_emitters.pop() {
                    let area_position = light_local_position.map(|f| f as i32) + Vector3::new(0, changed_block_position.chunk.y * CHUNK_SIZE_I32, 0);
                    area.propagate_block_light_at(area_position, &mut afflicted_chunk_parts);
                }

                for chunk in area.chunks {
                    self.world.chunk_manager.chunk_map.insert_arc(chunk.position, chunk);
                }

                for (chunk_position, chunk_part_index) in afflicted_chunk_parts {
                    let Some(mesh) = self.world.chunk_manager.chunk_mesh_map.get_mut(chunk_position) else { continue; };
                    mesh.parts_need_meshing[chunk_part_index] = true;
                }
            }
        });

        for event in self.keyboard_input_reader.read(events) {
            self.world.player.handle_keyboard_input(event.key_code, event.pressed);
        }

        for event in self.mouse_input_reader.read(events) {
            self.world.player.handle_mouse_input(event.button, event.pressed);
            self.world.player.modify_block(&mut self.world.chunk_manager, BLOCK_MAP.get("torch").unwrap().clone().into());
        }

        for event in self.mouse_move_reader.read(events) {
            self.world.player.handle_mouse_movement(event.delta.map(|f| f as f32));
        }
    }

    fn on_render(&mut self, events: &mut Events, application: &mut Application) {
        self.world.chunk_manager.collect_meshing_outputs(&application.device, &application.queue);
        events.send(ChunkUpdateRenderMesh {
            meshes: self.world.chunk_manager.get_ready_meshes().into(),
        });
        self.world.player.update();
        application.render_thread.update_view_projection(&self.world.player, application.aspect_ratio);
    }
}

impl GameLogicLayer {
    pub fn new(events: &Events, settings: &Settings) -> Self {
        Self {
            world: World::new(settings),
            interval_300hz: Interval::new_hz(300.0),
            interval_60hz: Interval::new_hz(60.0),
            keyboard_input_reader: EventReader::new(events),
            mouse_input_reader: EventReader::new(events),
            mouse_move_reader: EventReader::new(events),
        }
    }
}

#[derive(Clone)]
pub struct ChunkUpdateRenderMesh {
    pub meshes: Arc<[DynamicChunkMesh]>,
}