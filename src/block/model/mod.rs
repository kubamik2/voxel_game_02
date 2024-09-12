pub mod cuboid_block_model;
pub mod block_model_variant;
pub mod quad_block_model;

use block_model_variant::BlockModelVariantDeserialize;
use cgmath::{Deg, Rotation, Vector2, Vector3, Zero};

use super::{block_state::{BlockState, Value}, light::{LightLevel, LIGHT_LEVEL_BITS}, quad_buffer::QuadBuffer, AlphaMode, Block, FaceDirection, Properties, PropertiesDeserialize, PropertiesOptional, FACE_DIRECTIONS_NUM};


#[derive(serde::Deserialize)]
pub struct BlockDeserialize {
    pub variants: Box<[BlockModelVariantDeserialize]>,
    #[serde(default)]
    pub default_state: BlockState,
    #[serde(default)]
    pub base_properties: PropertiesDeserialize,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, serde::Deserialize)]
#[serde(untagged)]
pub enum ModelTexture {
    Placeholder(String),
    Index(u16)
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
            (self.lighting[0].to_u8() as u128) |
            (self.lighting[1].to_u8() as u128) << (2 * LIGHT_LEVEL_BITS) |
            (self.lighting[2].to_u8() as u128) << (2 * 2 * LIGHT_LEVEL_BITS) |
            (self.lighting[3].to_u8() as u128) << (2 * 3 * LIGHT_LEVEL_BITS) |

            ((self.block_position[0] & 0b11111) as u128) << (8 * LIGHT_LEVEL_BITS) |
            ((self.block_position[1] & 0b11111) as u128) << (8 * LIGHT_LEVEL_BITS + 5) |
            ((self.block_position[2] & 0b11111) as u128) << (8 * LIGHT_LEVEL_BITS + 2 * 5) |

            (self.texture_index as u128) << (8 * LIGHT_LEVEL_BITS + 3 * 5 + 1) |

            (self.quad_index as u128) << (8 * LIGHT_LEVEL_BITS + 3 * 5 + 1 + 16)
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct FacePacked(u128);
