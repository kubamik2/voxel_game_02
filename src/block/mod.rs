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
    id: BlockId,
    name: Box<str>,
    pub block_state: BlockState
}

impl Block {
    pub fn new(id: BlockId, name: &str, block_state: BlockState) -> Self {
        Self {
            id,
            name: name.into(), 
            block_state,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &BlockId {
        &self.id
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct BlockInformation {
    id: BlockId,
    name: Box<str>,
    default_state: BlockState,
    properties: Properties,
}

impl BlockInformation {
    pub fn new(id: BlockId, name: &str, default_state: BlockState, properties: Properties) -> Self {
        Self {
            id,
            name: name.into(),
            default_state,
            properties,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &BlockId {
        &self.id
    }

    pub fn default_state(&self) -> &BlockState {
        &self.default_state
    }

    pub fn properties(&self) -> &Properties {
        &self.properties
    }
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

impl AlphaMode {
    pub fn is_opaque(&self) -> bool {
        *self == AlphaMode::Opaque
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
