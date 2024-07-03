use std::ops::{Add, AddAssign, Sub, SubAssign};

use cgmath::{num_traits::Euclid, Vector3};
use crate::world::chunk::chunk_part::{CHUNK_SIZE_F32, CHUNK_SIZE_F64};

#[derive(Debug, Clone, Copy)]
pub struct RelVec3 {
    local_pos: Vector3<f32>,
    chunk_pos: Vector3<i32>,
}

impl RelVec3 {
    #[inline]
    pub fn local_pos(&self) -> Vector3<f32> {
        self.local_pos
    }

    #[inline]
    pub fn chunk_pos(&self) -> Vector3<i32> {
        self.chunk_pos
    }

    #[inline]
    pub fn apply_bounds(&mut self) {
        let (div_x, rem_x) = self.local_pos.x.div_rem_euclid(&CHUNK_SIZE_F32);
        let (div_y, rem_y) = self.local_pos.y.div_rem_euclid(&CHUNK_SIZE_F32);
        let (div_z, rem_z) = self.local_pos.z.div_rem_euclid(&CHUNK_SIZE_F32);

        self.local_pos = Vector3 {
            x: rem_x,
            y: rem_y,
            z: rem_z
        };

        // assert local position bounds
        assert!(self.local_pos.x < CHUNK_SIZE_F32 && self.local_pos.y < CHUNK_SIZE_F32 && self.local_pos.z < CHUNK_SIZE_F32);
        assert!(!self.local_pos.x.is_sign_negative() && !self.local_pos.y.is_sign_negative() && !self.local_pos.z.is_sign_negative());


        self.chunk_pos.x += div_x as i32;
        self.chunk_pos.y += div_y as i32;
        self.chunk_pos.z += div_z as i32;
    }


    pub fn edit<F: FnMut(&mut Self)>(&mut self, mut f: F) {
        f(self);
        self.apply_bounds();
    }
}

impl Add<Vector3<f32>> for RelVec3 {
    type Output = RelVec3;
    #[inline]
    fn add(mut self, rhs: Vector3<f32>) -> Self::Output {
        self.local_pos += rhs;
        self.apply_bounds();
        self
    }
}

impl AddAssign<Vector3<f32>> for RelVec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Vector3<f32>) {
        self.local_pos += rhs;
        self.apply_bounds();
    }
}

impl Sub<Vector3<f32>> for RelVec3 {
    type Output = RelVec3;
    #[inline]
    fn sub(mut self, rhs: Vector3<f32>) -> Self::Output {
        self.local_pos -= rhs;
        self.apply_bounds();
        self
    }
}

impl SubAssign<Vector3<f32>> for RelVec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vector3<f32>) {
        self.local_pos -= rhs;
        self.apply_bounds();
    }
}

impl Add<RelVec3> for RelVec3 {
    type Output = RelVec3;
    #[inline]
    fn add(mut self, rhs: RelVec3) -> Self::Output {
        self.local_pos += rhs.local_pos;
        self.chunk_pos += rhs.chunk_pos;
        self.apply_bounds();
        self
    }
}

impl AddAssign<RelVec3> for RelVec3 {
    #[inline]
    fn add_assign(&mut self, rhs: RelVec3) {
        self.local_pos += rhs.local_pos;
        self.chunk_pos += rhs.chunk_pos;
        self.apply_bounds();
    }
}

impl Sub<RelVec3> for RelVec3 {
    type Output = RelVec3;
    #[inline]
    fn sub(mut self, rhs: RelVec3) -> Self::Output {
        self.local_pos -= rhs.local_pos;
        self.chunk_pos -= rhs.chunk_pos;
        self.apply_bounds();
        self
    }
}

impl SubAssign<RelVec3> for RelVec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: RelVec3) {
        self.local_pos -= rhs.local_pos;
        self.chunk_pos -= rhs.chunk_pos;
        self.apply_bounds();
    }
}

impl From<Vector3<f32>> for RelVec3 {
    #[inline]
    fn from(value: Vector3<f32>) -> Self {
        let mut rel_vec = RelVec3 { local_pos: value, chunk_pos: Vector3 { x: 0, y: 0, z: 0 } };
        rel_vec.apply_bounds();
        rel_vec
    }
}

impl From<Vector3<f64>> for RelVec3 {
    #[inline]
    fn from(value: Vector3<f64>) -> Self {
        let (div_x, rem_x) = value.x.div_rem_euclid(&CHUNK_SIZE_F64);
        let (div_y, rem_y) = value.y.div_rem_euclid(&CHUNK_SIZE_F64);
        let (div_z, rem_z) = value.z.div_rem_euclid(&CHUNK_SIZE_F64);

        let local_pos = Vector3 {
            x: rem_x as f32,
            y: rem_y as f32,
            z: rem_z as f32
        };

        let chunk_pos = Vector3 {
            x: div_x as i32,
            y: div_y as i32,
            z: div_z as i32
        };

        // assert local position bounds
        assert!(local_pos.x < CHUNK_SIZE_F32 && local_pos.y < CHUNK_SIZE_F32 && local_pos.z < CHUNK_SIZE_F32);
        assert!(!local_pos.x.is_sign_negative() && !local_pos.y.is_sign_negative() && !local_pos.z.is_sign_negative());

        Self { local_pos, chunk_pos }
    }
}