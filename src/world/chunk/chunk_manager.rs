use std::sync::Arc;

use cgmath::Vector2;

use crate::world::PARTS_PER_CHUNK;

use super::{chunk_generator::{ChunkGenerator, GenerationStage}, chunk_map::ChunkMap, chunk_mesh_map::ChunkMeshMap, chunk_part::{chunk_part_mesher::ChunkPartMesher, expanded_chunk_part::ExpandedChunkPart}, dynamic_chunk_mesh::DynamicChunkMesh, Chunk};

pub struct ChunkManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pub chunk_map: ChunkMap,
    pub chunk_mesh_map: ChunkMeshMap,
    mesher: ChunkPartMesher,
    chunk_generator: ChunkGenerator,
    render_radius: u32,
}

impl ChunkManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, render_distance: u32, mesher_num_threads: usize, generator_num_threads: usize) -> Self {
        Self {
            device,
            queue,
            chunk_map: ChunkMap::new(),
            chunk_mesh_map: ChunkMeshMap::new(),
            mesher: ChunkPartMesher::new(mesher_num_threads),
            chunk_generator: ChunkGenerator::new(generator_num_threads),
            render_radius: render_distance,
        }
    }

    pub fn render_distance(&self) -> u32 {
        self.render_radius
    }

    pub fn set_render_distance(&mut self, value: u32) {
        self.render_radius = value;
    }

    pub fn collect_meshing_outputs(&mut self) {
        for meshing_data in self.mesher.collect_meshing_outputs() {
            let Some(mesh) = self.chunk_mesh_map.get_mut(meshing_data.chunk_position) else { continue; };
            mesh.insert_meshed_chunk_part(&self.device, &self.queue, meshing_data);
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
        for chunk_lock in self.chunk_map.values() {
            let chunk = chunk_lock.lock().unwrap();
            let gen_stage = chunk.generation_stage;
            let mut pos = chunk.position;
            drop(chunk);
            pos += Vector2::new(self.render_radius as i32, self.render_radius as i32);
            let pos = pos.cast::<usize>().unwrap();
            s[pos.y][pos.x] = match gen_stage {
                GenerationStage::Empty => 'E',
                GenerationStage::Shape => 'S',
                GenerationStage::Terrain => 'T',
                GenerationStage::Decoration => 'D',
                GenerationStage::Full => 'F',
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

    pub fn update(&mut self) {
        self.chunk_generator.update();

        self.collect_meshing_outputs();
        let mut issued_generations = 0;
        let idle_gen_threads = self.chunk_generator.idle_threads();
        for chunk_lock in self.chunk_map.values() {
            if issued_generations >= idle_gen_threads { break; }
            {
                let Ok(mut chunk) = chunk_lock.try_lock() else { continue; };
                if chunk.generation_stage == GenerationStage::Full || chunk.generation_scheduled { continue; }
                if chunk.generation_stage == GenerationStage::Decoration {
                    if !self.chunk_map.is_chunk_surrounded_by_chunks_at_least_at_stage(chunk.position, GenerationStage::Decoration) { continue; }
                }
                chunk.generation_scheduled = true;
            }
            self.chunk_generator.generate_chunk_to_next_stage(chunk_lock.clone());
            issued_generations += 1;
        }

        let mut issued_meshings = 0;
        let idle_mesh_threads = self.mesher.idle_threads();

        // let now = std::time::Instant::now();
        for chunk_lock in self.chunk_map.values() {
            if issued_meshings >= idle_mesh_threads { break; }
            let chunk_position = {
                let Ok(chunk) = chunk_lock.try_lock() else { continue; };
                if chunk.generation_stage != GenerationStage::Full { continue; }
                chunk.position
            };

            if !self.chunk_map.is_chunk_surrounded_by_chunks_at_least_at_stage(chunk_position, GenerationStage::Full) { continue; }

            match self.chunk_mesh_map.entry(chunk_position) {
                std::collections::hash_map::Entry::Occupied(mut occupied) => {
                    let mesh = occupied.get_mut();
                    let mut i = 0;
                    for (chunk_part_index, (is_part_meshed, is_part_meshing_scheduled)) in itertools::izip!(mesh.parts_meshed, mesh.parts_meshing_scheduled).enumerate() {
                        if issued_meshings >= idle_mesh_threads { break; }
                        if is_part_meshed || is_part_meshing_scheduled { continue; }

                        mesh.parts_meshing_scheduled[chunk_part_index] = true;
                        let expanded_chunk_part = ExpandedChunkPart::new(&self.chunk_map, chunk_position, chunk_part_index).unwrap();
                        self.mesher.mesh_chunk_part(expanded_chunk_part, chunk_position, chunk_part_index).unwrap();
                        issued_meshings += 1;
                        i += 1;
                    }
                },
                std::collections::hash_map::Entry::Vacant(vacant) => {
                    let mut mesh = DynamicChunkMesh::new(&self.device, chunk_position);
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
        // dbg!(now.elapsed());
    }

    pub fn insert_chunks_around_player(&mut self, player_chunk_position: Vector2<i32>) {
        for z in -(self.render_radius as i32)..=self.render_radius as i32 {
            for x in -(self.render_radius as i32)..=self.render_radius as i32 {
                let pos = player_chunk_position + Vector2::new(x, z);
                if !self.chunk_map.contains_key(pos) {
                    self.chunk_map.insert(pos, Chunk::new_air(pos));
                }
            }
        }
    }
}