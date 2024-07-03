use std::{collections::HashMap, io::Read, ops::Index};

use cgmath::{Deg, Rotation, Vector3};

use super::{model::{BlockDeserialize, BlockModelVariant, BlockModelVariants, CuboidBlockModel, QuadBlockModel, QuadIndexBlockModel, QuadRaw}, quad_buffer::QuadBuffer, Block, BlockId, BlockInformation};

pub fn load_models<T: Into<std::path::PathBuf>>(path: T) -> anyhow::Result<BaseQuadBlockModels> {
    let path: std::path::PathBuf = path.into();
    let dir = path.read_dir()?;
    let mut baked_block_models = BaseQuadBlockModels::new();

    for entry_res in dir {
        let Ok(entry) = entry_res else { continue; };
        let model_name = entry.file_name().into_string().unwrap().trim_end_matches(".json").to_string();
        let mut contents = String::new();
        std::fs::File::open(entry.path())?.read_to_string(&mut contents)?;
        let block_model: CuboidBlockModel = serde_json::from_str(&contents)?;
        let baked_block_model = block_model.bake();   
        baked_block_models.insert(model_name, baked_block_model);
    }

    Ok(baked_block_models)
}

pub fn load_blocks<T: Into<std::path::PathBuf>>(path: T, baked_block_models: &BaseQuadBlockModels) -> anyhow::Result<(BlockMap, BlockList, BlockModelVariants, Vec<QuadRaw>)> {
    let path: std::path::PathBuf = path.into();
    let dir = path.read_dir()?;
    let mut id: BlockId = 0;
    let mut quads: Vec<QuadRaw> = vec![];
    let mut quad_map = QuadIndicesMap::new();
    let mut block_models = BlockModelVariants::new();
    let mut block_map = BlockMap::new();
    let mut block_list = BlockList::new();

    for entry_res in dir {
        let Ok(entry) = entry_res else { continue; };
        let block_name = entry.file_name().into_string().unwrap().trim_end_matches(".json").to_string();
        let mut contents = String::new();
        std::fs::File::open(entry.path())?.read_to_string(&mut contents)?;
        let block_deserialize: BlockDeserialize = serde_json::from_str(&contents)?;

        let block_info = BlockInformation {
            id,
            name: block_name.clone(),
            default_state: block_deserialize.default_state,
            properties: block_deserialize.properties,
        };
        let mut model_variants = vec![];
        for variant in block_deserialize.variants {
            let entry = quad_map.entry(variant.applied_model.clone(), variant.rotation);
            match entry {
                std::collections::hash_map::Entry::Occupied(occupied) => {
                    let quad_index_block_model = occupied.get().clone();
                    let model_variant = BlockModelVariant {
                        applied_model: variant.applied_model,
                        quad_indices: quad_index_block_model.quad_indices,
                        texture_indices: quad_index_block_model.texture_indices,
                        required_state: variant.required_state,
                        standalone: variant.standalone,
                        rotation: variant.rotation,
                        hitboxes: variant.hitboxes,
                    };
                    model_variants.push(model_variant);
                },
                std::collections::hash_map::Entry::Vacant(vacant) => {
                    let mut baked_block_model = baked_block_models.get(&variant.applied_model).unwrap().clone();
                    baked_block_model = create_rotated_quad_model(baked_block_model, variant.rotation);

                    let quads_len = quads.len();
                    quads.extend(baked_block_model.quads.iter().map(|f| f.into_raw()));

                    assert!(quads.len() < u16::MAX as usize);

                    let quad_indices = (quads_len..quads_len + baked_block_model.quads.len()).map(|f| f as u16).collect::<Box<[u16]>>();
                    let quad_index_block_model = QuadIndexBlockModel {
                        quad_indices,
                        texture_indices: baked_block_model.texture_indices
                    };

                    vacant.insert(quad_index_block_model.clone());
                    let model_variant = BlockModelVariant {
                        applied_model: variant.applied_model,
                        quad_indices: quad_index_block_model.quad_indices,
                        texture_indices: quad_index_block_model.texture_indices,
                        required_state: variant.required_state,
                        standalone: variant.standalone,
                        rotation: variant.rotation,
                        hitboxes: variant.hitboxes,
                    };
                    model_variants.push(model_variant);
                }
            }
        }
        block_models.insert(block_name.clone(), model_variants.into_boxed_slice());
        block_map.insert(block_name, block_info.clone());
        block_list.push(block_info);

        match id.checked_add(1) {
            Some(new_id) => { id = new_id },
            None => panic!("block id overflow")
        }
    }
    Ok((block_map, block_list, block_models, quads))
}

pub fn create_rotated_quad_model(mut baked_model: QuadBlockModel, rotation: Vector3<f32>) -> QuadBlockModel {
    let euler_angles = cgmath::Euler::new(Deg(rotation.x), Deg(rotation.y), Deg(rotation.z));
    let rotation_matrix = cgmath::Basis3::from(euler_angles);
    for quad in baked_model.quads.iter_mut() {
        for position in quad.vertex_positions.iter_mut() {
            *position = rotation_matrix.rotate_vector(*position);
        }

        quad.normal = rotation_matrix.rotate_vector(quad.normal);
    }

    baked_model
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