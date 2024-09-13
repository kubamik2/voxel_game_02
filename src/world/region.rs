use std::{io::{Read, Seek, Write}, os::linux::fs::MetadataExt};

use cgmath::Vector2;

use super::chunk::chunk_map::ChunkMap;
use hashbrown::HashMap;

pub const REGION_SIZE: usize = 16; // width and height of chunks in a region
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Region {
    pub chunks: ChunkMap,
    pub position: Vector2<i32>,
}

impl Region {
    pub fn new(position: Vector2<i32>) -> Self {
        Self {
            chunks: ChunkMap::new(),
            position,
        }
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, directory: P) -> anyhow::Result<()> {
        let mut path: std::path::PathBuf = directory.as_ref().to_owned();
        path.push(Self::position_to_file_name(self.position));
        let mut file = std::fs::File::create(path)?;
        let buf = rmp_serde::to_vec(self)?;
        file.write_all(&buf)?;
        Ok(())
    }

    pub fn load<P: AsRef<std::path::Path>>(directory: P, position: Vector2<i32>) -> anyhow::Result<Self> {
        let mut path: std::path::PathBuf = directory.as_ref().to_owned();
        path.push(Self::position_to_file_name(position));
        let mut file = std::fs::File::open(path)?;
        let size = file.metadata()?.st_size() as usize;
        let mut buf = Vec::with_capacity(size);
        file.read_to_end(&mut buf)?;

        Ok(rmp_serde::from_slice(&buf)?)
    }

    fn position_to_file_name(position: Vector2<i32>) -> String {
        format!("{}_{}", position.x, position.y)
    }

    pub fn are_all_chunks_unloaded(&self) -> bool {
        for chunk in self.chunks.values() {

        }
        true
    }
}

pub struct Regions(HashMap<Vector2<i32>, Region>);

impl Regions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}
