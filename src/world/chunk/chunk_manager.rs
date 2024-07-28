use std::sync::Arc;

use cgmath::{Vector2, Vector3};
use hashbrown::HashSet;

use crate::{relative_vector::RelVec3, world::PARTS_PER_CHUNK};

use super::{chunk_generator::{ChunkGenerator, ChunkGeneratorOutput, GenerationStage}, chunk_map::ChunkMap, chunk_mesh_map::ChunkMeshMap, chunk_part::{chunk_part_mesher::ChunkPartMesher, expanded_chunk_part::ExpandedChunkPart}, dynamic_chunk_mesh::DynamicChunkMesh, Chunk};

pub struct ChunkManager {
    pub chunk_map: ChunkMap,
    pub chunk_mesh_map: ChunkMeshMap,
    mesher: ChunkPartMesher,
    chunk_generator: ChunkGenerator,
    render_radius: u32,
    scheduled_generations: HashSet<Vector2<i32>>,
    pub changed_blocks: Vec<RelVec3>,
}

impl ChunkManager {
    pub fn new(render_distance: u32, mesher_num_threads: usize, generator_num_threads: usize) -> Self {
        Self {
            chunk_map: ChunkMap::new(),
            chunk_mesh_map: ChunkMeshMap::new(),
            mesher: ChunkPartMesher::new(mesher_num_threads),
            chunk_generator: ChunkGenerator::new(generator_num_threads),
            render_radius: render_distance,
            scheduled_generations: HashSet::new(),
            changed_blocks: vec![],
        }
    }

    pub fn render_distance(&self) -> u32 {
        self.render_radius
    }

    pub fn set_render_distance(&mut self, value: u32) {
        self.render_radius = value;
    }

    pub fn collect_meshing_outputs(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for meshing_data in self.mesher.collect_meshing_outputs() {
            let Some(mesh) = self.chunk_mesh_map.get_mut(meshing_data.chunk_position) else { continue; };
            mesh.insert_meshed_chunk_part(device, queue, meshing_data);
        }
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
    
    pub fn print_chunk_generation_stages(&self) {
        let mut s = vec![vec!['.'; 1 + 2 * (self.render_radius as usize)]; 1 + 2 * (self.render_radius as usize)];
        for chunk in self.chunk_map.values() {
            let gen_stage = chunk.generation_stage;
            let mut pos = chunk.position;
            pos += Vector2::new(self.render_radius as i32, self.render_radius as i32);
            let pos = pos.cast::<usize>().unwrap();
            s[pos.y][pos.x] = match gen_stage {
                GenerationStage::Empty => 'E',
                GenerationStage::Shape => 'S',
                GenerationStage::Terrain => 'T',
                GenerationStage::Decoration => 'D',
                GenerationStage::Light => 'L',
            };
        }
        let mut string = String::new();
        for row in s {
            for c in row {
                string.push(c);
            }
            string.push('\n');
        }
        println!("{}", string);
    }

    pub fn update(&mut self, device: &wgpu::Device) {
        for gen_out in self.chunk_generator.iter_outputs() {
            match gen_out {
                ChunkGeneratorOutput::Chunk(chunk) => {
                    self.scheduled_generations.remove(&chunk.position);
                    self.chunk_map.insert_arc(chunk.position, chunk);
                },
                ChunkGeneratorOutput::Area(area) => {
                    for chunk in area.chunks {
                        self.scheduled_generations.remove(&chunk.position);
                        self.chunk_map.insert_arc(chunk.position, chunk);
                    }
                },
            }
        }

        let mut issued_generations = 0;
        let idle_gen_threads = self.chunk_generator.idle_threads();
        for chunk_position in self.chunk_map.positions().cloned().collect::<Box<[Vector2<i32>]>>() {
            if issued_generations >= idle_gen_threads { break; }
            let generation_stage = {
                let Some(chunk) = self.chunk_map.get(chunk_position) else { continue; };
                if chunk.generation_stage == GenerationStage::LAST_GENERATION_STAGE { continue; }
                if !self.chunk_map.is_chunk_surrounded_by_chunks_at_least_at_stage(chunk.position, chunk.generation_stage) { continue; }
                chunk.generation_stage
            };
            self.chunk_generator.generate_chunk_to_next_stage(generation_stage, &mut self.chunk_map, chunk_position.into(), &mut self.scheduled_generations);
            issued_generations += 1;
        }

        let mut issued_meshings = 0;
        let idle_mesh_threads = self.mesher.idle_threads();

        for chunk in self.chunk_map.values() {
            if issued_meshings >= idle_mesh_threads { break; }
            let chunk_position = {
                // if chunk.generation_stage != GenerationStage::LAST_GENERATION_STAGE { continue; }
                chunk.position
            };

            if !self.chunk_map.is_chunk_surrounded_by_chunks_at_least_at_stage(chunk_position, GenerationStage::LAST_GENERATION_STAGE) { continue; }

            match self.chunk_mesh_map.entry(chunk_position) {
                std::collections::hash_map::Entry::Occupied(mut occupied) => {
                    let mesh = occupied.get_mut();
                    for (chunk_part_index, (is_part_meshed, is_part_meshing_scheduled, needs_meshing)) in itertools::izip!(mesh.parts_meshed, mesh.parts_meshing_scheduled, mesh.parts_need_meshing).enumerate() {
                        if issued_meshings >= idle_mesh_threads { break; }
                        if (is_part_meshed && !needs_meshing) || is_part_meshing_scheduled { continue; }

                        mesh.parts_meshing_scheduled[chunk_part_index] = true;
                        let expanded_chunk_part = ExpandedChunkPart::new(&self.chunk_map, chunk_position, chunk_part_index).unwrap();
                        self.mesher.mesh_chunk_part(expanded_chunk_part, chunk_position, chunk_part_index).unwrap();
                        issued_meshings += 1;
                    }
                },
                std::collections::hash_map::Entry::Vacant(vacant) => {
                    let mut mesh = DynamicChunkMesh::new(device, chunk_position);
                    for chunk_part_index in 0..idle_mesh_threads.saturating_sub(issued_meshings).min(PARTS_PER_CHUNK) {
                        mesh.parts_meshing_scheduled[chunk_part_index] = true;
                        let expanded_chunk_part = ExpandedChunkPart::new(&self.chunk_map, chunk_position, chunk_part_index).unwrap();
                        self.mesher.mesh_chunk_part(expanded_chunk_part, chunk_position, chunk_part_index).unwrap();
                        issued_meshings += 1;
                    }
                    vacant.insert(mesh);
                }
            }
        }
    }

    pub fn insert_chunks_around_player(&mut self, player_chunk_position: Vector2<i32>) {
        for z in -(self.render_radius as i32)..=self.render_radius as i32 {
            for x in -(self.render_radius as i32)..=self.render_radius as i32 {
                let pos = player_chunk_position + Vector2::new(x, z);
                if self.chunk_map.contains_key(pos) || self.scheduled_generations.contains(&pos) { continue; }
                self.chunk_map.insert(pos, Chunk::new_air(pos));
            }
        }
    }
}