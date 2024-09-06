use cgmath::{Deg, Rotation, Vector3};

use crate::block::FACE_DIRECTIONS_NUM;

use super::Quad;

#[derive(Debug, Clone)]
pub struct QuadBlockModel {
    pub quads_per_face: [Box<[Quad]>; FACE_DIRECTIONS_NUM],
    pub texture_indices_per_face: [Box<[u16]>; FACE_DIRECTIONS_NUM],
    pub quad_culling_per_face: [Box<[bool]>; FACE_DIRECTIONS_NUM],
}

impl QuadBlockModel {
    pub fn rotate(&mut self, rotation: Vector3<f32>) {
        let euler_angles = cgmath::Euler::new(Deg(rotation.x), Deg(rotation.y), Deg(rotation.z));
        let rotation_matrix = cgmath::Basis3::from(euler_angles);
        for quads in self.quads_per_face.iter_mut() {
            for quad in quads {
                for position in quad.vertex_positions.iter_mut() {
                    *position = rotation_matrix.rotate_vector(*position);
                }
                quad.normal = rotation_matrix.rotate_vector(quad.normal);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuadIndexBlockModel {
    pub quad_indices_per_face: [Box<[u16]>; FACE_DIRECTIONS_NUM],
    pub texture_indices_per_face: [Box<[u16]>; FACE_DIRECTIONS_NUM],
    pub quad_culling_per_face: [Box<[bool]>; FACE_DIRECTIONS_NUM],
}

#[derive(Clone)]
pub struct QuadIndexBlockModelRef<'a> {
    pub quad_indices_per_face: &'a [Box<[u16]>; FACE_DIRECTIONS_NUM],
    pub texture_indices_per_face: &'a [Box<[u16]>; FACE_DIRECTIONS_NUM],
    pub quad_culling_per_face: &'a [Box<[bool]>; FACE_DIRECTIONS_NUM],
}