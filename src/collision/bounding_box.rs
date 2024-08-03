use cgmath::Vector3;

use crate::{block::FaceDirection, global_vector::GlobalVecF, world::chunk::chunk_part::CHUNK_SIZE};

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub struct LocalBoundingBox {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
}

impl Default for LocalBoundingBox {
    fn default() -> Self {
        Self {
            start: Vector3::new(0.0, 0.0, 0.0),
            end: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

pub struct GlobalBoundingBox {
    pub start: GlobalVecF,
    pub end: GlobalVecF,
}

pub struct Ray {
    pub origin: GlobalVecF,
    pub direction: Vector3<f32>,
    pub direction_inverse: Vector3<f32>,
    pub length: f32,
}

impl Ray {
    pub fn new(origin: GlobalVecF, direction: Vector3<f32>, length: f32) -> Self {
        let direction_inverse = direction.map(|f| 1.0 / f);
        Self { origin, direction, length, direction_inverse }
    }
}

impl GlobalBoundingBox {
    #[inline]
    pub fn ray_intersection(&self, ray: &Ray) -> (f32, f32) {
        let start = self.start.local() + (self.start.chunk - ray.origin.chunk * CHUNK_SIZE as i32).map(|f| f as f32);
        let end = self.end.local() + ((self.end.chunk - ray.origin.chunk) * CHUNK_SIZE as i32).map(|f| f as f32);
        let origin = ray.origin.local();

        let mut t_min = 0.0;
        let mut t_max = f32::MAX;
        
        let t1 = (start.x - origin.x) * ray.direction_inverse.x;
        let t2 = (end.x -origin.x) * ray.direction_inverse.x;

        t_min = f32::min(f32::max(t1, t_min), f32::max(t2, t_min));
        t_max = f32::max(f32::min(t1, t_max), f32::min(t2, t_max));


        let t1 = (start.y - origin.y) * ray.direction_inverse.y;
        let t2 = (end.y - origin.y) * ray.direction_inverse.y;

        t_min = f32::min(f32::max(t1, t_min), f32::max(t2, t_min));
        t_max = f32::max(f32::min(t1, t_max), f32::min(t2, t_max));

        let t1 = (start.z - origin.z) * ray.direction_inverse.z;
        let t2 = (end.z - origin.z) * ray.direction_inverse.z;

        t_min = f32::min(f32::max(t1, t_min), f32::max(t2, t_min));
        t_max = f32::max(f32::min(t1, t_max), f32::min(t2, t_max));

        (t_min, t_max)
    }

    #[inline]
    pub fn ray_intersection_block_face(&self, ray: &Ray) -> Option<FaceDirection> {
        let start = self.start.local() + ((self.start.chunk - ray.origin.chunk) * CHUNK_SIZE as i32).map(|f| f as f32);
        let end = self.end.local() + ((self.end.chunk - ray.origin.chunk) * CHUNK_SIZE as i32).map(|f| f as f32);
        let origin = ray.origin.local();

        let mut t_min = 0.0;
        let mut t_max = f32::MAX;
        
        let t1 = (start.x - origin.x) * ray.direction_inverse.x;
        let t2 = (end.x - origin.x) * ray.direction_inverse.x;

        t_min = f32::min(f32::max(t1, t_min), f32::max(t2, t_min));
        t_max = f32::max(f32::min(t1, t_max), f32::min(t2, t_max));


        let t3 = (start.y - origin.y) * ray.direction_inverse.y;
        let t4 = (end.y - origin.y) * ray.direction_inverse.y;

        t_min = f32::min(f32::max(t3, t_min), f32::max(t4, t_min));
        t_max = f32::max(f32::min(t3, t_max), f32::min(t4, t_max));

        let t5 = (start.z - origin.z) * ray.direction_inverse.z;
        let t6 = (end.z - origin.z) * ray.direction_inverse.z;

        t_min = f32::min(f32::max(t5, t_min), f32::max(t6, t_min));
        t_max = f32::max(f32::min(t5, t_max), f32::min(t6, t_max));

        if t_min < t_max {
            return Some(
                if t_min == t1 {
                    FaceDirection::NegativeX
                } else if t_min == t2 {
                    FaceDirection::PositiveX
                } else if t_min == t3 {
                    FaceDirection::NegativeY
                } else if t_min == t4 {
                    FaceDirection::PositiveY
                } else if t_min == t5 {
                    FaceDirection::NegativeZ
                } else {
                    FaceDirection::PositiveZ
                }
            );
        }
        return None;
    }

    #[inline]
    pub fn intersecting_voxels(&self) -> Vec<GlobalVecF> {
        let start_floor = self.start.floor();
        let end_ceil = self.end.ceil();

        let diff: Vector3<f32> = (end_ceil - start_floor).into();
        let diff = diff.map(|f| f as i32);

        let mut intersecting_voxels = Vec::with_capacity((diff.x * diff.y * diff.z).abs() as usize);
        for dy in 0..diff.y {
            for dz in 0..diff.z {
                for dx in 0..diff.x {
                    intersecting_voxels.push(start_floor + Vector3::new(dx as f32, dy as f32, dz as f32));
                }
            }
        }

        intersecting_voxels
    }

    #[inline]
    pub fn voxels_beneath(&self) -> Vec<GlobalVecF> {
        let start_floor = self.start.floor();
        let end_ceil = self.end.ceil();

        let diff: Vector3<f32> = (end_ceil - start_floor).into();
        let diff = diff.map(|f| f as i32);

        let mut intersecting_voxels = Vec::with_capacity((diff.x * diff.y * diff.z).abs() as usize);

        for dy in 0..=1 {
            for dz in 0..diff.z {
                for dx in 0..diff.x {
                    intersecting_voxels.push(start_floor + Vector3::new(dx as f32, -1.0 + dy as f32, dz as f32));
                }
            }
        }

        intersecting_voxels
    }

    #[inline]
    pub fn ray_intersection_block_face_time(&self, ray: &Ray) -> Option<(FaceDirection, f32)> {
        let start = self.start.local() + ((self.start.chunk - ray.origin.chunk) * CHUNK_SIZE as i32).map(|f| f as f32);
        let end = self.end.local() + ((self.end.chunk - ray.origin.chunk) * CHUNK_SIZE as i32).map(|f| f as f32);
        let origin = ray.origin.local();

        let mut t_min = f32::MIN;
        let mut t_max = f32::MAX;
        
        let t1 = (start.x - origin.x) * ray.direction_inverse.x;
        let t2 = (end.x - origin.x) * ray.direction_inverse.x;

        t_min = f32::min(f32::max(t1, t_min), f32::max(t2, t_min));
        t_max = f32::max(f32::min(t1, t_max), f32::min(t2, t_max));


        let t3 = (start.y - origin.y) * ray.direction_inverse.y;
        let t4 = (end.y - origin.y) * ray.direction_inverse.y;

        t_min = f32::min(f32::max(t3, t_min), f32::max(t4, t_min));
        t_max = f32::max(f32::min(t3, t_max), f32::min(t4, t_max));

        let t5 = (start.z - origin.z) * ray.direction_inverse.z;
        let t6 = (end.z - origin.z) * ray.direction_inverse.z;

        t_min = f32::min(f32::max(t5, t_min), f32::max(t6, t_min));
        t_max = f32::max(f32::min(t5, t_max), f32::min(t6, t_max));

        if t_min < t_max && t_min >= 0.0 {
            return Some((
                if t_min == t1 {
                    FaceDirection::NegativeX
                } else if t_min == t2 {
                    FaceDirection::PositiveX
                } else if t_min == t3 {
                    FaceDirection::NegativeY
                } else if t_min == t4 {
                    FaceDirection::PositiveY
                } else if t_min == t5 {
                    FaceDirection::NegativeZ
                } else if t_min == t6 {
                    FaceDirection::PositiveZ
                } else {
                    unreachable!()
                }
            , t_min));
        }
        return None;
    }

    #[inline]
    pub fn intersects_bounding_box(&self, other: GlobalBoundingBox) -> bool {
        let start_chunk_pos = self.start.chunk;
        let start = self.start.local();
        let end: Vector3<f32> = {
            let mut end = self.end;
            end.chunk -= start_chunk_pos;
            end.into()
        };

        let other_start: Vector3<f32> = {
            let mut other_start = other.start;
            other_start.chunk -= start_chunk_pos;
            other_start.into()
        };
        let other_end: Vector3<f32> = {
            let mut other_end = other.end;
            other_end.chunk -= start_chunk_pos;
            other_end.into()
        };

        let largest_start_x = start.x.max(other_start.x);
        let smallest_end_x = end.x.min(other_end.x);

        let intersects_x = largest_start_x < smallest_end_x;


        let largest_start_z = start.z.max(other_start.z);
        let smallest_end_z = end.z.min(other_end.z);

        let intersects_z = largest_start_z < smallest_end_z;

        
        let largest_start_y = start.y.max(other_start.y);
        let smallest_end_y = end.y.min(other_end.y);

        let intersects_y = largest_start_y < smallest_end_y;

        intersects_x && intersects_y && intersects_z
    }
}