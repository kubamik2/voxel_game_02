use cgmath::Vector3;

use crate::relative_vector::RelVec3;

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub struct LocalHitbox {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>
}

impl Default for LocalHitbox {
    fn default() -> Self {
        Self {
            start: Vector3::new(0.0, 0.0, 0.0),
            end: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

pub struct GlobalHitbox {
    pub start: RelVec3,
    pub end: RelVec3,
}