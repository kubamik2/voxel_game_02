use std::sync::{mpsc::{Receiver, Sender}, Arc, Mutex};

use cgmath::{Vector2, Vector3};
use hashbrown::HashSet;

use crate::{block::Block, thread_work_dispatcher::ThreadWorkDispatcher, world::{structure::Structure, CHUNK_HEIGHT, PARTS_PER_CHUNK}, BLOCK_MAP};

use super::{area::Area, chunk_map::ChunkMap, chunk_part::CHUNK_SIZE, Chunk};

pub enum ChunkGeneratorInput {
    Chunk(Arc<Chunk>),
    Area(Area)
}

pub enum ChunkGeneratorOutput {
    Chunk(Arc<Chunk>),
    Area(Area)
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationStage {
    Empty,
    Shape,
    Terrain,
    Decoration,
}

impl GenerationStage {
    pub const LAST_GENERATION_STAGE: GenerationStage = GenerationStage::Decoration;
}

pub struct ChunkGenerator {
    thread_work_dispatcher: ThreadWorkDispatcher<ChunkGeneratorInput, ChunkGeneratorOutput>,
}

impl ChunkGenerator {
    pub fn new(num_threads: usize) -> Self {
        let thread_work_dispatcher = ThreadWorkDispatcher::new(num_threads, Self::run);

        Self { thread_work_dispatcher }
    }

    pub fn generate_chunk_to_next_stage(&mut self, current_stage: GenerationStage, chunk_map: &mut ChunkMap, chunk_position: Vector2<i32>, scheduled_generations: &mut HashSet<Vector2<i32>>) {
        let generation_input = match current_stage {
            GenerationStage::Empty => {
                let Some(chunk) = chunk_map.remove(chunk_position) else { return; };
                scheduled_generations.insert(chunk_position);
                ChunkGeneratorInput::Chunk(chunk)
            },
            _ => {
                let Some(area) = Area::new(chunk_map, chunk_position) else { return; };
                for z in -1..=1 {
                    for x in -1..=1 {
                        scheduled_generations.insert(chunk_position + Vector2::new(x, z));
                    }
                }
                ChunkGeneratorInput::Area(area)
            }
        };

        self.thread_work_dispatcher.dispatch_work(generation_input);
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
                        let a = (y + chunk_part_index * CHUNK_SIZE).saturating_sub(128) as f32 / CHUNK_HEIGHT as f32;
                        let density = fbm[x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE] - a.sqrt();
                        if density > 0.0 {
                            part.set_block_pallet_id(Vector3 { x, y, z }, cobblestone_id);
                        }
                    }
                }
            }
        }
        
        chunk.generation_stage = GenerationStage::Shape;
    }

    fn terrain(area: &mut Area) {
        let chunk = area.get_chunk_mut(Vector2::new(0, 0)).unwrap();
        let air: Block = BLOCK_MAP.get("air").unwrap().clone().into();
        let dirt: Block = BLOCK_MAP.get("dirt").unwrap().clone().into();
        let now = std::time::Instant::now();
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let position = Vector3::new(x, y, z);
                    if let Some(block) = chunk.get_block(position) {
                        if block.name() == "air" { continue; }
                    }

                    let air_on_top = {
                        let Some(block) = chunk.get_block(position + Vector3::unit_y()) else { continue; };
                        block.name() == "air"
                    };

                    if air_on_top {
                        chunk.set_block(position, dirt.clone());
                    }
                }
            }
        }
        area.get_chunk_mut(Vector2::new(0, 0)).unwrap().generation_stage = GenerationStage::Terrain;
        // dbg!(now.elapsed());
    }

    fn decoration(area: &mut Area) {
        area.get_chunk_mut(Vector2::new(0, 0)).unwrap().generation_stage = GenerationStage::Decoration;
    }

    fn run(receiver: Receiver<ChunkGeneratorInput>, sender: Sender<ChunkGeneratorOutput>) {
        for generation_input in receiver.iter() {
            match generation_input {
                ChunkGeneratorInput::Chunk(mut chunk) => {
                    {
                        let chunk = Arc::make_mut(&mut chunk);
                        match chunk.generation_stage {
                            GenerationStage::Empty => Self::shape(chunk),
                            _ => panic!()
                        }
                    }

                    sender.send(ChunkGeneratorOutput::Chunk(chunk)).unwrap();
                },
                ChunkGeneratorInput::Area(mut area) => {
                    match area.get_chunk(Vector2::new(0, 0)).unwrap().generation_stage {
                        GenerationStage::Shape => Self::terrain(&mut area),
                        GenerationStage::Terrain => Self::decoration(&mut area),
                        _ => panic!()
                    }

                    sender.send(ChunkGeneratorOutput::Area(area)).unwrap();
                }
            }
        }
    }

    pub fn collect_outputs(&mut self) -> Box<[ChunkGeneratorOutput]> {
        self.thread_work_dispatcher.collect_outputs()
    }

    pub fn iter_outputs<'a>(&'a mut self) -> impl Iterator<Item = ChunkGeneratorOutput> + 'a {
        self.thread_work_dispatcher.iter_outputs()
    }
}