use cgmath::Vector2;
use hashbrown::HashSet;

use crate::{global_vector::GlobalVecU, world::PARTS_PER_CHUNK};

use super::{chunk_generator::{ChunkGenerator, ChunkGeneratorOutput, GenerationStage}, chunk_map::ChunkMapLock, chunk_mesh_map::ChunkMeshMap, chunk_part::{chunk_part_mesher::ChunkPartMesher, expanded_chunk_part::ExpandedChunkPart}, dynamic_chunk_mesh::DynamicChunkMesh, Chunk};
use std::sync::Arc;

pub struct ChunkManager {
    pub chunk_map_lock: ChunkMapLock,
    pub chunk_mesh_map: ChunkMeshMap,
    chunk_generator: Arc<ChunkGenerator>,
    render_radius: u32,
    pub changed_blocks: Vec<GlobalVecU>,
}

impl ChunkManager {
    pub fn new(render_distance: u32, mesher_num_threads: usize, generator_num_threads: usize) -> Self {
        Self {
            chunk_map_lock: ChunkMapLock::default(),
            chunk_mesh_map: ChunkMeshMap::new(),
            chunk_generator: Arc::new(ChunkGenerator::new(generator_num_threads)),
            render_radius: render_distance,
            changed_blocks: vec![],
        }
    }

    pub fn render_radius(&self) -> u32 {
        self.render_radius
    }

    pub fn set_render_radius(&mut self, value: u32) {
        self.render_radius = value;
    }

    pub fn collect_meshing_outputs(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.chunk_generator.collect_meshing_outputs(device, queue, &mut self.chunk_mesh_map);
    }

    pub fn get_ready_meshes(&self) -> Box<[DynamicChunkMesh]> {
        let mut meshes = vec![];
        for mesh in self.chunk_mesh_map.values() {
            if mesh.parts_meshed.iter().all(|p| *p) {
                meshes.push(mesh.clone());
            }
        }
        meshes.into_boxed_slice()
    }
    
    pub fn update(&mut self, device: &wgpu::Device) {
        self.chunk_generator.update(device, &mut self.chunk_map_lock.write(), &mut self.chunk_mesh_map);
    }

    pub fn insert_chunks_around_player(&mut self, player_chunk_position: Vector2<i32>) {
        let mut chunk_map = self.chunk_map_lock.write();
        for z in -(self.render_radius as i32)..=self.render_radius as i32 {
            for x in -(self.render_radius as i32)..=self.render_radius as i32 {
                let pos = player_chunk_position + Vector2::new(x, z);
                if chunk_map.contains_position(&pos) { continue; }
                chunk_map.insert(Chunk::new_air(pos));
            }
        }
    }
}
