use std::ops::{Add, AddAssign, Sub, SubAssign};

use cgmath::{num_traits::Euclid, InnerSpace, MetricSpace, Vector3};
use crate::{chunk_position::ChunkPosition, world::chunk::chunk_part::{CHUNK_SIZE_F32, CHUNK_SIZE_F64, CHUNK_SIZE_I32, INVERSE_CHUNK_SIZE}};

#[derive(Debug, Clone, Copy)]
pub struct GlobalVecF {
    local: Vector3<f32>,
    pub chunk: Vector3<i32>,
}

impl GlobalVecF {
    #[inline]
    pub fn local(&self) -> Vector3<f32> {
        self.local
    }

    #[inline]
    pub fn apply_invariants(&mut self) {
        let (div_x, rem_x) = self.local.x.div_rem_euclid(&CHUNK_SIZE_F32);
        let (div_y, rem_y) = self.local.y.div_rem_euclid(&CHUNK_SIZE_F32);
        let (div_z, rem_z) = self.local.z.div_rem_euclid(&CHUNK_SIZE_F32);

        self.local = Vector3 {
            x: rem_x.clamp(0.0, CHUNK_SIZE_F32.next_down()),
            y: rem_y.clamp(0.0, CHUNK_SIZE_F32.next_down()),
            z: rem_z.clamp(0.0, CHUNK_SIZE_F32.next_down()),
        };

        self.chunk.x += div_x as i32;
        self.chunk.y += div_y as i32;
        self.chunk.z += div_z as i32;
    }

    #[inline]
    pub fn edit<F: FnMut(&mut Self)>(&mut self, mut f: F) {
        f(self);
        self.apply_invariants();
    }

    #[inline]
    pub fn chunk_fractional_position(&self) -> Vector3<f32> {
        self.local * INVERSE_CHUNK_SIZE + self.chunk.map(|f| f as f32)
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
        self.local = self.local.map(|f| f.floor());
        self
    }

    #[inline]
    pub fn ceil(mut self) -> Self {
        self.local = self.local.map(|f| f.ceil());
        self.apply_invariants();
        self
    }

    #[inline]
    pub fn in_bounds(&self) -> bool {
        self.chunk.y > 0 && self.chunk.y < CHUNK_SIZE_I32
    }
}

impl Add<Vector3<f32>> for GlobalVecF {
    type Output = GlobalVecF;
    #[inline]
    fn add(mut self, rhs: Vector3<f32>) -> Self::Output {
        self.local += rhs;
        self.apply_invariants();
        self
    }
}

impl AddAssign<Vector3<f32>> for GlobalVecF {
    #[inline]
    fn add_assign(&mut self, rhs: Vector3<f32>) {
        self.local += rhs;
        self.apply_invariants();
    }
}

impl Sub<Vector3<f32>> for GlobalVecF {
    type Output = GlobalVecF;
    #[inline]
    fn sub(mut self, rhs: Vector3<f32>) -> Self::Output {
        self.local -= rhs;
        self.apply_invariants();
        self
    }
}

impl SubAssign<Vector3<f32>> for GlobalVecF {
    #[inline]
    fn sub_assign(&mut self, rhs: Vector3<f32>) {
        self.local -= rhs;
        self.apply_invariants();
    }
}

impl Add<GlobalVecF> for GlobalVecF {
    type Output = GlobalVecF;
    #[inline]
    fn add(mut self, rhs: GlobalVecF) -> Self::Output {
        self.local += rhs.local;
        self.chunk += rhs.chunk;
        self.apply_invariants();
        self
    }
}

impl AddAssign<GlobalVecF> for GlobalVecF {
    #[inline]
    fn add_assign(&mut self, rhs: GlobalVecF) {
        self.local += rhs.local;
        self.chunk += rhs.chunk;
        self.apply_invariants();
    }
}

impl Sub<GlobalVecF> for GlobalVecF {
    type Output = GlobalVecF;
    #[inline]
    fn sub(mut self, rhs: GlobalVecF) -> Self::Output {
        self.local -= rhs.local;
        self.chunk -= rhs.chunk;
        self.apply_invariants();
        self
    }
}

impl SubAssign<GlobalVecF> for GlobalVecF {
    #[inline]
    fn sub_assign(&mut self, rhs: GlobalVecF) {
        self.local -= rhs.local;
        self.chunk -= rhs.chunk;
        self.apply_invariants();
    }
}

impl Add<GlobalVecU> for GlobalVecF {
    type Output = GlobalVecF;
    #[inline]
    fn add(mut self, rhs: GlobalVecU) -> Self::Output {
        self.local += rhs.local.map(|f| f as f32);
        self.chunk += rhs.chunk;
        self.apply_invariants();
        self
    }
}

impl AddAssign<GlobalVecU> for GlobalVecF {
    #[inline]
    fn add_assign(&mut self, rhs: GlobalVecU) {
        self.local += rhs.local.map(|f| f as f32);
        self.chunk += rhs.chunk;
        self.apply_invariants();
    }
}

impl Add<GlobalVecF> for GlobalVecU {
    type Output = GlobalVecF;
    #[inline]
    fn add(self, mut rhs: GlobalVecF) -> Self::Output {
        rhs.local += self.local.map(|f| f as f32);
        rhs.chunk += self.chunk;
        rhs.apply_invariants();
        rhs
    }
}

impl Sub<GlobalVecU> for GlobalVecF {
    type Output = GlobalVecF;
    #[inline]
    fn sub(mut self, rhs: GlobalVecU) -> Self::Output {
        self.local -= rhs.local.map(|f| f as f32);
        self.chunk -= rhs.chunk;
        self.apply_invariants();
        self
    }
}

impl SubAssign<GlobalVecU> for GlobalVecF {
    #[inline]
    fn sub_assign(&mut self, rhs: GlobalVecU) {
        self.local -= rhs.local.map(|f| f as f32);
        self.chunk -= rhs.chunk;
        self.apply_invariants();
    }
}

impl Sub<GlobalVecF> for GlobalVecU {
    type Output = GlobalVecF;
    #[inline]
    fn sub(self, mut rhs: GlobalVecF) -> Self::Output {
        rhs.local -= self.local.map(|f| f as f32);
        rhs.chunk -= self.chunk;
        rhs.apply_invariants();
        rhs
    }
}

impl From<Vector3<f32>> for GlobalVecF {
    #[inline]
    fn from(value: Vector3<f32>) -> Self {
        let mut rel_vec = GlobalVecF { local: value, chunk: Vector3 { x: 0, y: 0, z: 0 } };
        rel_vec.apply_invariants();
        rel_vec
    }
}

impl From<Vector3<f64>> for GlobalVecF {
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

        Self { local: local_pos, chunk: chunk_pos }
    }
}

impl Into<Vector3<f32>> for GlobalVecF {
    #[inline]
    fn into(self) -> Vector3<f32> {
        self.local + self.chunk.map(|f| f as f32 * CHUNK_SIZE_F32)
    }
}

impl Into<Vector3<f64>> for GlobalVecF {
    #[inline]
    fn into(self) -> Vector3<f64> {
        self.local.map(|f| f as f64) + self.chunk.map(|f| f as f64 * CHUNK_SIZE_F64)
    }
}
pub struct InterpolateVoxelsIter {
    origin: GlobalVecF,
    direction: Vector3<f32>,
    point: GlobalVecF,
    inv_direction: Vector3<f32>,
    range2: f32,
}

impl InterpolateVoxelsIter {
    pub fn new(origin: GlobalVecF, direction: Vector3<f32>, range: f32) -> Self {
        let inv_direction = direction.map(|f| 1.0 / f);
        Self { origin, direction, point: origin, inv_direction, range2: range * range }
    }
}

impl Iterator for InterpolateVoxelsIter {
    type Item = GlobalVecU;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.origin.distance2(self.point) > self.range2 { return None; }
        let x_next_int = if self.direction.x.is_sign_positive() {
            (self.point.local.x).floor()
        } else {
            (self.point.local.x).ceil()
        } + self.direction.x.signum();
    
        let y_next_int = if self.direction.y.is_sign_positive() {
            (self.point.local.y).floor()
        } else {
            (self.point.local.y).ceil()
        } + self.direction.y.signum();

        let z_next_int = if self.direction.z.is_sign_positive() {
            (self.point.local.z).floor()
        } else {
            (self.point.local.z).ceil()
        } + self.direction.z.signum();

        let t_x = (x_next_int - self.point.local.x) * self.inv_direction.x;
        let t_y = (y_next_int - self.point.local.y) * self.inv_direction.y;
        let t_z = (z_next_int - self.point.local.z) * self.inv_direction.z;

        let t_min = t_x.min(t_y.min(t_z));
        
        let voxel_point_f = self.point + self.direction * t_min * 0.99;
        let voxel_point_local = voxel_point_f.local.map(|f| f.floor() as i32);
        let voxel_point_chunk = voxel_point_f.chunk;

        self.point += self.direction * t_min;
        
        Some(GlobalVecU { local: voxel_point_local, chunk: voxel_point_chunk })
    }
}

pub struct InterpolateVoxelEdgesIter {
    origin: GlobalVecF,
    direction: Vector3<f32>,
    point: GlobalVecF,
    inv_direction: Vector3<f32>,
    range2: f32,
}

impl InterpolateVoxelEdgesIter {
    pub fn new(origin: GlobalVecF, direction: Vector3<f32>, range: f32) -> Self {
        let inv_direction = direction.map(|f| 1.0 / f);
        Self { origin, direction, point: origin, inv_direction, range2: range * range }
    }
}

impl Iterator for InterpolateVoxelEdgesIter {
    type Item = GlobalVecF;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.origin.distance2(self.point) > self.range2 { return None; }
        let x_next_int = if self.direction.x.is_sign_positive() {
            (self.point.local.x).floor()
        } else {
            (self.point.local.x).ceil()
        } + self.direction.x.signum();
    
        let y_next_int = if self.direction.y.is_sign_positive() {
            (self.point.local.y).floor()
        } else {
            (self.point.local.y).ceil()
        } + self.direction.y.signum();

        let z_next_int = if self.direction.z.is_sign_positive() {
            (self.point.local.z).floor()
        } else {
            (self.point.local.z).ceil()
        } + self.direction.z.signum();

        let t_x = (x_next_int - self.point.local.x) * self.inv_direction.x;
        let t_y = (y_next_int - self.point.local.y) * self.inv_direction.y;
        let t_z = (z_next_int - self.point.local.z) * self.inv_direction.z;

        let t_min = t_x.min(t_y.min(t_z));

        self.point += self.direction * t_min;
        
        Some(self.point)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalVecU {
    local: Vector3<i32>,
    pub chunk: Vector3<i32>,
}

impl GlobalVecU {
    #[inline]
    pub fn local(&self) -> Vector3<u32> {
        self.local.map(|f| f as u32)
    }

    #[inline]
    pub fn apply_invariants(&mut self) {
        let (div_x, rem_x) = self.local.x.div_rem_euclid(&CHUNK_SIZE_I32);
        let (div_y, rem_y) = self.local.y.div_rem_euclid(&CHUNK_SIZE_I32);
        let (div_z, rem_z) = self.local.z.div_rem_euclid(&CHUNK_SIZE_I32);

        self.local = Vector3 {
            x: rem_x,
            y: rem_y,
            z: rem_z,
        };

        self.chunk.x += div_x;
        self.chunk.y += div_y;
        self.chunk.z += div_z;
    }

    #[inline]
    pub fn edit<F: FnMut(&mut Self)>(&mut self, mut f: F) {
        f(self);
        self.apply_invariants();
    }

    #[inline]
    pub fn chunk_fractional_position(&self) -> Vector3<f32> {
        self.local.map(|f| f as f32) * INVERSE_CHUNK_SIZE + self.chunk.map(|f| f as f32)
    }

    #[inline]
    pub fn distance2(&self, other: Self) -> f32 {
        let fract_self = self.chunk_fractional_position();
        let fract_other = other.chunk_fractional_position();

        fract_self.distance2(fract_other) * CHUNK_SIZE_F32 * CHUNK_SIZE_F32
    }

    #[inline]
    pub fn interpolate_voxels(&self, direction: Vector3<f32>, range: f32) -> InterpolateVoxelsIter {
        InterpolateVoxelsIter::new((*self).into(), direction, range)
    }

    #[inline]
    pub fn in_bounds(&self) -> bool {
        self.chunk.y > 0 && self.chunk.y < CHUNK_SIZE_I32
    }
}

impl Into<GlobalVecF> for GlobalVecU {
    #[inline]
    fn into(self) -> GlobalVecF {
        GlobalVecF { local: self.local.map(|f| f as f32), chunk: self.chunk }
    }
}

impl Into<GlobalVecU> for GlobalVecF {
    #[inline]
    fn into(self) -> GlobalVecU {
        GlobalVecU { local: self.local.map(|f| f.floor() as i32), chunk: self.chunk }
    }
}

impl Add<Vector3<i32>> for GlobalVecU {
    type Output = GlobalVecU;
    #[inline]
    fn add(mut self, rhs: Vector3<i32>) -> Self::Output {
        self.local += rhs;
        self.apply_invariants();
        self
    }
}

impl AddAssign<Vector3<i32>> for GlobalVecU {
    #[inline]
    fn add_assign(&mut self, rhs: Vector3<i32>) {
        self.local += rhs;
        self.apply_invariants();
    }
}

impl Sub<Vector3<i32>> for GlobalVecU {
    type Output = GlobalVecU;
    #[inline]
    fn sub(mut self, rhs: Vector3<i32>) -> Self::Output {
        self.local -= rhs;
        self.apply_invariants();
        self
    }
}

impl SubAssign<Vector3<i32>> for GlobalVecU {
    #[inline]
    fn sub_assign(&mut self, rhs: Vector3<i32>) {
        self.local -= rhs;
        self.apply_invariants();
    }
}

impl Add<Vector3<f32>> for GlobalVecU {
    type Output = GlobalVecF;
    #[inline]
    fn add(self, rhs: Vector3<f32>) -> Self::Output {
        let mut out: GlobalVecF = self.into();
        out += rhs;
        out
    }
}

impl Sub<Vector3<f32>> for GlobalVecU {
    type Output = GlobalVecF;
    #[inline]
    fn sub(self, rhs: Vector3<f32>) -> Self::Output {
        let mut out: GlobalVecF = self.into();
        out -= rhs;
        out
    }
}

impl From<Vector3<i32>> for GlobalVecU {
    #[inline]
    fn from(value: Vector3<i32>) -> Self {
        let mut out = Self { chunk: Vector3::new(0, 0, 0), local: Vector3::new(0, 0, 0) };
        out += value;

        out
    }
}
