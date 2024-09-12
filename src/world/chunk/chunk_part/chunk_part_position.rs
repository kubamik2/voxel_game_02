use std::ops::Deref;

use cgmath::{Vector3, Zero};
use super::{CHUNK_SIZE, CHUNK_SIZE_I32};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkPartPosition(Vector3<u8>);

impl ChunkPartPosition {
    #[inline]
    pub fn new(position: Vector3<u32>) -> Option<Self> {
        if position.x >= CHUNK_SIZE as u32 || position.y >= CHUNK_SIZE as u32 || position.z >= CHUNK_SIZE as u32 { return None; }
        Some(Self(position.map(|f| f as u8)))
    }

    #[inline]
    pub fn zero() -> Self {
        Self(Vector3::zero())
    }

    #[inline]
    pub unsafe fn new_unchecked(position: Vector3<u32>) -> Self {
        Self(position.map(|f| f as u8))
    }

    #[inline]
    pub fn position(&self) -> Vector3<u8> {
        self.0.map(|f| f as u8)
    }

    #[inline]
    pub fn checked_add_i32(self, value: Vector3<i32>) -> Option<Self> {
        let x = (self.0.x as u32).checked_add_signed(value.x)?;
        if x >= CHUNK_SIZE as u32 { return None; }

        let y = (self.0.y as u32).checked_add_signed(value.y)?;
        if y >= CHUNK_SIZE as u32 { return None; }

        let z = (self.0.z as u32).checked_add_signed(value.z)?;
        if z >= CHUNK_SIZE as u32 { return None; }

        Some(Self(Vector3::new(x as u8, y as u8, z as u8)))
    }

    #[inline]
    pub fn checked_sub_i32(self, value: Vector3<i32>) -> Option<Self> {
        let x = (self.0.x as i32).checked_sub(value.x)?;
        if !(0..CHUNK_SIZE_I32).contains(&x) { return None; }

        let y = (self.0.y as i32).checked_sub(value.y)?;
        if !(0..CHUNK_SIZE_I32).contains(&y) { return None; }

        let z = (self.0.z as i32).checked_sub(value.z)?;
        if !(0..CHUNK_SIZE_I32).contains(&z) { return None; }

        Some(Self(Vector3::new(x as u8, y as u8, z as u8)))
    }

    #[inline]
    pub fn checked_add_u32(self, value: Vector3<u32>) -> Option<Self> {
        let x = (self.0.x as u32).checked_add(value.x)?;
        if x >= CHUNK_SIZE as u32 { return None; }

        let y = (self.0.y as u32).checked_add(value.y)?;
        if y >= CHUNK_SIZE as u32 { return None; }

        let z = (self.0.z as u32).checked_add(value.z)?;
        if z >= CHUNK_SIZE as u32 { return None; }

        Some(Self(Vector3::new(x as u8, y as u8, z as u8)))
    }

    #[inline]
    pub fn checked_sub_u32(self, value: Vector3<u32>) -> Option<Self> {
        let x = (self.0.x as u32).checked_sub(value.x)?;
        let y = (self.0.y as u32).checked_sub(value.y)?;
        let z = (self.0.z as u32).checked_sub(value.z)?;

        Some(Self(Vector3::new(x as u8, y as u8, z as u8)))
    }

    #[inline]
    pub unsafe fn unchecked_add_i32(self, value: Vector3<i32>) -> Self {
        Self((self.0.map(|f| f as i32) + value).map(|f| f as u8))
    }

    #[inline]
    pub unsafe fn unchecked_sub_i32(self, value: Vector3<i32>) -> Self {
        Self((self.0.map(|f| f as i32) - value).map(|f| f as u8))
    }

    #[inline]
    pub unsafe fn unchecked_add_u32(self, value: Vector3<u32>) -> Self {
        Self((self.0.map(|f| f as u32) + value).map(|f| f as u8))
    }

    #[inline]
    pub unsafe fn unchecked_sub_u32(self, value: Vector3<u32>) -> Self {
        Self((self.0.map(|f| f as u32) - value).map(|f| f as u8))
    }
}

impl Deref for ChunkPartPosition {
    type Target = Vector3<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
