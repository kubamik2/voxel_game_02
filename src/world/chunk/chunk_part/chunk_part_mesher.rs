use std::{sync::mpsc::{channel, Receiver, Sender}, thread::JoinHandle};

use cgmath::Vector2;

use crate::{block::{light::LightLevel, model::{Face, FacePacked}}, BLOCK_MAP, BLOCK_MODEL_VARIANTS};

use super::{expanded_chunk_part::ExpandedChunkPart, CHUNK_SIZE};

pub const MESH_THREADS: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThreadStatus {
    Working,
    Idle
}

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
    senders: Vec<Sender<MeshingInput>>,
    receivers: Vec<Receiver<MeshingOutput>>,
    threads_status: Vec<ThreadStatus>,
}

impl ChunkPartMesher {
    pub fn new(thread_num: usize) -> Self {
        let mut senders = vec![];
        let mut receivers = vec![];
        let mut threads_status = vec![];

        for _ in 0..thread_num {
            let (input_sender, input_receiver) = channel();
            let (output_sender, output_receiver) = channel();
            rayon::spawn(move || { Self::run_mesher(input_receiver, output_sender) });
            senders.push(input_sender);
            receivers.push(output_receiver);
            threads_status.push(ThreadStatus::Idle);
        }

        Self { senders, receivers, threads_status }
    }

    fn run_mesher(receiver: Receiver<MeshingInput>, sender: Sender<MeshingOutput>) {
        for meshing_input in receiver.iter() {
            let mut faces: Vec<FacePacked> = vec![];
            let now = std::time::Instant::now();
            let block_model_variants_guard = BLOCK_MODEL_VARIANTS.lock().unwrap();
            let block_map_guard = BLOCK_MAP.lock().unwrap();
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let block = meshing_input.expanded_chunk_part.index_inner_block((x, y, z));
                        let block_info = { block_map_guard.get(&block.name).unwrap() };
                        let block_models = { block_model_variants_guard.get_quad_block_models(block).unwrap() };
                        for block_model in block_models {
                            let quad_indices = block_model.quad_indices;
                            let texture_indices = block_model.texture_indices;
                            for (quad_index, texture_index) in IntoIterator::into_iter(quad_indices).zip(IntoIterator::into_iter(texture_indices)) {
                                faces.push(Face {
                                    block_position: [x as u8, y as u8, z as u8],
                                    lighting: [LightLevel::new(0).unwrap(); 4],
                                    texture_index,
                                    quad_index
                                }.pack())
                            }
                        }
                    }
                }
            }
            // dbg!(now.elapsed());
            drop(block_model_variants_guard);
            drop(block_map_guard);
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
    fn get_idle_thread_index(&self) -> Option<usize> {
        self.threads_status.iter().position(|p| *p == ThreadStatus::Idle)
    }

    pub fn collect_meshing_outputs(&mut self) -> Box<[MeshingOutput]> {
        let mut meshing_outputs = vec![];
        for (i, receiver) in self.receivers.iter().enumerate() {
            for meshing_output in receiver.try_iter() {
                meshing_outputs.push(meshing_output);
            }
            self.threads_status[i] = ThreadStatus::Idle;
        }

        meshing_outputs.into_boxed_slice()
    }

    #[inline]
    pub fn mesh_chunk_part(&mut self, expanded_chunk_part: ExpandedChunkPart, chunk_position: Vector2<i32>, chunk_part_index: usize) -> Option<()> {
        self.get_idle_thread_index().map(|i| {
            self.threads_status[i] = ThreadStatus::Working;
            self.senders[i].send(MeshingInput {
                expanded_chunk_part: Box::new(expanded_chunk_part),
                chunk_position,
                chunk_part_index
            }).unwrap()
        })   
    }

    #[inline]
    pub fn idle_threads(&self) -> usize {
        self.threads_status.iter().filter(|p| **p == ThreadStatus::Idle).count()
    }
}