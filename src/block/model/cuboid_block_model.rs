use std::collections::HashMap;

use cgmath::{Vector2, Vector3};
use serde::Deserialize;

use crate::{block::{asset_loader::BaseCuboidBlockModels, FaceDirection, FACE_DIRECTIONS_NUM}, utils::{bool_true, none}};

use super::{quad_block_model::QuadBlockModel, ModelTexture, Quad};

#[derive(serde::Deserialize, Debug)]
pub struct CuboidBlockModelDeserialize {
    #[serde(default)]
    pub parent_model: Option<String>,
    #[serde(default)]
    pub texture_overrides: HashMap<ModelTexture, ModelTexture>,
    #[serde(default)]
    pub cuboids: Vec<CuboidDeserialize>,
}

impl CuboidBlockModelDeserialize {
    pub fn override_textures(&mut self) {
        for cuboid in self.cuboids.iter_mut() {
            for face in cuboid.faces.iter_mut() {
                if let Some(face) = face {
                    if let Some(texture_override) = self.texture_overrides.get(&face.texture) {
                        face.texture = texture_override.clone();
                    }
                }
            }
        }
        self.texture_overrides.clear();
    }
    
    pub fn combine(&mut self, other: &Self) {
        self.cuboids.extend(other.cuboids.clone());
    }
}

impl TryInto<CuboidBlockModel> for CuboidBlockModelDeserialize {
    type Error = ();
    fn try_into(mut self) -> Result<CuboidBlockModel, Self::Error> {
        if self.parent_model.is_some() { return Err(()); }
        self.override_textures();
        let mut cuboids = vec![];
        for cuboid in self.cuboids {
            cuboids.push(cuboid.try_into()?);
        }
        Ok(CuboidBlockModel { cuboids })
    }
}

#[derive(Debug)]
pub struct DeserializedCuboidModels {
    inner: HashMap<String, CuboidBlockModelDeserialize>
}

impl DeserializedCuboidModels {
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn insert(&mut self, name: String, model: CuboidBlockModelDeserialize) {
        self.inner.insert(name, model);
    }

    pub fn to_base_cuboid_models(mut self) -> BaseCuboidBlockModels {
        let mut base_cuboid_models = BaseCuboidBlockModels::new();
        let mut parentless_cuboid_models: HashMap<String, CuboidBlockModelDeserialize> = HashMap::new();

        let parentless_model_names = self.inner.iter().filter(|p| p.1.parent_model.is_none()).map(|f| f.0).cloned().collect::<Box<[String]>>();
        for model_name in parentless_model_names {
            let mut model = self.inner.remove(&model_name).unwrap();
            model.override_textures();
            parentless_cuboid_models.insert(model_name, model);
        }


        loop {
            let mut next_model_names = vec![];
            for (name, model) in self.inner.iter() {
                let parent_model = model.parent_model.as_ref().unwrap();
                if parentless_cuboid_models.contains_key(parent_model) { next_model_names.push(name.clone()) }
            }

            if next_model_names.len() == 0 { break; }

            for model_name in next_model_names {
                let mut model = self.inner.remove(&model_name).unwrap();
                let parent_model = parentless_cuboid_models.get(model.parent_model.as_ref().unwrap()).unwrap();
                model.combine(parent_model);
                model.override_textures();
                model.parent_model = None;
                parentless_cuboid_models.insert(model_name, model);
            }
        }

        for (name, model) in parentless_cuboid_models {
            if let Ok(model) = model.try_into() {
                base_cuboid_models.insert(name, model);
            }
        }
        
        base_cuboid_models
    }
}

#[derive(Debug)]
pub struct CuboidBlockModel {
    pub cuboids: Vec<Cuboid>,
}

impl CuboidBlockModel {
    pub fn bake(&self) -> QuadBlockModel {
        let mut quads_per_face : [Vec<Quad>; FACE_DIRECTIONS_NUM] = std::array::from_fn(|_| vec![]);
        let mut texture_indices_per_face: [Vec<u16>; FACE_DIRECTIONS_NUM] = std::array::from_fn(|_| vec![]);
        let mut quad_culling_per_face: [Vec<bool>; FACE_DIRECTIONS_NUM] = std::array::from_fn(|_| vec![]);
        for cuboid in self.cuboids.iter() {
            cuboid.append_quads(&mut quads_per_face);
            cuboid.append_texture_indices(&mut texture_indices_per_face);
            cuboid.append_quad_culling(&mut quad_culling_per_face)
        }

        QuadBlockModel {
            quads_per_face: quads_per_face.map(|f| f.into_boxed_slice()),
            texture_indices_per_face: texture_indices_per_face.map(|f| f.into_boxed_slice()),
            quad_culling_per_face: quad_culling_per_face.map(|f| f.into_boxed_slice()),
        }
    }
}

#[derive(Debug)]
pub struct Cuboid {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub faces: [Option<CuboidFace>; FACE_DIRECTIONS_NUM]
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct CuboidDeserialize {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    #[serde(deserialize_with = "deserialize_faces")]
    pub faces: [Option<CuboidFaceDeserialize>; FACE_DIRECTIONS_NUM]
}

impl TryInto<Cuboid> for CuboidDeserialize {
    type Error = ();
    fn try_into(self) -> Result<Cuboid, Self::Error> {
        let mut faces = std::array::from_fn(|_| None);
        for (i, face) in self.faces.into_iter().enumerate() {
            if let Some(face) = face {
                faces[i] = Some(face.try_into()?);
            } else {
                faces[i] = None;
            }
        }

        Ok(Cuboid {
            start: self.start,
            end: self.end,
            faces
        })
    }
}

impl Cuboid {
    pub fn append_quads(&self, quads_per_face: &mut [Vec<Quad>; FACE_DIRECTIONS_NUM]) {
        for face_num in 0..FACE_DIRECTIONS_NUM {
            let Some(cuboid_face) = &self.faces[face_num] else { continue; };
            let face_direction = unsafe { std::mem::transmute::<u8, FaceDirection>(face_num as u8) }; 
            let normal = face_direction.normal_f32();
            let quads = &mut quads_per_face[face_num];
            
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

    pub fn append_texture_indices(&self, texture_indices_per_face: &mut [Vec<u16>; FACE_DIRECTIONS_NUM]) {
        for face_num in 0..FACE_DIRECTIONS_NUM {
            let Some(face) = &self.faces[face_num] else { continue; };
            let texture_indices = &mut texture_indices_per_face[face_num];
            texture_indices.push(face.texture_index);
        }
    }

    pub fn append_quad_culling(&self, quad_culling_per_face: &mut [Vec<bool>; FACE_DIRECTIONS_NUM]) {
        for face_num in 0..FACE_DIRECTIONS_NUM {
            let Some(face) = &self.faces[face_num] else { continue; };
            let quad_culling = &mut quad_culling_per_face[face_num];
            quad_culling.push(face.culling);
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CuboidFace {
    pub uv_start: Vector2<f32>,
    pub uv_end: Vector2<f32>,
    pub texture_index: u16,
    #[serde(default = "bool_true")]
    pub culling: bool    
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CuboidFaceDeserialize {
    pub uv_start: Vector2<f32>,
    pub uv_end: Vector2<f32>,
    pub texture: ModelTexture,
    #[serde(default = "bool_true")]
    pub culling: bool    
}

impl TryInto<CuboidFace> for CuboidFaceDeserialize {
    type Error = ();
    fn try_into(self) -> Result<CuboidFace, Self::Error> {
        let ModelTexture::Index(texture_index) = self.texture else { return Err(()); };
        Ok(CuboidFace {
            uv_start: self.uv_start,
            uv_end: self.uv_end,
            texture_index,
            culling: self.culling,
        })
    }
}

#[derive(serde::Deserialize)]
pub struct CuboidFacesDeserialize {
    #[serde(default = "none")]
    #[serde(rename = "+x")]
    px: Option<CuboidFaceDeserialize>,

    #[serde(default = "none")]
    #[serde(rename = "-x")]
    nx: Option<CuboidFaceDeserialize>,

    #[serde(default = "none")]
    #[serde(rename = "+z")]
    pz: Option<CuboidFaceDeserialize>,

    #[serde(default = "none")]
    #[serde(rename = "-z")]
    nz: Option<CuboidFaceDeserialize>,

    #[serde(default = "none")]
    #[serde(rename = "+y")]
    py: Option<CuboidFaceDeserialize>,

    #[serde(default = "none")]
    #[serde(rename = "-y")]
    ny: Option<CuboidFaceDeserialize>,
}

impl Into<[Option<CuboidFaceDeserialize>; FACE_DIRECTIONS_NUM]> for CuboidFacesDeserialize {
    fn into(self) -> [Option<CuboidFaceDeserialize>; FACE_DIRECTIONS_NUM] {
        [self.px, self.nx, self.pz, self.nz, self.py, self.ny]
    }
}

pub fn deserialize_faces<'de, D>(deserialize: D) -> Result<[Option<CuboidFaceDeserialize>; FACE_DIRECTIONS_NUM], D::Error> where D: serde::Deserializer<'de> {
    CuboidFacesDeserialize::deserialize(deserialize).map(|f| f.into())
}