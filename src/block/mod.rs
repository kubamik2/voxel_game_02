use std::sync::Arc;

use block_state::BlockState;
use cgmath::Vector3;
use light::LIGHT_LEVEL_MAX_VALUE;
use serde::Deserialize;

use crate::{AIR_ID, BLOCK_LIST, BLOCK_MODEL_VARIANTS};

pub mod model;
pub mod light;
pub mod block_pallet;
pub mod block_state;
pub mod asset_loader;
pub mod quad_buffer;

pub const FACE_DIRECTIONS_NUM: usize = std::mem::variant_count::<FaceDirection>();
pub type BlockId = u16;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Block {
    id: BlockId,
    name: Arc<str>,
    pub block_state: BlockState,
    properties: Properties,
}

impl Block {
    pub fn new(id: BlockId, name: &str, block_state: BlockState) -> Self {
        let mut block = Self {
            id,
            name: name.into(),
            block_state,
            properties: Properties::default(),
        };

        let mut properties = BLOCK_LIST.get(block.id).unwrap().base_properties;
        for variant in BLOCK_MODEL_VARIANTS.get_model_variants(&block).unwrap() {
            properties.join_optional(variant.properties);
        }

        block.properties = properties;

        block
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn id(&self) -> BlockId {
        self.id
    }

    #[inline]
    pub fn properties(&self) -> Properties {
        self.properties
    }

    #[inline]
    pub fn is_air(&self) -> bool {
        self.id() == *AIR_ID
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct BlockInformation {
    id: BlockId,
    name: Arc<str>,
    default_state: BlockState,
    base_properties: Properties,
}

impl BlockInformation {
    pub fn new(id: BlockId, name: &str, default_state: BlockState, base_properties: Properties) -> Self {
        Self {
            id,
            name: name.into(),
            default_state,
            base_properties,
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

    pub fn base_properties(&self) -> &Properties {
        &self.base_properties
    }
}

impl Into<Block> for BlockInformation {
    fn into(self) -> Block {
        let mut block = Block {
            id: self.id,
            name: self.name.clone(),
            block_state: self.default_state.clone(),
            properties: Properties::default(),
        };

        let mut properties = BLOCK_LIST.get(block.id).unwrap().base_properties;
        for variant in BLOCK_MODEL_VARIANTS.get_model_variants(&block).unwrap() {
            properties.join_optional(variant.properties);
        }

        block.properties = properties;

        block
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightAttenuation([u8; FACE_DIRECTIONS_NUM / 2]);

impl LightAttenuation {
    pub fn opaque() -> Self {
        Self(std::array::from_fn(|_| u8::MAX))
    }

    pub fn is_opaque(&self) -> bool {
        self.0.iter().all(|f| *f == LIGHT_LEVEL_MAX_VALUE)
    }

    pub fn is_transparent(&self) -> bool {
        self.0.iter().all(|f| *f == 0)
    }

    // TODO change name
    #[inline]
    pub const fn from_direction(&self, direction: Vector3<i32>) -> Option<u8> {
        match direction {
            Vector3 { x: 1, y: 0, z: 0 } => Some(self.0[0] & LIGHT_LEVEL_MAX_VALUE),
            Vector3 { x: -1, y: 0, z: 0 } => Some(self.0[0] >> 4),
            Vector3 { x: 0, y: 0, z: 1 } => Some(self.0[1] & LIGHT_LEVEL_MAX_VALUE),
            Vector3 { x: 0, y: 0, z: -1 } => Some(self.0[1] >> 4),
            Vector3 { x: 0, y: 1, z: 0 } => Some(self.0[2] & LIGHT_LEVEL_MAX_VALUE),
            Vector3 { x: 0, y: -1, z: 0 } => Some(self.0[2] >> 4),
            _ => None
        }
    }
}

const fn light_level_max_value() -> u8 { LIGHT_LEVEL_MAX_VALUE }
#[derive(serde::Deserialize)]
pub struct LightAttenuationDeserialize {
    #[serde(default = "light_level_max_value")]
    #[serde(rename = "+x")]
    px: u8,
    #[serde(default = "light_level_max_value")]
    #[serde(rename = "-x")]
    nx: u8,
    #[serde(default = "light_level_max_value")]
    #[serde(rename = "+z")]
    pz: u8,
    #[serde(default = "light_level_max_value")]
    #[serde(rename = "-z")]
    nz: u8,
    #[serde(default = "light_level_max_value")]
    #[serde(rename = "+y")]
    py: u8,
    #[serde(default = "light_level_max_value")]
    #[serde(rename = "-y")]
    ny: u8,
}

impl Into<LightAttenuation> for LightAttenuationDeserialize {
    fn into(self) -> LightAttenuation {
        assert!(self.px <= LIGHT_LEVEL_MAX_VALUE);
        assert!(self.nx <= LIGHT_LEVEL_MAX_VALUE);
        assert!(self.pz <= LIGHT_LEVEL_MAX_VALUE);
        assert!(self.nz <= LIGHT_LEVEL_MAX_VALUE);
        assert!(self.py <= LIGHT_LEVEL_MAX_VALUE);
        assert!(self.ny <= LIGHT_LEVEL_MAX_VALUE);
        
        LightAttenuation([self.px | (self.nx << 4), self.pz | (self.nz << 4), self.py | (self.ny << 4)])
    }
}

pub fn deserialize_light_attenuation<'de, D>(deserialize: D) -> Result<LightAttenuation, D::Error> where D: serde::Deserializer<'de> {
    LightAttenuationDeserialize::deserialize(deserialize).map(|f| f.into())
}

pub fn deserialize_light_attenuation_option<'de, D>(deserialize: D) -> Result<Option<LightAttenuation>, D::Error> where D: serde::Deserializer<'de> {
    Option::<LightAttenuationDeserialize>::deserialize(deserialize).map(|f| f.map(|f| f.into()))
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Copy, Debug)]
pub struct Properties {
    #[serde(default)]
    pub alpha_mode: AlphaMode,

    #[serde(default = "bool_true")]
    pub targetable: bool,

    #[serde(default = "bool_false")]
    pub replaceable: bool,

    #[serde(default = "bool_true")]
    pub collideable: bool,

    pub light_attenuation: LightAttenuation,

    #[serde(default = "u8_0")]
    pub emitted_light: u8,
}

#[derive(serde::Deserialize)]
pub struct PropertiesDeserialize {
    #[serde(default)]
    pub alpha_mode: AlphaMode,

    #[serde(default = "bool_true")]
    pub targetable: bool,

    #[serde(default = "bool_false")]
    pub replaceable: bool,

    #[serde(default = "bool_true")]
    pub collideable: bool,

    pub light_attenuation: LightAttenuationDeserialize,

    #[serde(default = "u8_0")]
    pub emitted_light: u8,
}

impl Default for PropertiesDeserialize {
    fn default() -> Self {
        Self {
            alpha_mode: AlphaMode::default(),
            targetable: true,
            replaceable: false,
            collideable: true,
            light_attenuation: LightAttenuationDeserialize { px: LIGHT_LEVEL_MAX_VALUE, nx: LIGHT_LEVEL_MAX_VALUE, py: LIGHT_LEVEL_MAX_VALUE, ny: LIGHT_LEVEL_MAX_VALUE, pz: LIGHT_LEVEL_MAX_VALUE, nz: LIGHT_LEVEL_MAX_VALUE },
            emitted_light: 0
        }       
    }
}

impl Into<Properties> for PropertiesDeserialize {
    fn into(self) -> Properties {
        Properties {
            alpha_mode: self.alpha_mode,
            targetable: self.targetable,
            replaceable: self.replaceable,
            collideable: self.collideable,
            light_attenuation: self.light_attenuation.into(),
            emitted_light: self.emitted_light,
        }
    }
}

impl Properties {
    pub fn join_optional(&mut self, optional: PropertiesOptional) {
        self.alpha_mode = optional.alpha_mode.unwrap_or(self.alpha_mode);
        self.targetable = optional.targetable.unwrap_or(self.targetable);
        self.replaceable = optional.replaceable.unwrap_or(self.replaceable);
        self.collideable = optional.collideable.unwrap_or(self.collideable);
        self.light_attenuation = optional.light_attenuation.unwrap_or(self.light_attenuation);
        self.emitted_light = optional.emitted_light.unwrap_or(self.emitted_light);
    }
}

const fn bool_true() -> bool { true }
const fn bool_false() -> bool { false }
const fn none<T>() -> Option<T> { None }
const fn u8_0() -> u8 { 0 }

impl Default for Properties {
    fn default() -> Self {
        Self {
            alpha_mode: AlphaMode::default(),
            targetable: true,
            replaceable: false,
            collideable: true,
            light_attenuation: LightAttenuation::opaque(),
            emitted_light: 0,
        }
    }
}

#[derive(serde::Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct PropertiesOptional {
    #[serde(default = "none")]
    pub alpha_mode: Option<AlphaMode>,

    #[serde(default = "none")]
    pub targetable: Option<bool>,

    #[serde(default = "none")]
    pub replaceable: Option<bool>,

    #[serde(default = "none")]
    pub collideable: Option<bool>,

    #[serde(default = "none")]
    #[serde(deserialize_with = "deserialize_light_attenuation_option")]
    pub light_attenuation: Option<LightAttenuation>,

    #[serde(default = "none")]
    pub emitted_light: Option<u8>,
}

impl Default for PropertiesOptional {
    fn default() -> Self {
        Self {
            alpha_mode: None,
            targetable: None,
            replaceable: None,
            collideable: None,
            emitted_light: None,
            light_attenuation: None,
        }
    }
}


#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum AlphaMode {
    Opaque,
    Transparent,
    Translucent
}

impl Default for AlphaMode {
    fn default() -> Self {
        Self::Opaque
    }
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
    pub const fn normal_f32(&self) -> Vector3<f32> {
        match self {
            Self::PositiveX => Vector3 { x: 1.0, y: 0.0, z: 0.0 },
            Self::NegativeX => Vector3 { x: -1.0, y: 0.0, z: 0.0 },
            Self::PositiveZ => Vector3 { x: 0.0, y: 0.0, z: 1.0 },
            Self::NegativeZ => Vector3 { x: 0.0, y: 0.0, z: -1.0 },
            Self::PositiveY => Vector3 { x: 0.0, y: 1.0, z: 0.0 },
            Self::NegativeY => Vector3 { x: 0.0, y: -1.0, z: 0.0 },
        }
    }

    pub const fn normal_i32(&self) -> Vector3<i32> {
        match self {
            Self::PositiveX => Vector3 { x: 1, y: 0, z: 0 },
            Self::NegativeX => Vector3 { x: -1, y: 0, z: 0 },
            Self::PositiveZ => Vector3 { x: 0, y: 0, z: 1 },
            Self::NegativeZ => Vector3 { x: 0, y: 0, z: -1 },
            Self::PositiveY => Vector3 { x: 0, y: 1, z: 0 },
            Self::NegativeY => Vector3 { x: 0, y: -1, z: 0 },
        }
    }
}
