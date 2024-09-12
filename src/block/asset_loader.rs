use std::{collections::HashMap, io::Read, ops::Index};

use cgmath::{Deg, Rotation, Vector3, Zero};

use crate::{block::{model::{block_model_variant::BlockModelVariant, quad_block_model::QuadIndexBlockModel}, FACE_DIRECTIONS_NUM}, collision::bounding_box::LocalBoundingBox};

use super::{model::{block_model_variant::BlockModelVariants, cuboid_block_model::{CuboidBlockModel, CuboidBlockModelDeserialize, DeserializedCuboidModels}, quad_block_model::QuadBlockModel, BlockDeserialize, QuadRaw}, BlockId, BlockInformation, Properties, PropertiesOptional};
const TEXTURE_ATLAS_HEIGHT_IN_BLOCKS: u32 = 16;
const TEXTURE_ATLAS_WIDTH_IN_BLOCKS: u32 = 16;
const TEXTURE_ATLAS_UV_HEIGHT_PER_BLOCK: f32 = 1.0 / TEXTURE_ATLAS_HEIGHT_IN_BLOCKS as f32;
const TEXTURE_ATLAS_UV_WIDTH_PER_BLOCK: f32 = 1.0 / TEXTURE_ATLAS_WIDTH_IN_BLOCKS as f32;

pub fn load_models<T: Into<std::path::PathBuf>>(path: T) -> anyhow::Result<BaseCuboidBlockModels> {
    // load directory
    let path: std::path::PathBuf = path.into();
    let dir = path.read_dir()?;

    let mut deserialized_cuboid_models = DeserializedCuboidModels::new();

    // load all base models
    for entry_res in dir {
        let Ok(entry) = entry_res else { continue; };
        let model_name = entry.file_name().into_string().unwrap().trim_end_matches(".json").to_string();
        let mut contents = String::new();
        std::fs::File::open(entry.path())?.read_to_string(&mut contents)?;
        let block_model: CuboidBlockModelDeserialize = serde_json::from_str(&contents)?;
        deserialized_cuboid_models.insert(model_name, block_model);
    }
    let base_cuboid_models = deserialized_cuboid_models.to_base_cuboid_models();
    println!("Loaded models: {}", base_cuboid_models.model_names().cloned().collect::<Vec<String>>().join(", "));
    Ok(base_cuboid_models)
}

pub fn load_blocks<T: Into<std::path::PathBuf>>(path: T, base_cuboid_block_models: &BaseCuboidBlockModels) -> anyhow::Result<(BlockMap, BlockList, BlockModelVariants, Vec<QuadRaw>)> {
    // load directory
    let path: std::path::PathBuf = path.into();
    let dir = path.read_dir()?;

    let mut id: BlockId = 0;
    let mut quads: Vec<QuadRaw> = vec![];
    let mut block_models = BlockModelVariants::new();
    let mut block_map = BlockMap::new();
    let mut block_list = BlockList::new();

    for entry_res in dir {
        let Ok(entry) = entry_res else { continue; };

        let block_name = entry.file_name().into_string().unwrap().trim_end_matches(".json").to_string();

        // deserialize block model variants
        let mut contents = String::new();
        std::fs::File::open(entry.path())?.read_to_string(&mut contents)?;
        let block_deserialize: BlockDeserialize = serde_json::from_str(&contents)?;

        let block_info = BlockInformation::new(id, &block_name, block_deserialize.default_state, block_deserialize.base_properties.into());

        let mut block_model_variants = vec![];
        for variant in block_deserialize.variants {
            let mut quad_block_model = base_cuboid_block_models.get(&variant.model).unwrap().bake();
            quad_block_model.rotate(variant.rotation);

            let mut quad_indices_per_face: [Box<[u16]>; FACE_DIRECTIONS_NUM] = [Box::new([]), Box::new([]), Box::new([]), Box::new([]), Box::new([]), Box::new([])];
            for (i, model_quads ) in quad_block_model.quads_per_face.into_iter().enumerate() {
                let quads_len = quads.len();
                quads.extend(model_quads.iter().map(|f| f.into_raw()));
                let quad_indices = (quads_len..quads.len()).map(|f| f as u16).collect::<Box<[u16]>>();
                quad_indices_per_face[i] = quad_indices;
            }

            assert!(quads.len() < u16::MAX as usize);

            let quad_index_block_model = QuadIndexBlockModel {
                quad_indices_per_face,
                texture_indices_per_face: quad_block_model.texture_indices_per_face,
                quad_culling_per_face: quad_block_model.quad_culling_per_face,
            };

            let block_model_variant = BlockModelVariant {
                parent_model: variant.model,
                quad_indices_per_face: quad_index_block_model.quad_indices_per_face,
                texture_indices_per_face: quad_index_block_model.texture_indices_per_face,
                quad_culling_per_face: quad_index_block_model.quad_culling_per_face,
                required_state: variant.required_state,
                compound: variant.compound,
                rotation: variant.rotation,
                pivot: variant.pivot,
                hitboxes: variant.hitboxes,
                properties: variant.properties
            };
            block_model_variants.push(block_model_variant);
        }
        block_models.insert(block_name.clone(), block_model_variants.into_boxed_slice());
        block_map.insert(block_name, block_info.clone());
        block_list.push(block_info);

        match id.checked_add(1) {
            Some(new_id) => { id = new_id },
            None => panic!("block id overflow")
        }
    }
    Ok((block_map, block_list, block_models, quads))
}

pub struct QuadIndicesMap(HashMap<BlockModelVariantDescriptorRaw, QuadIndexBlockModel>);
    
impl QuadIndicesMap {
    pub fn new() -> Self {
        Self(HashMap::new()) 
    }

    pub fn get(&self, model_name: &str, rotation: &Vector3<f32>) -> Option<&QuadIndexBlockModel> {
        let k: BlockModelVariantDescriptorRaw = BlockModelVariantDesciptor { base_model_name: model_name.to_owned(), rotation: *rotation }.try_into().unwrap();
        self.0.get(&k)
    }

    pub fn entry(&mut self, model_name: String, rotation: Vector3<f32>) -> std::collections::hash_map::Entry<BlockModelVariantDescriptorRaw, QuadIndexBlockModel>{
        let k: BlockModelVariantDescriptorRaw = BlockModelVariantDesciptor { base_model_name: model_name, rotation }.try_into().unwrap();
        self.0.entry(k)
    }
}

#[derive(Debug, Clone)]
pub struct BlockMap(HashMap<String, BlockInformation>);

impl BlockMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, block_name: String, block: BlockInformation) {
        self.0.insert(block_name, block);
    }

    pub fn get(&self, block_name: &str) -> Option<&BlockInformation> {
        self.0.get(block_name)
    }
}

#[derive(Debug, Clone)]
pub struct BlockList(Vec<BlockInformation>);

impl BlockList {
    pub fn new() -> Self {
        Self(vec![])
    }

    #[inline]
    pub fn push(&mut self, block: BlockInformation) {
        self.0.push(block);
    }

    #[inline]
    pub fn get(&self, id: BlockId) -> Option<&BlockInformation> {
        self.0.get(id as usize)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<BlockInformation> {
        self.0.iter()
    }
}

impl Index<BlockId> for BlockList {
    type Output = BlockInformation;
    #[inline]
    fn index(&self, index: BlockId) -> &Self::Output {
        &self.0[index as usize]
    }
}

pub struct BaseQuadBlockModels(HashMap<String, QuadBlockModel>);

impl BaseQuadBlockModels {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, model_name: String, model: QuadBlockModel) {
        self.0.insert(model_name, model);
    }

    pub fn get(&self, model_name: &str) -> Option<&QuadBlockModel> {
        self.0.get(model_name)
    }
}


#[derive(Debug)]
pub struct BaseCuboidBlockModels(HashMap<String, CuboidBlockModel>);

impl BaseCuboidBlockModels {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, model_name: String, model: CuboidBlockModel) {
        self.0.insert(model_name, model);
    }

    pub fn get(&self, model_name: &str) -> Option<&CuboidBlockModel> {
        self.0.get(model_name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn model_names(&self) -> std::collections::hash_map::Keys<'_, String, CuboidBlockModel> {
        self.0.keys()
    }
}

pub struct BlockModelVariantDesciptor {
    base_model_name: String,
    rotation: Vector3<f32>
}

impl TryInto<BlockModelVariantDescriptorRaw> for BlockModelVariantDesciptor {
    type Error = ();
    fn try_into(self) -> Result<BlockModelVariantDescriptorRaw, Self::Error> {
        for i in 0..3 {
            let elem = self.rotation[i];
            if elem.is_infinite() || elem.is_nan() { return Err(()); }
        }

        Ok(BlockModelVariantDescriptorRaw {
            base_model_name: self.base_model_name,
            rotation: self.rotation.map(|f| unsafe { std::mem::transmute::<f32, u32>(f)})
        })
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct BlockModelVariantDescriptorRaw {
    base_model_name: String,
    rotation: Vector3<u32>
}
