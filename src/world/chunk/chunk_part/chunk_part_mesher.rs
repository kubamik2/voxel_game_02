use std::{collections::HashMap, sync::mpsc::{channel, Receiver, Sender}, thread::JoinHandle};

use cgmath::Vector2;

use crate::{block::{light::LightLevel, model::{Face, FacePacked}, FaceDirection, FACE_DIRECTIONS_NUM}, BLOCK_MAP, BLOCK_MODEL_VARIANTS};

use super::{expanded_chunk_part::ExpandedChunkPart, CHUNK_SIZE};


#[derive(Debug)]
pub struct MeshingInput {
    pub expanded_chunk_part: Box<ExpandedChunkPart>,
    pub chunk_position: Vector2<i32>,
    pub chunk_part_index: usize,
}

pub struct MeshingOutput {
    pub faces: Box<[FacePacked]>,
    pub faces_num: usize,
    pub chunk_position: Vector2<i32>,
    pub chunk_part_index: usize,
}

pub struct ChunkPartMesher {
    thread_work_dispatcher: crate::thread_work_dispatcher::ThreadWorkDispatcher<MeshingInput, MeshingOutput>
}

impl ChunkPartMesher {
    pub fn new(num_threads: usize) -> Self {
        let x = crate::thread_work_dispatcher::ThreadWorkDispatcher::new(num_threads, Self::run_mesher);

        Self { thread_work_dispatcher: x }
    }

    fn run_mesher(receiver: Receiver<MeshingInput>, sender: Sender<MeshingOutput>) {
        for meshing_input in receiver.iter() {
            let now = std::time::Instant::now();
            let mut faces: Vec<FacePacked> = vec![];

            let max_block_pallet_id = meshing_input.expanded_chunk_part.block_pallet.ids().max().unwrap();
            let mut block_models_cache = vec![None; max_block_pallet_id as usize + 1];
            for (block_pallet_id, item) in meshing_input.expanded_chunk_part.block_pallet.iter() {
                let variants = BLOCK_MODEL_VARIANTS.get_quad_block_models(&item.block).unwrap();
                block_models_cache[block_pallet_id as usize] = Some(variants);
            }

            let mut block_info_cache = vec![None; max_block_pallet_id as usize + 1];
            for (block_pallet_id, item) in meshing_input.expanded_chunk_part.block_pallet.iter() {
                let block_info = BLOCK_MAP.get(item.block.name()).unwrap();
                block_info_cache[block_pallet_id as usize] = Some(block_info);
            }

            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let block_pallet_id = meshing_input.expanded_chunk_part.index_inner_block_pallet_id((x, y, z));

                        let block_models = block_models_cache[*block_pallet_id as usize].as_ref().unwrap();
                        let block_info = block_info_cache[*block_pallet_id as usize].unwrap();

                        for block_model in block_models {
                            let quad_indices_per_face = block_model.quad_indices_per_face;
                            let texture_indices_per_face = block_model.texture_indices_per_face;
                            let quad_culling_per_face = block_model.quad_culling_per_face;
                            for face_num in 0..FACE_DIRECTIONS_NUM {
                                let face_direction = unsafe { std::mem::transmute::<u8, FaceDirection>(face_num as u8) };
                                let normal = face_direction.normal().map(|f| f as i32);
                                
                                let quad_indices = &quad_indices_per_face[face_num];
                                let texture_indices = &texture_indices_per_face[face_num];
                                let quad_culling = &quad_culling_per_face[face_num];

                                let adjacent_block_position = ((x as i32 + 1 + normal.x) as usize, (y as i32 + 1 + normal.y) as usize, (z as i32 + 1 + normal.z) as usize);
                                let adjacent_block_pallet_id = meshing_input.expanded_chunk_part.index_block_pallet_id(adjacent_block_position);
                                let adjacent_block_info = block_info_cache[*adjacent_block_pallet_id as usize].unwrap();
                                let adjacent_block_light_level = *meshing_input.expanded_chunk_part.index_light_level(adjacent_block_position);
                                
                                let can_cull = {
                                    if block_info.properties().alpha_mode.is_opaque() {
                                        adjacent_block_info.properties().alpha_mode.is_opaque()
                                    } else {
                                        if adjacent_block_info.properties().alpha_mode.is_opaque() {
                                            true
                                        } else {
                                            block_info.id() == adjacent_block_info.id()
                                        }
                                    } 
                                };
                                let mut light = adjacent_block_light_level.get_block().max(adjacent_block_light_level.get_sky());
                                if face_num == 0 || face_num == 1 { light = light.saturating_sub(8); }
                                for (quad_index, texture_index, culling) in itertools::izip!(IntoIterator::into_iter(quad_indices), IntoIterator::into_iter(texture_indices), IntoIterator::into_iter(quad_culling)){
                                    if can_cull && *culling { continue; }
                                    faces.push(Face {
                                        block_position: [x as u8, y as u8, z as u8],
                                        lighting: [light; 4],
                                        texture_index: *texture_index,
                                        quad_index: *quad_index,
                                    }.pack())
                                }
                            }
                            
                        }
                    }
                }
            }
            // dbg!(now.elapsed());
            let faces_num = faces.len();
            let meshing_output = MeshingOutput {
                faces: faces.into_boxed_slice(),
                faces_num,
                chunk_position: meshing_input.chunk_position,
                chunk_part_index: meshing_input.chunk_part_index,
            };

            sender.send(meshing_output).unwrap();
        }
    }


    #[inline]
    pub fn collect_meshing_outputs(&mut self) -> Box<[MeshingOutput]> {
        self.thread_work_dispatcher.collect_outputs()
    }

    #[inline]
    pub fn mesh_chunk_part(&mut self, expanded_chunk_part: ExpandedChunkPart, chunk_position: Vector2<i32>, chunk_part_index: usize) -> Result<(), crate::thread_work_dispatcher::ThreadWorkDispatcherError<MeshingInput>> {
        self.thread_work_dispatcher.dispatch_work(MeshingInput {
            expanded_chunk_part: Box::new(expanded_chunk_part),
            chunk_position,
            chunk_part_index
        })
    }

    #[inline]
    pub fn idle_threads(&self) -> usize {
        self.thread_work_dispatcher.idle_threads()
    }
}