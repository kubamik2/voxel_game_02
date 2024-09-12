use std::ops::{Add, Sub};

use cgmath::{num_traits::Euclid, Vector3, Zero};

use crate::{global_vector::GlobalVecU, world::{chunk::chunk_part::{chunk_part_position::ChunkPartPosition, CHUNK_SIZE, CHUNK_SIZE_I32}, PARTS_PER_CHUNK}};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChunkPosition {
    chunk_part_index: u8,
    chunk_part_position: ChunkPartPosition,
}

impl ChunkPosition {
    #[inline]
    pub fn new(chunk_part_position: Vector3<u32>, chunk_part_index: usize) -> Option<Self> {
        if chunk_part_index >= PARTS_PER_CHUNK { return None; }
        Some(Self {
            chunk_part_index: chunk_part_index as u8,
            chunk_part_position: ChunkPartPosition::new(chunk_part_position)?,
        })
    }

    #[inline]
    pub unsafe fn new_unchecked(chunk_part_position: Vector3<u32>, chunk_part_index: usize) -> Self {
        Self {
            chunk_part_index: chunk_part_index as u8,
            chunk_part_position: ChunkPartPosition::new_unchecked(chunk_part_position)
        }
    }

    #[inline]
    pub fn chunk_part_index(&self) -> usize {
        self.chunk_part_index as usize
    }

    #[inline]
    pub fn chunk_part_position(&self) -> ChunkPartPosition {
        self.chunk_part_position
    }

    #[inline]
    pub fn checked_add_i32(mut self, value: Vector3<i32>) -> Option<Self> {
        let mut new_position = Vector3::zero();

        let x = self.chunk_part_position.x as i32 + value.x;
        if !(0..CHUNK_SIZE_I32).contains(&x) { return None; }
        new_position.x = x as u32;

        let z = self.chunk_part_position.z as i32 + value.z;
        if !(0..CHUNK_SIZE_I32).contains(&z) { return None; }
        new_position.z = z as u32;

        let (div_y, rem_y) = (self.chunk_part_position.y as i32 + value.y).div_rem_euclid(&CHUNK_SIZE_I32);
        new_position.y = rem_y as u32;

        let chunk_part_index = self.chunk_part_index as i32 + div_y;
        if !(0..PARTS_PER_CHUNK as i32).contains(&chunk_part_index) { return None; }
        self.chunk_part_index = chunk_part_index as u8;
        self.chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(new_position) };
    
        Some(self)
    }

    #[inline]
    pub fn checked_sub_i32(mut self, value: Vector3<i32>) -> Option<Self> {
        let mut new_position = Vector3::zero();

        let x = self.chunk_part_position.x as i32 - value.x;
        if !(0..CHUNK_SIZE_I32).contains(&x) { return None; }
        new_position.x = x as u32;

        let z = self.chunk_part_position.z as i32 - value.z;
        if !(0..CHUNK_SIZE_I32).contains(&z) { return None; }
        new_position.z = z as u32;

        let (div_y, rem_y) = (self.chunk_part_position.y as i32 - value.y).div_rem_euclid(&CHUNK_SIZE_I32);
        new_position.y = rem_y as u32;

        let chunk_part_index = self.chunk_part_index as i32 - div_y;
        if !(0..PARTS_PER_CHUNK as i32).contains(&chunk_part_index) { return None; }
        self.chunk_part_index = chunk_part_index as u8;
        self.chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(new_position) };
    
        Some(self)
    }

    #[inline]
    pub fn checked_add_u32(mut self, value: Vector3<u32>) -> Option<Self> {
        let mut new_position = Vector3::zero();

        let x = self.chunk_part_position.x as u32 + value.x;   
        if x >= CHUNK_SIZE as u32 { return None; }
        new_position.x = x;

        let z = self.chunk_part_position.z as u32 + value.z;   
        if z >= CHUNK_SIZE as u32 { return None; }
        new_position.z = z;

        let (div_y, rem_y) = (self.chunk_part_position.y as u32 + value.y).div_rem_euclid(&(CHUNK_SIZE as u32));
        new_position.y = rem_y;


        let chunk_part_index = self.chunk_part_index as u32 + div_y;
        if chunk_part_index >= PARTS_PER_CHUNK as u32 { return None; }
        self.chunk_part_index = chunk_part_index as u8;
        self.chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(new_position) };

        Some(self)
    }

    #[inline]
    pub fn checked_sub_u32(mut self, value: Vector3<u32>) -> Option<Self> {
        let mut new_position = Vector3::zero();
        new_position.x = (self.chunk_part_position.x as u32).checked_sub(value.x)?;
        new_position.z = (self.chunk_part_position.z as u32).checked_sub(value.z)?;

        let (div_y, rem_y) = (self.chunk_part_position.y as i32 - value.y as i32).div_rem_euclid(&CHUNK_SIZE_I32);
        new_position.y = rem_y as u32;

        let chunk_part_index = self.chunk_part_index as i32 + div_y;
        if chunk_part_index < 0 || chunk_part_index >= PARTS_PER_CHUNK as i32 { return None; }
        self.chunk_part_index = chunk_part_index as u8;
        self.chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(new_position) };
        
        Some(self)
    }

    #[inline]
    pub unsafe fn unchecked_add_i32(mut self, value: Vector3<i32>) -> Self {
        let mut new_position = self.chunk_part_position.map(|f| f as i32) + value;

        let (div_y, rem_y) = new_position.y.div_rem_euclid(&CHUNK_SIZE_I32);
        new_position.y = rem_y;

        self.chunk_part_index = (self.chunk_part_index as i32 + div_y) as u8;
        self.chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(new_position.map(|f| f as u32)) };

        self
    }

    #[inline]
    pub unsafe fn unchecked_sub_i32(mut self, value: Vector3<i32>) -> Self {
        let mut new_position = self.chunk_part_position.map(|f| f as i32) - value;

        let (div_y, rem_y) = new_position.y.div_rem_euclid(&CHUNK_SIZE_I32);
        new_position.y = rem_y;

        self.chunk_part_index = (self.chunk_part_index as i32 + div_y) as u8;
        self.chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(new_position.map(|f| f as u32)) };

        self
    }

    #[inline]
    pub unsafe fn unchecked_add_u32(mut self, value: Vector3<u32>) -> Self {
        let mut new_position = self.chunk_part_position.map(|f| f as u32) + value;

        let (div_y, rem_y) = new_position.y.div_rem_euclid(&(CHUNK_SIZE as u32));
        new_position.y = rem_y;
        self.chunk_part_index = (self.chunk_part_index as u32 + div_y) as u8;
        self.chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(new_position) };

        self
    }

    #[inline]
    pub unsafe fn unchecked_sub_u32(mut self, value: Vector3<u32>) -> Self {
        let mut new_position = self.chunk_part_position.map(|f| f as u32) - value;

        let (div_y, rem_y) = new_position.y.div_rem_euclid(&(CHUNK_SIZE as u32));
        new_position.y = rem_y;
        self.chunk_part_index = (self.chunk_part_index as u32 + div_y) as u8;
        self.chunk_part_position = unsafe { ChunkPartPosition::new_unchecked(new_position) };

        self
    }
}

impl TryFrom<Vector3<u32>> for ChunkPosition {
    type Error = ();
    #[inline]
    fn try_from(value: Vector3<u32>) -> Result<Self, Self::Error> {
        let out = Self { chunk_part_index: 0, chunk_part_position: ChunkPartPosition::zero() };
        out.checked_add_u32(value).ok_or(())
    }
}

impl Into<Vector3<u32>> for ChunkPosition {
    #[inline]
    fn into(self) -> Vector3<u32> {
        let mut out = self.chunk_part_position.map(|f| f as u32);
        out.y += self.chunk_part_index as u32 * CHUNK_SIZE as u32;
        out
    }
}

impl TryFrom<GlobalVecU> for ChunkPosition {
    type Error = ();
    #[inline]
    fn try_from(value: GlobalVecU) -> Result<Self, Self::Error> {
        if value.chunk.y.is_negative() { return Err(()); }
        ChunkPosition::new(value.local(), value.chunk.y as usize).ok_or(())
    }
}
