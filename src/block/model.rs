use std::{collections::HashMap, ops::{Deref, DerefMut}};

use cgmath::{Point2, Point3, Vector2, Vector3, Vector4};

use crate::collision::bounding_box::LocalBoundingBox;

use super::{asset_loader::QuadIndicesMap, block_state::{BlockState, Value}, light::{LightLevel, LIGHT_LEVEL_BITS}, Block, FaceDirection, Properties, FACE_DIRECTIONS_NUM};

pub const INDICES_PER_FACE: u32 = 6;
pub type IndexFormat = u32;

#[derive(serde::Deserialize, Debug)]
pub struct CuboidBlockModel {
    pub cuboids: Vec<Cuboid>
}

impl CuboidBlockModel {
    pub fn bake(&self) -> QuadBlockModel {
        let mut quads = vec![];
        let mut texture_indices = vec![];
        for cuboid in self.cuboids.iter() {
            cuboid.append_quads(&mut quads);
            cuboid.append_texture_indices(&mut texture_indices);
        }

        QuadBlockModel { quads: quads.into_boxed_slice(), texture_indices: texture_indices.into_boxed_slice() }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Cuboid {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub faces: [Option<CuboidFace>; FACE_DIRECTIONS_NUM]
}

impl Cuboid {
    pub fn append_quads(&self, quads: &mut Vec<Quad>) {
        for face_num in 0..FACE_DIRECTIONS_NUM {
            let Some(cuboid_face) = &self.faces[face_num] else { continue; };
            let face_direction = unsafe { std::mem::transmute::<u8, FaceDirection>(face_num as u8) }; 
            let normal = face_direction.normal();
            
            let vertex_positions = match face_direction {
                FaceDirection::PositiveX => [
                    Vector3::new(self.end.x, self.start.y, self.end.z),
                    Vector3::new(self.end.x, self.start.y, self.start.z),
                    self.end,
                    Vector3::new(self.end.x, self.end.y, self.start.z),
                ],
                FaceDirection::NegativeX => [
                    self.start,
                    Vector3::new(self.start.x, self.start.y, self.end.z),
                    Vector3::new(self.start.x, self.end.y, self.start.z),
                    Vector3::new(self.start.x, self.end.y, self.end.z),
                ],
                FaceDirection::PositiveZ => [
                    Vector3::new(self.start.x, self.start.y, self.end.z),
                    Vector3::new(self.end.x, self.start.y, self.end.z),
                    Vector3::new(self.start.x, self.end.y, self.end.z),
                    self.end,
                ],
                FaceDirection::NegativeZ => [
                    Vector3::new(self.end.x, self.start.y, self.start.z),
                    self.start,
                    Vector3::new(self.end.x, self.end.y, self.start.z),
                    Vector3::new(self.start.x, self.end.y, self.start.z),
                ],
                FaceDirection::PositiveY => [
                    Vector3::new(self.start.x, self.end.y, self.start.z),
                    Vector3::new(self.start.x, self.end.y, self.end.z),
                    Vector3::new(self.end.x, self.end.y, self.start.z),
                    self.end,
                ],
                FaceDirection::NegativeY => [
                    Vector3::new(self.start.x, self.start.y, self.end.z),
                    self.start,
                    Vector3::new(self.end.x, self.start.y, self.end.z),
                    Vector3::new(self.end.x, self.start.y, self.start.z),
                ]
            };

            let uv = [
                cuboid_face.uv_start,
                Vector2::new(cuboid_face.uv_end.x, cuboid_face.uv_start.y),
                Vector2::new(cuboid_face.uv_start.x, cuboid_face.uv_end.y),
                cuboid_face.uv_end
            ];

            let quad = Quad {
                normal,
                uv,
                vertex_positions,
            };

            quads.push(quad);
        }
    }

    pub fn append_texture_indices(&self, texture_indices: &mut Vec<u16>) {
        for face_num in 0..FACE_DIRECTIONS_NUM {
            let Some(face) = self.faces[face_num] else { continue; };
            texture_indices.push(face.texture_index);
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub struct CuboidFace {
    pub uv_start: Vector2<f32>,
    pub uv_end: Vector2<f32>,
    pub texture_index: u16,
    #[serde(default = "bool_true")]
    pub culling: bool    
}

const fn bool_true() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct BlockModelVariant {
    pub applied_model: String,
    pub quad_indices: Box<[u16]>,
    pub texture_indices: Box<[u16]>,
    pub required_state: Vec<(String, Value)>,
    pub standalone: bool,
    pub rotation: Vector3<f32>,
    pub hitboxes: Box<[LocalBoundingBox]>,
}

#[derive(serde::Deserialize)]
pub struct BlockModelVariantDeserialize {
    pub applied_model: String,
    pub required_state: Vec<(String, Value)>,
    pub standalone: bool,
    pub rotation: Vector3<f32>,
    #[serde(default)]
    pub hitboxes: Box<[LocalBoundingBox]>,
}


#[derive(serde::Deserialize)]
pub struct BlockDeserialize {
    pub variants: Vec<BlockModelVariantDeserialize>,
    
    #[serde(default)]
    pub default_state: BlockState,

    #[serde(default)]
    pub properties: Properties,
}

#[derive(Debug, Clone)]
pub struct BlockModelVariants {
    models: HashMap<String, Box<[BlockModelVariant]>>
}

impl BlockModelVariants {
    pub fn new() -> Self {
        Self { models: HashMap::new() }
    }

    pub fn get_quad_block_models(&self, block: &Block) -> Option<Box<[QuadIndexBlockModel]>> {
        let mut models = vec![];
        let Some(variants) = self.models.get(&block.name) else { return None; };

        'outer: for variant in variants.iter() {
            'inner: for (name, value) in variant.required_state.iter() {
                let Some(block_state_value) = block.block_state.get(&name) else { continue 'inner; };
                if *block_state_value == *value {
                    let quad_index_block_model = QuadIndexBlockModel {
                        quad_indices: variant.quad_indices.clone(),
                        texture_indices: variant.texture_indices.clone(),
                    };
                    if variant.standalone {
                        models.clear();
                        models.push(quad_index_block_model);
                        return Some(models.into_boxed_slice());
                    }
                    models.push(quad_index_block_model);
                }
            }
            if variant.required_state.is_empty() {
                let quad_index_block_model = QuadIndexBlockModel {
                    quad_indices: variant.quad_indices.clone(),
                    texture_indices: variant.texture_indices.clone(),
                };
                if variant.standalone {
                    models.clear();
                    models.push(quad_index_block_model);
                    return Some(models.into_boxed_slice());
                }
                models.push(quad_index_block_model);
            }
        }
        Some(models.into_boxed_slice())
    }

    pub fn insert(&mut self, block_name: String, model_variants: Box<[BlockModelVariant]>) {
        self.models.insert(block_name, model_variants);
    }
}

#[derive(Debug, Clone)]
pub struct QuadBlockModel {
    pub quads: Box<[Quad]>,
    pub texture_indices: Box<[u16]>,
}

#[derive(Debug, Clone)]
pub struct QuadIndexBlockModel {
    pub quad_indices: Box<[u16]>,
    pub texture_indices: Box<[u16]>,
}


#[derive(Debug, Clone)]
pub struct Quad {
    pub vertex_positions: [Vector3<f32>; 4],
    pub normal: Vector3<f32>,
    pub uv: [Vector2<f32>; 4],
}

impl Quad {
    pub fn into_raw(&self) -> QuadRaw {
        QuadRaw {
            vertex_positions: self.vertex_positions.map(|f| f.extend(0.0).into()),
            normal: self.normal.extend(0.0).into(),
            uv: self.uv.map(|f| f.into())
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Debug)]
pub struct QuadRaw {
    pub vertex_positions: [[f32; 4]; 4],
    pub normal: [f32; 4],
    pub uv: [[f32; 2]; 4],
}

pub struct Face {
    pub lighting: [LightLevel; 4], // 4xu4,
    pub block_position: [u8; 3], // 3xu5,
    pub texture_index: u16, // u16
    pub quad_index: u16, // u16
}

impl Face {
    pub const INDICES_PER_FACE: usize = 6;
    pub const VERTICES_PER_FACE: usize = 4;
    pub fn pack(&self) -> FacePacked {
        FacePacked(
            (self.lighting[0].get() as u64) |
            (self.lighting[1].get() as u64) << (LIGHT_LEVEL_BITS) |
            (self.lighting[2].get() as u64) << (2 * LIGHT_LEVEL_BITS) |
            (self.lighting[3].get() as u64) << (3 * LIGHT_LEVEL_BITS) |

            ((self.block_position[0] & 0b11111) as u64) << (4 * LIGHT_LEVEL_BITS) |
            ((self.block_position[1] & 0b11111) as u64) << (4 * LIGHT_LEVEL_BITS + 5) |
            ((self.block_position[2] & 0b11111) as u64) << (4 * LIGHT_LEVEL_BITS + 2 * 5) |

            (self.texture_index as u64) << (4 * LIGHT_LEVEL_BITS + 3 * 5) |

            (self.quad_index as u64) << (4 * LIGHT_LEVEL_BITS + 3 * 5 + 16)
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct FacePacked(u64);
