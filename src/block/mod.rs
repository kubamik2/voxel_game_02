use block_state::BlockState;
use cgmath::Vector3;

pub mod model;
pub mod light;
pub mod block_pallet;
pub mod block_state;
pub mod asset_loader;
pub mod quad_buffer;

pub const FACE_DIRECTIONS_NUM: usize = std::mem::variant_count::<FaceDirection>();
pub type BlockId = u16;
static LAST_ID: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub id: BlockId,
    pub name: String,
    pub block_state: BlockState
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct BlockInformation {
    pub id: BlockId,
    pub name: String,
    pub default_state: BlockState,
    pub properties: Properties,
}

impl Into<Block> for  BlockInformation {
    fn into(self) -> Block {
        Block { id: self.id, name: self.name, block_state: self.default_state }
    }
}

#[derive(serde::Deserialize, PartialEq, Clone, Debug)]
pub struct Properties {
    pub alpha_mode: AlphaMode,
    pub targetable: bool,
    pub replaceable: bool,
    pub collideable: bool
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            alpha_mode: AlphaMode::Opaque,
            collideable: true,
            replaceable: false,
            targetable: true
        }
    }
}

#[derive(serde::Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum AlphaMode {
    Opaque,
    Transparent,
    Translucent
}


pub enum FaceDirection {
    PositiveX,
    NegativeX,
    PositiveZ,
    NegativeZ,
    PositiveY,
    NegativeY,  
}

impl FaceDirection {
    pub const fn normal(&self) -> Vector3<f32> {
        match self {
            Self::PositiveX => Vector3 { x: 1.0, y: 0.0, z: 0.0 },
            Self::NegativeX => Vector3 { x: -1.0, y: 0.0, z: 0.0 },
            Self::PositiveZ => Vector3 { x: 0.0, y: 0.0, z: 1.0 },
            Self::NegativeZ => Vector3 { x: 0.0, y: 0.0, z: -1.0 },
            Self::PositiveY => Vector3 { x: 0.0, y: 1.0, z: 0.0 },
            Self::NegativeY => Vector3 { x: 0.0, y: -1.0, z: 0.0 },
        }
    }
}
