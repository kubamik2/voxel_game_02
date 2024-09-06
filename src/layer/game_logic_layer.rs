use std::sync::Arc;

use cgmath::{Vector2, Vector3};

use crate::{game::Game, camera::Camera, event::{EventReader, Events}, game_window::{KeyboardInputEvent, MouseInputEvent, MouseMoveEvent}, gui::DebugGui, interval::Interval, layer::Layer, settings::Settings, world::{chunk::{chunks3x3::Chunks3x3, chunk_part::CHUNK_SIZE_I32, dynamic_chunk_mesh::DynamicChunkMesh}, World}, BLOCK_MAP};

pub struct GameLogicLayer {
    world: World,
    interval_300hz: Interval,
    interval_60hz: Interval,
    interval_20hz: Interval,
    keyboard_input_reader: EventReader<KeyboardInputEvent>,
    mouse_input_reader: EventReader<MouseInputEvent>,
    mouse_move_reader: EventReader<MouseMoveEvent>,
}

impl Layer for GameLogicLayer {
    fn on_update(&mut self, events: &mut Events, game: &mut Game) {
        self.interval_300hz.tick(|| {
            self.world.chunk_manager.update(&game.device);
        });

        self.interval_20hz.tick(|| {
            self.world.chunk_manager.insert_chunks_around_player(Vector2::new(0, 0));
            self.world.player.modify_block(&mut self.world.chunk_manager, BLOCK_MAP.get("stone").unwrap().clone().into());
            while let Some(changed_block_position) = self.world.chunk_manager.changed_blocks.pop() {
                let mut inner_chunk_position = changed_block_position.local().map(|f| f as i32);
                inner_chunk_position.y += changed_block_position.chunk.y * CHUNK_SIZE_I32;

                let Some(chunk_part) = self.world.chunk_manager.chunk_map.get_mut_chunk_part(changed_block_position.chunk) else { continue; };
                
                let mut removed_light_emitters = std::mem::take(&mut chunk_part.removed_light_emitters);
                let mut added_light_emitters = std::mem::take(&mut chunk_part.added_light_emitters);

                let Some(mut chunks3x3) = Chunks3x3::new(&mut self.world.chunk_manager.chunk_map, changed_block_position.chunk.xz()) else { continue; };
                let now = std::time::Instant::now();
                chunks3x3.remove_block_light_at(inner_chunk_position);
                let removal_time = now.elapsed();
                let now = std::time::Instant::now();
                chunks3x3.propagate_block_light_at(inner_chunk_position);
                let propagation_time = now.elapsed();
                let now = std::time::Instant::now();
                chunks3x3.update_sky_light_level_at(inner_chunk_position);
                let sky_light_update_time = now.elapsed();
                println!("propagation_time: {:?}\nremoval_time: {:?}\nsky_light_update_time: {:?}\n=====\n", propagation_time, removal_time, sky_light_update_time);
                
                for chunk in chunks3x3.chunks {
                    self.world.chunk_manager.chunk_map.insert(chunk.position, chunk);
                }
            }
        });

        for event in self.keyboard_input_reader.read(events) {
            self.world.player.handle_keyboard_input(event.key_code, event.pressed);
        }

        for event in self.mouse_input_reader.read(events) {
            self.world.player.handle_mouse_input(event.button, event.pressed);
            self.world.player.modify_block(&mut self.world.chunk_manager, BLOCK_MAP.get("stone").unwrap().clone().into());
        }

        for event in self.mouse_move_reader.read(events) {
            self.world.player.handle_mouse_movement(event.delta.map(|f| f as f32));
        }
    }

    fn on_render(&mut self, events: &mut Events, game: &mut Game) {
        let dt = game.last_render_instant.elapsed();
        self.world.player.update(dt.as_secs_f32());
        let debug_gui = DebugGui::new(&self.world, dt);
        debug_gui.show(game.egui_winit_state.egui_ctx());
        self.world.chunk_renderer.view_projection.update_from_matrix(self.world.player.build_view_projection_matrix(game.aspect_ratio));
        self.world.chunk_renderer.render(&game.device, &game.queue, &mut self.world.chunk_manager, &mut game.render_thread);
    }
}

impl GameLogicLayer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration, events: &Events, settings: &Settings) -> anyhow::Result<Self> {
        Ok(Self {
            world: World::new(device, queue, surface_config, settings)?,
            interval_300hz: Interval::new_hz(300.0),
            interval_60hz: Interval::new_hz(60.0),
            interval_20hz: Interval::new_hz(20.0),
            keyboard_input_reader: EventReader::new(events),
            mouse_input_reader: EventReader::new(events),
            mouse_move_reader: EventReader::new(events),
        })
    }
}

#[derive(Clone)]
pub struct ChunkUpdateRenderMesh {
    pub meshes: Arc<[DynamicChunkMesh]>,
}
