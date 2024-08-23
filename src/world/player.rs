use cgmath::{Deg, InnerSpace, Point3, Rad, Vector2, Vector3};
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{block::Block, camera::Camera, collision::bounding_box::{GlobalBoundingBox, Ray}, global_vector::{GlobalVecF, GlobalVecU}, world::{chunk::chunk_part::CHUNK_SIZE, PARTS_PER_CHUNK}, BLOCK_LIST, BLOCK_MAP, BLOCK_MODEL_VARIANTS};

use super::chunk::chunk_manager::ChunkManager;

pub struct Player {
    pub position: GlobalVecF,
    pub direction: Vector3<f32>,
    pub yaw: Deg<f32>,
    pub pitch: Deg<f32>,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub is_left_mouse_pressed: bool,
    pub is_right_mouse_pressed: bool,
    pub last_block_modification: std::time::Instant,
}
const PITCH_LIMIT: f32 = 90.0 - 0.0001;
impl Player {
    pub fn new() -> Self {
        let yaw = Deg(90.0_f32);
        let pitch = Deg(0.0_f32);
        
        let (sin_pitch, cos_pitch) = Rad::from(pitch).0.sin_cos();
        let (sin_yaw, cos_yaw) =  Rad::from(yaw).0.sin_cos();

        let direction = Vector3::new(
            cos_pitch * cos_yaw,
            sin_pitch,
            cos_pitch * sin_yaw
        ).normalize();

        Self {
            position: GlobalVecF::from(Vector3::new(0.0, 200.0, 0.0)),
            direction,
            yaw,
            pitch,
            is_backward_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_left_mouse_pressed: false,
            is_right_mouse_pressed: false,
            last_block_modification: std::time::Instant::now(),
        }
    }

    pub fn handle_mouse_movement(&mut self, delta: Vector2<f32>) {
        self.yaw += Deg(delta.x / 8.0).into();
        self.pitch -= Deg(delta.y / 8.0).into();
        self.pitch.0 = self.pitch.0.clamp(-PITCH_LIMIT, PITCH_LIMIT);
        
        let (sin_pitch, cos_pitch) = Rad::from(self.pitch).0.sin_cos();
        let (sin_yaw, cos_yaw) =  Rad::from(self.yaw).0.sin_cos();

        let direction = Vector3::new(
            cos_pitch * cos_yaw,
            sin_pitch,
            cos_pitch * sin_yaw
        ).normalize();

        self.direction = direction;
    }

    pub fn handle_keyboard_input(&mut self, key_code: KeyCode, pressed: bool) {
        match key_code {
            KeyCode::KeyW => { self.is_forward_pressed = pressed },
            KeyCode::KeyS => { self.is_backward_pressed = pressed },
            KeyCode::KeyA => { self.is_left_pressed = pressed },
            KeyCode::KeyD => { self.is_right_pressed = pressed },
            KeyCode::Space => { self.is_up_pressed = pressed },
            KeyCode::ShiftLeft => { self.is_down_pressed = pressed },
            _ => ()
        }
    }

    pub fn handle_mouse_input(&mut self, button: MouseButton, pressed: bool) {
        match button {
            MouseButton::Left => {
                self.is_left_mouse_pressed = pressed;
                if pressed {
                    self.last_block_modification = std::time::Instant::now();
                }
            },
            MouseButton::Right => {
                self.is_right_mouse_pressed = pressed;
                if pressed {
                    self.last_block_modification = std::time::Instant::now();
                }
            },
            _ => ()
        }
    }

    pub fn update(&mut self) {
        let forward = Vector3::new(self.direction.x, 0.0, self.direction.z).normalize();
        let right = forward.cross(Vector3::unit_y());

        let mut horizontal_movement_vector: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        let mut vertical_movement_vector: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        let speed = 0.04;

        if self.is_forward_pressed {
            horizontal_movement_vector += forward;
        }

        if self.is_backward_pressed {
            horizontal_movement_vector -= forward;
        }

        if self.is_right_pressed {
            horizontal_movement_vector += right;
        }

        if self.is_left_pressed {
            horizontal_movement_vector -= right;
        }


        if self.is_up_pressed {
            vertical_movement_vector += Vector3::unit_y() * speed;
        }

        if self.is_down_pressed {
            vertical_movement_vector -= Vector3::unit_y() * speed;
        }


        if horizontal_movement_vector.magnitude2() > 0.0 {
            horizontal_movement_vector = horizontal_movement_vector.normalize();
            self.position += horizontal_movement_vector * speed;
        }

        if vertical_movement_vector.magnitude2() > 0.0 {
            self.position += vertical_movement_vector * speed * 10.0;
        }
    }

    pub fn modify_block(&mut self, chunk_manager: &mut ChunkManager, block: Block) {
        if self.last_block_modification.elapsed().as_nanos() == 0 { return; }
        self.last_block_modification = std::time::Instant::now() + std::time::Duration::from_millis(200);
        
        let mut collided_face = None;
        let mut collision_pos = None;
        
        for voxel_pos in self.position.interpolate_voxels(self.direction, 5.0) {
            let Some(block) = chunk_manager.chunk_map.get_block_global(voxel_pos) else { continue; };
            let block_info = BLOCK_LIST.get(*block.id()).unwrap();
            if !block_info.properties().targetable { continue; }
            let Some(variants) = BLOCK_MODEL_VARIANTS.get_model_variants(block) else { continue; };

            let ray = Ray::new(self.position, self.direction, 5.0);
            let mut nearest_collision = None;
            for variant in variants {
                for bounding_box in variant.hitboxes.iter() {
                    let global_bounding_box = GlobalBoundingBox {
                        start: voxel_pos + bounding_box.start,
                        end: voxel_pos + bounding_box.end,
                    };

                    if let Some((face, t)) = global_bounding_box.ray_intersection_block_face_time(&ray) {
                        match nearest_collision {
                            Some((face, min_t)) => {
                                if t < min_t {
                                    nearest_collision = Some((face, min_t));
                                }
                            },
                            None => {
                                nearest_collision = Some((face, t));
                            }
                        }
                    }
                }
            }
            if let Some((face, _)) = nearest_collision {
                collided_face = Some(face);
                collision_pos = Some(voxel_pos);
                break;
            }
        }

        let Some(face) = collided_face else { return; };
        let mut voxel_pos = collision_pos.unwrap();

        let air = BLOCK_MAP.get("air").unwrap().clone().into();
        if self.is_left_mouse_pressed {
            chunk_manager.chunk_map.set_block_global(voxel_pos, air);
        } else if self.is_right_mouse_pressed {
            voxel_pos = voxel_pos + face.normal_i32();
            {
                let Some(block) = chunk_manager.chunk_map.get_block_global(voxel_pos) else { return; };
                if !BLOCK_LIST.get(*block.id()).unwrap().properties().replaceable { return; }
            }

            chunk_manager.chunk_map.set_block_global(voxel_pos, block);
        } else {
            return;
        }
        
        chunk_manager.changed_blocks.push(voxel_pos);
        
        #[inline]
        fn mark_chunk_part_for_meshing(chunk_manager: &mut ChunkManager, offset: Vector2<i32>, voxel_pos: GlobalVecU) {
            if let Some(mesh) = chunk_manager.chunk_mesh_map.get_mut(voxel_pos.chunk.xz() + offset) {
                mesh.parts_need_meshing[voxel_pos.chunk.y as usize] = true;
                if voxel_pos.chunk.y < PARTS_PER_CHUNK as i32 - 1 && voxel_pos.local().y == CHUNK_SIZE - 1 {
                    mesh.parts_need_meshing[voxel_pos.chunk.y as usize + 1] = true;
                }

                if voxel_pos.chunk.y > 0 && voxel_pos.local().y == 0 {
                    mesh.parts_need_meshing[voxel_pos.chunk.y as usize - 1] = true;
                }
            }
        }

        if voxel_pos.local().x == 0 {
            mark_chunk_part_for_meshing(chunk_manager, Vector2::new(-1, 0), voxel_pos);
            if voxel_pos.local().z == 0 {
                mark_chunk_part_for_meshing(chunk_manager, Vector2::new(-1, -1), voxel_pos);
            } else if voxel_pos.local().z == CHUNK_SIZE - 1 {
                mark_chunk_part_for_meshing(chunk_manager, Vector2::new(-1, 1), voxel_pos);
            }
        } else if voxel_pos.local().x == CHUNK_SIZE - 1 {
            mark_chunk_part_for_meshing(chunk_manager, Vector2::new(1, 0), voxel_pos);
            if voxel_pos.local().z == 0 {
                mark_chunk_part_for_meshing(chunk_manager, Vector2::new(1, -1), voxel_pos);
            } else if voxel_pos.local().z == CHUNK_SIZE - 1 {
                mark_chunk_part_for_meshing(chunk_manager, Vector2::new(1, 1), voxel_pos);
            }
        }

        if voxel_pos.local().z == 0 {
            mark_chunk_part_for_meshing(chunk_manager, Vector2::new(0, -1), voxel_pos);
        } else if voxel_pos.local().z == CHUNK_SIZE - 1 {
            mark_chunk_part_for_meshing(chunk_manager, Vector2::new(0, 1), voxel_pos);
        }
    }
}

impl Camera for Player {
    fn z_near(&self) -> f32 {
        0.1   
    }

    fn z_far(&self) -> f32 {
        100.0
    }

    fn camera_direction(&self) -> Vector3<f32> {
        self.direction
    }

    fn camera_position(&self) -> Point3<f32> {
        let vector: Vector3<f32> = self.position.into();
        Point3 { x: vector.x, y: vector.y, z: vector.z }
    }

    fn fovy(&self) -> Deg<f32> {
        Deg(50.0)
    }
}