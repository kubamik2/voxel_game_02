use std::sync::Arc;

use cgmath::{Vector2, Vector3};

use crate::{camera::Camera, event::{EventManager, EventReader}, game::Game, game_window::{KeyboardInputEvent, MouseInputEvent, MouseMoveEvent}, global_vector::GlobalVecU, gui::DebugGui, interval::Interval, layer::Layer, settings::Settings, world::{chunk::{chunk_part::CHUNK_SIZE_I32, chunks3x3::Chunks3x3, dynamic_chunk_mesh::DynamicChunkMesh}, region::Region, World, PARTS_PER_CHUNK}, BLOCK_MAP, GLOBAL_RESOURCES};

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
    fn on_update(&mut self, event_manager: &EventManager, game: &mut Game) {
        self.interval_300hz.tick(|| {
            self.world.chunk_manager.update(&game.device);
        });
        

        self.interval_20hz.tick(|| {
            let now = std::time::Instant::now();
            self.world.chunk_manager.insert_chunks_around_player(self.world.player.position.chunk.xz());
            self.world.player.modify_block(&mut self.world.chunk_manager, BLOCK_MAP.get("torch").unwrap().clone().into());
            while let Some(changed_block_position) = self.world.chunk_manager.changed_blocks.pop() {
                let mut inner_chunk_position = changed_block_position.local().map(|f| f as i32);
                inner_chunk_position.y += changed_block_position.chunk.y * CHUNK_SIZE_I32;

                let Some(mut chunks3x3) = Chunks3x3::new(&mut self.world.chunk_manager.chunk_map_lock.write(), changed_block_position.chunk.xz()) else { continue; };
                chunks3x3.remove_block_light_at(inner_chunk_position);
                chunks3x3.propagate_block_light_at(inner_chunk_position);
                chunks3x3.update_sky_light_level_at(inner_chunk_position);
                
                chunks3x3.return_to_chunk_map(&mut self.world.chunk_manager.chunk_map_lock);

                if let Some(direction) = changed_block_position.touching_sides() {
                    if let Some(mesh) = self.world.chunk_manager.chunk_mesh_map.get_mut(changed_block_position.chunk.xz() + Vector2::new(direction.x, 0)) {
                        mesh.parts_need_meshing[changed_block_position.chunk.y as usize] = true;
                    }

                    if let Some(mesh) = self.world.chunk_manager.chunk_mesh_map.get_mut(changed_block_position.chunk.xz() + Vector2::new(0, direction.z)) {
                        mesh.parts_need_meshing[changed_block_position.chunk.y as usize] = true;
                    }

                    if let Some(mesh) = self.world.chunk_manager.chunk_mesh_map.get_mut(changed_block_position.chunk.xz()) {
                        let chunk_part_index = changed_block_position.chunk.y + direction.y;
                        if chunk_part_index >= 0 && chunk_part_index < PARTS_PER_CHUNK as i32 {
                            mesh.parts_need_meshing[chunk_part_index as usize] = true;
                        }
                    }

                    if let Some(mesh) = self.world.chunk_manager.chunk_mesh_map.get_mut(changed_block_position.chunk.xz() + Vector2::new(direction.x, direction.z)) {
                        let chunk_part_index = changed_block_position.chunk.y + direction.y;
                        if chunk_part_index >= 0 && chunk_part_index < PARTS_PER_CHUNK as i32 {
                            mesh.parts_need_meshing[chunk_part_index as usize] = true;
                        }
                    }
                }
            }

            // if self.world.player.is_r_pressed {
            //     let mut region = Region::new(Vector2::new(0, 0));

            //     for z in -8..8 {
            //         for x in -8..8 {
            //             let position = Vector2::new(x, z);

            //             let mut chunk = self.world.chunk_manager.chunk_map.get_chunk(position).cloned().unwrap();
            //             chunk.maintain_parts();
            //             region.chunks.insert(chunk.position, chunk);
            //         }
            //     }
            //     region.save("./save/").unwrap();
            //     let now = std::time::Instant::now();
            //     let region = Region::load("./save/", Vector2::new(0, 0)).unwrap();
            //     dbg!(now.elapsed());
            // }
            for chunk in self.world.chunk_manager.chunk_map_lock.write().iter_mut_chunks() {
                let Some(mesh) = self.world.chunk_manager.chunk_mesh_map.get_mut(chunk.position) else { continue; };
                let chunk = Arc::get_mut(chunk).unwrap();
                for (i, part) in chunk.parts.iter_mut().enumerate() {
                    mesh.parts_need_meshing[i] |= part.was_modified;
                    part.was_modified = false;
                }
            }

            game.last_update_time = now.elapsed();
        });

        for event in self.keyboard_input_reader.read() {
            self.world.player.handle_keyboard_input(event.key_code, event.pressed);
        }

        for event in self.mouse_input_reader.read() {
            self.world.player.handle_mouse_input(event.button, event.pressed);
            self.world.player.modify_block(&mut self.world.chunk_manager, BLOCK_MAP.get("torch").unwrap().clone().into());
        }

        for event in self.mouse_move_reader.read() {
            self.world.player.handle_mouse_movement(event.delta.map(|f| f as f32));
        }
    }

    fn on_render(&mut self, events: &EventManager, game: &mut Game) {
        let dt = game.last_render_instant.elapsed();
        self.world.player.update(dt.as_secs_f32());
        let debug_gui = DebugGui::new(&self.world, dt, game.last_update_time);
        debug_gui.show(game.egui_winit_state.egui_ctx());
        self.world.chunk_renderer.view_projection.update_from_matrix(self.world.player.build_view_projection_matrix(game.aspect_ratio));
        self.world.chunk_renderer.render(&game.device, &game.queue, &mut self.world.chunk_manager, &mut game.render_thread);
    }
}

impl GameLogicLayer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_config: &wgpu::SurfaceConfiguration, settings: &Settings) -> anyhow::Result<Self> {
        let event_manager = (*GLOBAL_RESOURCES).get::<EventManager>().unwrap();
        Ok(Self {
            world: World::new(device, queue, surface_config, settings)?,
            interval_300hz: Interval::new_hz(300.0),
            interval_60hz: Interval::new_hz(60.0),
            interval_20hz: Interval::new_hz(20.0),
            keyboard_input_reader: event_manager.create_reader(),
            mouse_input_reader: event_manager.create_reader(),
            mouse_move_reader: event_manager.create_reader(),
        })
    }
}

#[derive(Clone)]
pub struct ChunkUpdateRenderMesh {
    pub meshes: Arc<[DynamicChunkMesh]>,
}
