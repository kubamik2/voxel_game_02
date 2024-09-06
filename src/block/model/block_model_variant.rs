use std::collections::HashMap;

use cgmath::{Vector3, Zero};

use crate::{block::{block_state::Value, Block, PropertiesOptional, FACE_DIRECTIONS_NUM}, collision::bounding_box::LocalBoundingBox, utils::bool_false};

use super::quad_block_model::QuadIndexBlockModelRef;

#[derive(Debug, Clone)]
pub struct BlockModelVariant {
    pub parent_model: String,
    pub quad_indices_per_face: [Box<[u16]>; FACE_DIRECTIONS_NUM],
    pub texture_indices_per_face: [Box<[u16]>; FACE_DIRECTIONS_NUM],
    pub quad_culling_per_face: [Box<[bool]>; FACE_DIRECTIONS_NUM],
    pub required_state: Vec<(String, Value)>,
    pub compound: bool,
    pub rotation: Vector3<f32>,
    pub pivot: Vector3<f32>,
    pub hitboxes: Box<[LocalBoundingBox]>,
    pub properties: PropertiesOptional,
}

#[derive(serde::Deserialize)]
pub struct BlockModelVariantDeserialize {
    pub model: String,
    #[serde(default)]
    pub required_state: Vec<(String, Value)>,
    #[serde(default = "Vector3::zero")]
    pub rotation: Vector3<f32>,
    #[serde(default = "Vector3::zero")]
    pub pivot: Vector3<f32>,
    #[serde(default)]
    pub hitboxes: Box<[LocalBoundingBox]>,
    #[serde(default = "bool_false")]
    pub compound: bool,
    #[serde(default)]
    pub properties: PropertiesOptional,
}

#[derive(Debug, Clone)]
pub struct BlockModelVariants {
    models: HashMap<Box<str>, Box<[BlockModelVariant]>>
}

impl BlockModelVariants {
    pub fn new() -> Self {
        Self { models: HashMap::new() }
    }

    #[inline]
    pub fn get_quad_block_models<'a>(&'a self, block: &Block) -> Option<Box<[QuadIndexBlockModelRef<'a>]>> {
        let mut models = vec![];
        let Some(variants) = self.models.get(block.name.as_ref().into()) else { return None; };

        for variant in variants.iter() {
            'inner: for (name, value) in variant.required_state.iter() {
                let Some(block_state_value) = block.block_state.get(&name) else { continue 'inner; };
                if *block_state_value == *value {
                    let quad_index_block_model = QuadIndexBlockModelRef {
                        quad_indices_per_face: &variant.quad_indices_per_face,
                        texture_indices_per_face: &variant.texture_indices_per_face,
                        quad_culling_per_face: &variant.quad_culling_per_face,
                    };
                    if !variant.compound {
                        models.clear();
                        models.push(quad_index_block_model);
                        return Some(models.into_boxed_slice());
                    }
                    models.push(quad_index_block_model);
                }
            }
            if variant.required_state.is_empty() {
                let quad_index_block_model = QuadIndexBlockModelRef {
                    quad_indices_per_face: &variant.quad_indices_per_face,
                    texture_indices_per_face: &variant.texture_indices_per_face,
                    quad_culling_per_face: &variant.quad_culling_per_face,
                };
                if !variant.compound {
                    models.clear();
                    models.push(quad_index_block_model);
                    return Some(models.into_boxed_slice());
                }
                models.push(quad_index_block_model);
            }
        }
        Some(models.into_boxed_slice())
    }

    #[inline]
    pub fn get_model_variants<'a>(&'a self, block: &Block) -> Option<Box<[&'a BlockModelVariant]>> {
        let variants = self.models.get(block.name.as_ref())?;
        let mut current_block_variants = vec![];
        for variant in variants.iter() {
            'inner: for (name, value) in variant.required_state.iter() {
                let Some(block_state_value) = block.block_state.get(name) else { continue 'inner; };
                if *block_state_value == *value && !variant.compound {
                    current_block_variants.clear();
                    current_block_variants.push(variant);
                    return Some(current_block_variants.into_boxed_slice());
                }
            }
            if variant.required_state.is_empty() && !variant.compound {
                current_block_variants.clear();
                current_block_variants.push(variant);
                return Some(current_block_variants.into_boxed_slice());
            }
        }
        Some(current_block_variants.into_boxed_slice())
    }

    pub fn insert(&mut self, block_name: String, model_variants: Box<[BlockModelVariant]>) {
        self.models.insert(block_name.into_boxed_str(), model_variants);
    }
}
