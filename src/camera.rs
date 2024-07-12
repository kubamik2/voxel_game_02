// TODO change file name to better reflect it's contents
use cgmath::{Deg, InnerSpace, Matrix4, Point3, Rad, Vector3, Vector4};
use winit::{event::ElementState, keyboard::KeyCode};

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

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ViewProjectionUniform {
    view_projection: [[f32; 4]; 4]
}

impl ViewProjectionUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self { view_projection: Matrix4::identity().into() }
    }

    pub fn update(&mut self, camera: &dyn Camera, aspect: f32) {
        self.view_projection = camera.build_view_projection_matrix(aspect).into();
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
    }
}

pub struct CameraTemp {
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub yaw: Deg<f32>,
    pub pitch: Deg<f32>,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
}
const PITCH_LIMIT: f32 = 90.0 - 0.0001;
impl CameraTemp {
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
            position: Point3::new(0.0, 0.0, -1.0),
            direction,
            yaw,
            pitch,
            is_backward_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
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

    pub fn update(&mut self) {
        let forward = Vector3::new(self.direction.x, 0.0, self.direction.z).normalize();
        let right = forward.cross(Vector3::unit_y());

        let mut horizontal_movement_vector: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        let mut vertical_movement_vector: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        let speed = 0.4;

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
            self.position += vertical_movement_vector * speed;
        }
    }

}

impl Camera for CameraTemp {
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
        self.position
    }

    fn fovy(&self) -> Deg<f32> {
        Deg(50.0)
    }
}