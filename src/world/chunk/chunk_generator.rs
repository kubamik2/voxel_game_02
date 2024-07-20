use std::sync::{mpsc::{Receiver, Sender}, Arc, Mutex};

use cgmath::Vector2;

use crate::{block::Block, thread_work_dispatcher::ThreadWorkDispatcher, BLOCK_MAP};

use super::{chunk_part::CHUNK_SIZE, Chunk};

pub struct ChunkGeneratorInput {
    chunk: Arc<Mutex<Chunk>>
}

pub struct ChunkGeneratorOutput {
    // chunk: Arc<Mutex<Chunk>>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationStage {
    Empty,
    Shape,
    Terrain,
    Decoration,
    Full,
}

pub struct ChunkGenerator {
    thread_work_dispatcher: ThreadWorkDispatcher<ChunkGeneratorInput, ChunkGeneratorOutput>,
}

impl ChunkGenerator {
    pub fn new(num_threads: usize) -> Self {
        let thread_work_dispatcher = ThreadWorkDispatcher::new(num_threads, Self::run);

        Self { thread_work_dispatcher }
    }

    pub fn generate_chunk_to_next_stage(&mut self, chunk: Arc<Mutex<Chunk>>) {
        self.thread_work_dispatcher.dispatch_work(ChunkGeneratorInput { chunk });
    }

    pub fn idle_threads(&self) -> usize {
        self.thread_work_dispatcher.idle_threads()
    }

    fn shape(chunk: &mut Chunk) {
        let offset_x = (chunk.position.x * CHUNK_SIZE as i32) as f32;
        let offset_y = (chunk.position.y * CHUNK_SIZE as i32) as f32;
        for (chunk_part_index, part) in chunk.parts.iter_mut().enumerate() {
            let fbm = simdnoise::NoiseBuilder::fbm_3d_offset(
                offset_x,
                CHUNK_SIZE,
                offset_y,
                CHUNK_SIZE,
                (chunk_part_index * CHUNK_SIZE) as f32,
                CHUNK_SIZE
            )
            .with_octaves(4)
            .with_freq(0.05)
            .with_seed(1)
            .generate().0;
            let cobblestone_id = part.block_pallet.insert_block(BLOCK_MAP.get("cobblestone").unwrap().clone().into());
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let density = fbm[x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE];
                        if density > 0.0 {
                            part.set_block_pallet_id((x, y, z), cobblestone_id);
                        }
                    }
                }
            }
        }
        
        chunk.generation_stage = GenerationStage::Shape;
    }

    fn terrain(chunk: &mut Chunk) {
        chunk.generation_stage = GenerationStage::Terrain;
    }

    fn decoration(chunk: &mut Chunk) {
        chunk.generation_stage = GenerationStage::Decoration;
    }

    fn full(chunk: &mut Chunk) {
        chunk.generation_stage = GenerationStage::Full;
    }

    fn run(receiver: Receiver<ChunkGeneratorInput>, sender: Sender<ChunkGeneratorOutput>) {
        for generation_input in receiver.iter() {
            let mut chunk = generation_input.chunk.lock().unwrap();
            match chunk.generation_stage {
                GenerationStage::Empty => Self::shape(&mut chunk),
                GenerationStage::Shape => Self::terrain(&mut chunk),
                GenerationStage::Terrain => Self::decoration(&mut chunk),
                GenerationStage::Decoration => Self::full(&mut chunk),
                GenerationStage::Full => panic!()
            }
            chunk.generation_scheduled = false;
            sender.send(ChunkGeneratorOutput {  }).unwrap();
        }
    }

    pub fn update(&mut self) {
        self.thread_work_dispatcher.collect_outputs();
    }
}