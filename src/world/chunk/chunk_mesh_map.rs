use std::collections::HashMap;

use super::dynamic_chunk_mesh::DynamicChunkMesh;

pub struct ChunkMeshMap {
    meshes: HashMap<(i32, i32), DynamicChunkMesh>
}

impl ChunkMeshMap {
    #[inline]
    pub fn new() -> Self {
        Self { meshes: HashMap::new() }
    }

    #[inline]
    pub fn insert<T: Into<(i32, i32)>>(&mut self, chunk_position: T, mesh: DynamicChunkMesh) -> Option<DynamicChunkMesh> {
        self.meshes.insert(chunk_position.into(), mesh)
    }

    #[inline]
    pub fn remove<T: Into<(i32, i32)>>(&mut self, chunk_position: T) -> Option<DynamicChunkMesh> {
        self.meshes.remove(&chunk_position.into())
    }

    #[inline]
    pub fn get<T: Into<(i32, i32)>>(&self, chunk_position: T) -> Option<&DynamicChunkMesh> {
        self.meshes.get(&chunk_position.into())
    }

    #[inline]
    pub fn get_mut<T: Into<(i32, i32)>>(&mut self, chunk_position: T) -> Option<&mut DynamicChunkMesh> {
        self.meshes.get_mut(&chunk_position.into())
    }

    #[inline]
    pub fn iter(&self) -> std::collections::hash_map::Iter<(i32, i32), DynamicChunkMesh> {
        self.meshes.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<(i32, i32), DynamicChunkMesh> {
        self.meshes.iter_mut()
    }

    #[inline]
    pub fn values(&self) -> std::collections::hash_map::Values<(i32, i32), DynamicChunkMesh> {
        self.meshes.values()
    }

    #[inline]
    pub fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<(i32, i32), DynamicChunkMesh> {
        self.meshes.values_mut()
    }

    #[inline]
    pub fn entry<T: Into<(i32, i32)>>(&mut self, chunk_position: T) -> std::collections::hash_map::Entry<(i32, i32), DynamicChunkMesh> {
        self.meshes.entry(chunk_position.into())
    }
}