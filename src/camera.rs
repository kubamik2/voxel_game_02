// TODO change file name to better reflect it's contents
use cgmath::{Deg, InnerSpace, Matrix4, Point3, Rad, Vector3, Vector4};
use wgpu::util::DeviceExt;
use winit::{event::{ElementState, MouseButton}, keyboard::KeyCode};

use crate::{block::Block, collision::bounding_box::{GlobalBoundingBox, Ray}, relative_vector::RelVec3, world::{chunk::{chunk_manager::ChunkManager, chunk_map::ChunkMap, chunk_mesh_map::ChunkMeshMap}, PARTS_PER_CHUNK}, BLOCK_LIST, BLOCK_MAP, BLOCK_MODEL_VARIANTS};

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);

pub trait Camera {
    fn z_near(&self) -> f32;
    fn z_far(&self) -> f32;
    fn camera_position(&self) -> Point3<f32>;
    fn camera_direction(&self) -> Vector3<f32>;
    fn fovy(&self) -> Deg<f32>;
    fn build_view_projection_matrix(&self, aspect: f32) -> Matrix4<f32> {
        let view = Matrix4::look_to_rh(self.camera_position(), self.camera_direction(), UP);
        let projection = cgmath::perspective(self.fovy(), aspect, self.z_near(), self.z_far());
        
        OPENGL_TO_WGPU_MATRIX * projection * view
    }
}

static VIEW_PROJECTION_BIND_GROUP_LAYOUT: std::sync::OnceLock<wgpu::BindGroupLayout> = std::sync::OnceLock::new();

pub struct ViewProjection {
    view_projection: [[f32; 4]; 4],
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl ViewProjection {
    pub fn new(device: &wgpu::Device) -> Self {
        use cgmath::SquareMatrix;
        let view_projection: [[f32; 4]; 4] = Matrix4::identity().into();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ViewProjection_buffer"),
            contents: bytemuck::cast_slice(&view_projection),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });
        let bind_group_layout = Self::get_or_init_bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ViewProjection_bind_group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding())
                }
            ],
            layout: bind_group_layout
        });
        Self { view_projection, buffer, bind_group }
    }

    pub fn update(&mut self, camera: &dyn Camera, aspect: f32) {
        self.view_projection = camera.build_view_projection_matrix(aspect).into();
    }

    pub fn update_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.view_projection));
    }

    pub fn update_from_matrix(&mut self, matrix: Matrix4<f32>) {
        self.view_projection = matrix.into();
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn get_or_init_bind_group_layout(device: &wgpu::Device) -> &wgpu::BindGroupLayout {
        VIEW_PROJECTION_BIND_GROUP_LAYOUT.get_or_init(||
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ViewProjectionUniform_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        ty: wgpu::BindingType::Buffer {
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            ty: wgpu::BufferBindingType::Uniform
                        },
                        visibility: wgpu::ShaderStages::VERTEX,
                        count: None
                    }
                ]
            })
        )
    }
}

pub struct Player {
    pub position: RelVec3,
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
            position: RelVec3::from(Vector3::new(0.0, 200.0, 0.0)),
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

    pub fn handle_mouse_movement(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw += Deg(delta_x / 8.0).into();
        self.pitch -= Deg(delta_y / 8.0).into();
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

    pub fn handle_keyboard_input(&mut self, key_code: KeyCode, state: ElementState) {
        let pressed = state.is_pressed();
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

    pub fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        let pressed = state.is_pressed();
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
            let Some(block) = chunk_manager.chunk_map.get_block(voxel_pos) else { continue; };
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
            chunk_manager.chunk_map.set_block(voxel_pos, air);
        } else if self.is_right_mouse_pressed {
            voxel_pos = voxel_pos + face.normal();
            {
                let Some(block) = chunk_manager.chunk_map.get_block(voxel_pos) else { return; };
                if !BLOCK_LIST.get(*block.id()).unwrap().properties().replaceable { return; }
            }
            chunk_manager.chunk_map.set_block(voxel_pos, block);
        } else {
            return;
        }
        
        chunk_manager.changed_blocks.push(voxel_pos);
        
        let Some(mesh) = chunk_manager.chunk_mesh_map.get_mut(voxel_pos.chunk_pos.xz()) else { return; };
        mesh.parts_need_meshing[voxel_pos.chunk_pos.y as usize] = true;

        if voxel_pos.chunk_pos.y < PARTS_PER_CHUNK as i32 - 1 {
            mesh.parts_need_meshing[voxel_pos.chunk_pos.y as usize + 1] = true;
        }

        if voxel_pos.chunk_pos.y > 0 {
            mesh.parts_need_meshing[voxel_pos.chunk_pos.y as usize - 1] = true;
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