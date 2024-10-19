use std::sync::mpsc::{Receiver, Sender};

use cgmath::Vector2;

use crate::{block::{light::LightLevel, model::{Face, FacePacked}, FaceDirection, FACE_DIRECTIONS_NUM}, BLOCK_MODEL_VARIANTS};

use super::{expanded_chunk_part::ExpandedChunkPart, CHUNK_SIZE, CHUNK_SIZE_U32};


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
            let mut faces: Vec<FacePacked> = vec![];

            let max_block_pallet_id = meshing_input.expanded_chunk_part.block_pallet.ids().max().unwrap();
            let mut block_models_cache = vec![None; max_block_pallet_id as usize + 1];
            for (block_pallet_id, item) in meshing_input.expanded_chunk_part.block_pallet.iter() {
                let variants = BLOCK_MODEL_VARIANTS.get_quad_block_models(&item.block).unwrap();
                block_models_cache[block_pallet_id as usize] = Some(variants);
            }

            let mut block_properties_cache = vec![None; max_block_pallet_id as usize + 1];
            for (block_pallet_id, item) in meshing_input.expanded_chunk_part.block_pallet.iter() {
                let properties = item.block.properties().clone();
                block_properties_cache[block_pallet_id as usize] = Some(properties);
            }

            for y in 0..CHUNK_SIZE_U32 {
                for z in 0..CHUNK_SIZE_U32 {
                    for x in 0..CHUNK_SIZE_U32 {
                        let block_pallet_id = meshing_input.expanded_chunk_part.index_inner_block_pallet_id((x, y, z));

                        let block_models = block_models_cache[*block_pallet_id as usize].as_ref().unwrap();
                        let block_properties = block_properties_cache[*block_pallet_id as usize].unwrap();

                        for block_model in block_models {
                            let quad_indices_per_face = block_model.quad_indices_per_face;
                            let texture_indices_per_face = block_model.texture_indices_per_face;
                            let quad_culling_per_face = block_model.quad_culling_per_face;
                            for face_num in 0..FACE_DIRECTIONS_NUM {
                                let face_direction = unsafe { std::mem::transmute::<u8, FaceDirection>(face_num as u8) };
                                let normal = face_direction.normal_f32().map(|f| f as i32);
                                
                                let quad_indices = &quad_indices_per_face[face_num];
                                let texture_indices = &texture_indices_per_face[face_num];
                                let quad_culling = &quad_culling_per_face[face_num];

                                let adjacent_block_position = ((x as i32 + 1 + normal.x) as u32, (y as i32 + 1 + normal.y) as u32, (z as i32 + 1 + normal.z) as u32);
                                let adjacent_block_pallet_id = meshing_input.expanded_chunk_part.index_block_pallet_id(adjacent_block_position);
                                let adjacent_block_properties = block_properties_cache[*adjacent_block_pallet_id as usize].unwrap();
                                let adjacent_block_light_level = *meshing_input.expanded_chunk_part.index_light_level(adjacent_block_position);
                                
                                let can_cull = {
                                    if block_properties.alpha_mode.is_opaque() {
                                        adjacent_block_properties.alpha_mode.is_opaque()
                                    } else {
                                        if adjacent_block_properties.alpha_mode.is_opaque() {
                                            true
                                        } else {
                                            *block_pallet_id == *adjacent_block_pallet_id
                                        }
                                    } 
                                };
                                for (quad_index, texture_index, culling) in itertools::izip!(IntoIterator::into_iter(quad_indices), IntoIterator::into_iter(texture_indices), IntoIterator::into_iter(quad_culling)){
                                    if can_cull && *culling { continue; }
                                    faces.push(Face {
                                        block_position: [x as u8, y as u8, z as u8],
                                        lighting: [adjacent_block_light_level; 4],
                                        texture_index: *texture_index,
                                        quad_index: *quad_index,
                                    }.pack())
                                }
                            }
                            
                        }
                    }
                }
            }
            
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
    pub fn collect_meshing_outputs(&self) -> Box<[MeshingOutput]> {
        self.thread_work_dispatcher.collect_outputs()
    }

    #[inline]
    pub fn mesh_chunk_part(&self, expanded_chunk_part: ExpandedChunkPart, chunk_position: Vector2<i32>, chunk_part_index: usize) -> Result<(), crate::thread_work_dispatcher::ThreadWorkDispatcherError<MeshingInput>> {
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
