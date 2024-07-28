use std::ops::{Add, AddAssign, Sub, SubAssign};

use cgmath::{num_traits::Euclid, InnerSpace, MetricSpace, Vector3};
use crate::world::chunk::chunk_part::{CHUNK_SIZE_F32, CHUNK_SIZE_F64, INVERSE_CHUNK_SIZE};

#[derive(Debug, Clone, Copy)]
pub struct RelVec3 {
    local_pos: Vector3<f32>,
    pub chunk_pos: Vector3<i32>,
}

impl RelVec3 {
    #[inline]
    pub fn local_pos(&self) -> Vector3<f32> {
        self.local_pos
    }

    #[inline]
    pub fn apply_bounds(&mut self) {
        let (div_x, rem_x) = self.local_pos.x.div_rem_euclid(&CHUNK_SIZE_F32);
        let (div_y, rem_y) = self.local_pos.y.div_rem_euclid(&CHUNK_SIZE_F32);
        let (div_z, rem_z) = self.local_pos.z.div_rem_euclid(&CHUNK_SIZE_F32);

        self.local_pos = Vector3 {
            x: rem_x.clamp(0.0, CHUNK_SIZE_F32.next_down()),
            y: rem_y.clamp(0.0, CHUNK_SIZE_F32.next_down()),
            z: rem_z.clamp(0.0, CHUNK_SIZE_F32.next_down()),
        };

        self.chunk_pos.x += div_x as i32;
        self.chunk_pos.y += div_y as i32;
        self.chunk_pos.z += div_z as i32;
    }

    #[inline]
    pub fn edit<F: FnMut(&mut Self)>(&mut self, mut f: F) {
        f(self);
        self.apply_bounds();
    }

    #[inline]
    pub fn chunk_fractional_position(&self) -> Vector3<f32> {
        self.local_pos * INVERSE_CHUNK_SIZE + self.chunk_pos.map(|f| f as f32)
    }

    #[inline]
    pub fn normalize(&self) -> Vector3<f32> {
        self.chunk_fractional_position().normalize()
    }

    #[inline]
    pub fn distance2(&self, other: Self) -> f32 {
        let fract_self = self.chunk_fractional_position();
        let fract_other = other.chunk_fractional_position();

        fract_self.distance2(fract_other) * CHUNK_SIZE_F32 * CHUNK_SIZE_F32
    }

    #[inline]
    pub fn interpolate_voxels(&self, direction: Vector3<f32>, range: f32) -> InterpolateVoxelsIter {
        InterpolateVoxelsIter::new(*self, direction, range)
    }

    #[inline]
    pub fn interpolate_voxel_edges(&self, direction: Vector3<f32>, range: f32) -> InterpolateVoxelEdgesIter {
        InterpolateVoxelEdgesIter::new(*self, direction, range)
    }

    #[inline]
    pub fn floor(mut self) -> Self {
        self.local_pos = self.local_pos.map(|f| f.floor());
        self
    }

    #[inline]
    pub fn ceil(mut self) -> Self {
        self.local_pos = self.local_pos.map(|f| f.ceil());
        self.apply_bounds();
        self
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
            x: (rem_x as f32).clamp(0.0, CHUNK_SIZE_F32.next_down()),
            y: (rem_y as f32).clamp(0.0, CHUNK_SIZE_F32.next_down()),
            z: (rem_z as f32).clamp(0.0, CHUNK_SIZE_F32.next_down())
        };

        let chunk_pos = Vector3 {
            x: div_x as i32,
            y: div_y as i32,
            z: div_z as i32
        };

        Self { local_pos, chunk_pos }
    }
}

impl Into<Vector3<f32>> for RelVec3 {
    #[inline]
    fn into(self) -> Vector3<f32> {
        self.local_pos + self.chunk_pos.map(|f| f as f32 * CHUNK_SIZE_F32)
    }
}

impl Into<Vector3<f64>> for RelVec3 {
    #[inline]
    fn into(self) -> Vector3<f64> {
        self.local_pos.map(|f| f as f64) + self.chunk_pos.map(|f| f as f64 * CHUNK_SIZE_F64)
    }
}
pub struct InterpolateVoxelsIter {
    origin: RelVec3,
    direction: Vector3<f32>,
    point: RelVec3,
    inv_direction: Vector3<f32>,
    range2: f32,
}

impl InterpolateVoxelsIter {
    pub fn new(origin: RelVec3, direction: Vector3<f32>, range: f32) -> Self {
        let inv_direction = direction.map(|f| 1.0 / f);
        Self { origin, direction, point: origin, inv_direction, range2: range * range }
    }
}

impl Iterator for InterpolateVoxelsIter {
    type Item = RelVec3;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.origin.distance2(self.point) > self.range2 { return None; }
        let x_next_int = if self.direction.x.is_sign_positive() {
            (self.point.local_pos.x).floor()
        } else {
            (self.point.local_pos.x).ceil()
        } + self.direction.x.signum();
    
        let y_next_int = if self.direction.y.is_sign_positive() {
            (self.point.local_pos.y).floor()
        } else {
            (self.point.local_pos.y).ceil()
        } + self.direction.y.signum();

        let z_next_int = if self.direction.z.is_sign_positive() {
            (self.point.local_pos.z).floor()
        } else {
            (self.point.local_pos.z).ceil()
        } + self.direction.z.signum();

        let t_x = (x_next_int - self.point.local_pos.x) * self.inv_direction.x;
        let t_y = (y_next_int - self.point.local_pos.y) * self.inv_direction.y;
        let t_z = (z_next_int - self.point.local_pos.z) * self.inv_direction.z;

        let t_min = t_x.min(t_y.min(t_z));
        
        let mut voxel_point = self.point + self.direction * t_min * 0.99;
        voxel_point.local_pos = voxel_point.local_pos.map(|f| f.floor());

        self.point += self.direction * t_min;
        
        Some(voxel_point)
    }
}

pub struct InterpolateVoxelEdgesIter {
    origin: RelVec3,
    direction: Vector3<f32>,
    point: RelVec3,
    inv_direction: Vector3<f32>,
    range2: f32,
}

impl InterpolateVoxelEdgesIter {
    pub fn new(origin: RelVec3, direction: Vector3<f32>, range: f32) -> Self {
        let inv_direction = direction.map(|f| 1.0 / f);
        Self { origin, direction, point: origin, inv_direction, range2: range * range }
    }
}

impl Iterator for InterpolateVoxelEdgesIter {
    type Item = RelVec3;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.origin.distance2(self.point) > self.range2 { return None; }
        let x_next_int = if self.direction.x.is_sign_positive() {
            (self.point.local_pos.x).floor()
        } else {
            (self.point.local_pos.x).ceil()
        } + self.direction.x.signum();
    
        let y_next_int = if self.direction.y.is_sign_positive() {
            (self.point.local_pos.y).floor()
        } else {
            (self.point.local_pos.y).ceil()
        } + self.direction.y.signum();

        let z_next_int = if self.direction.z.is_sign_positive() {
            (self.point.local_pos.z).floor()
        } else {
            (self.point.local_pos.z).ceil()
        } + self.direction.z.signum();

        let t_x = (x_next_int - self.point.local_pos.x) * self.inv_direction.x;
        let t_y = (y_next_int - self.point.local_pos.y) * self.inv_direction.y;
        let t_z = (z_next_int - self.point.local_pos.z) * self.inv_direction.z;

        let t_min = t_x.min(t_y.min(t_z));

        self.point += self.direction * t_min;
        
        Some(self.point)
    }
}