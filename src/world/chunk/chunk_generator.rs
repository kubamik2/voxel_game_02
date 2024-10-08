use std::sync::{mpsc::{Receiver, Sender}, Arc, Mutex};

use cgmath::{Vector2, Vector3};
use hashbrown::HashSet;

use crate::{block::Block, chunk_position::ChunkPosition, thread_work_dispatcher::ThreadWorkDispatcher, world::{CHUNK_HEIGHT, PARTS_PER_CHUNK}, BLOCK_MAP, STRUCTURES};

use super::{chunk_map::ChunkMap, chunk_part::{chunk_part_position::ChunkPartPosition, CHUNK_SIZE, CHUNK_SIZE_U32}, chunks3x3::Chunks3x3, Chunk};

lazy_static::lazy_static! {
    static ref DBG: Arc<Mutex<(usize, std::time::Duration, std::time::Duration, std::time::Duration)>> = Arc::new(Mutex::new((0, std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::MAX)));
}

#[derive(Debug)]
pub enum ChunkGeneratorInput {
    Chunk(Arc<Chunk>),
    Chunks3x3(Chunks3x3)
}

pub enum ChunkGeneratorOutput {
    Chunk(Arc<Chunk>),
    Chunks3x3(Chunks3x3)
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum GenerationStage {
    Empty,
    Shape,
    Terrain,
    Decoration,
    Light,
}

impl GenerationStage {
    pub const LAST_GENERATION_STAGE: GenerationStage = GenerationStage::Light;
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
        #[inline]
        fn create_input_area(chunk_map: &mut ChunkMap, chunk_position: Vector2<i32>, scheduled_generations: &mut HashSet<Vector2<i32>>) -> Option<ChunkGeneratorInput> {
            let Some(chunks3x3) = Chunks3x3::new(chunk_map, chunk_position) else { return None; };
            for z in -1..=1 {
                for x in -1..=1 {
                    scheduled_generations.insert(chunk_position + Vector2::new(x, z));
                }
            }
            Some(ChunkGeneratorInput::Chunks3x3(chunks3x3))
        }

        #[inline]
        fn create_input_chunk(chunk_map: &mut ChunkMap, chunk_position: Vector2<i32>, scheduled_generations: &mut HashSet<Vector2<i32>>) -> Option<ChunkGeneratorInput> {
            let Some(chunk) = chunk_map.remove(chunk_position) else { return None; };
            scheduled_generations.insert(chunk_position);
            Some(ChunkGeneratorInput::Chunk(chunk))
        }

        let Some(generation_input) = (match current_stage {
            GenerationStage::Empty => create_input_chunk(chunk_map, chunk_position, scheduled_generations),
            GenerationStage::Shape => create_input_area(chunk_map, chunk_position, scheduled_generations),
            GenerationStage::Terrain => create_input_area(chunk_map, chunk_position, scheduled_generations),
            GenerationStage::Decoration => create_input_area(chunk_map, chunk_position, scheduled_generations),
            GenerationStage::Light => create_input_area(chunk_map, chunk_position, scheduled_generations),
        }) else { return; };

        self.thread_work_dispatcher.dispatch_work(generation_input).expect("chunk_generator.generate_chunk_to_next_stage thread_work_dispatcher.dispatch_work failed");
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
            let stone_id = part.block_pallet.insert_block(BLOCK_MAP.get("stone").unwrap().clone().into());
            for y in 0..CHUNK_SIZE_U32 {
                for z in 0..CHUNK_SIZE_U32 {
                    for x in 0..CHUNK_SIZE_U32 {
                        let a = (y as usize + chunk_part_index * CHUNK_SIZE).saturating_sub(200) as f32 / CHUNK_HEIGHT as f32;
                        let density = fbm[x as usize + z as usize * CHUNK_SIZE + y as usize * CHUNK_SIZE * CHUNK_SIZE] - a.sqrt();
                        if density > 0.0 {
                            let position = unsafe { ChunkPartPosition::new_unchecked(Vector3 { x, y, z }) };
                            part.set_block_pallet_id(position, stone_id);
                            let highest_block_position = &mut chunk.highest_blocks[Vector2::new(x as u8, z as u8)];
                            if (chunk_part_index as u8 > highest_block_position.chunk_part_index)
                            || (chunk_part_index as u8 == highest_block_position.chunk_part_index && y as u8 > highest_block_position.y) {
                                highest_block_position.chunk_part_index = chunk_part_index as u8;
                                highest_block_position.y = y as u8;
                            }
                        }
                    }
                }
            }
        }
        chunk.maintain_parts();
        chunk.generation_stage = GenerationStage::Shape;
    }

    fn terrain(chunks3x3: &mut Chunks3x3) {
        let center_chunk = chunks3x3.get_chunk_mut(Vector2::new(0, 0)).unwrap();
        let grass: Block = BLOCK_MAP.get("grass").unwrap().clone().into();
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let position = ChunkPosition::try_from(Vector3::new(x as u32, y as u32, z as u32)).unwrap();
                    if center_chunk.get_block(position).name() == "air" { continue; }

                    let air_on_top = {
                        if let Some(position) = position.checked_add_u32(Vector3::unit_y()) {
                            let block = center_chunk.get_block(position);
                            block.name() == "air"
                        } else {
                            false
                        }
                    };

                    if air_on_top {
                        center_chunk.set_block(position, grass.clone());
                    }
                }
            }
        }

        center_chunk.generation_stage = GenerationStage::Terrain;
    }

    fn decoration(chunks3x3: &mut Chunks3x3) {
        let center_chunk = chunks3x3.get_chunk(Vector2::new(0, 0)).unwrap();
        let offset_x = (center_chunk.position.x * CHUNK_SIZE as i32) as f32;
        let offset_y = (center_chunk.position.y * CHUNK_SIZE as i32) as f32;
        let fbm = simdnoise::NoiseBuilder::fbm_2d_offset(
            offset_x,
            CHUNK_SIZE,
            offset_y,
            CHUNK_SIZE,
        )
        .with_octaves(2)
        .with_freq(10.5)
        .with_seed(2)
        .generate().0;
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if fbm[x + z * CHUNK_SIZE] > 0.06 {
                    let highest_block_position = chunks3x3.center_chunk().highest_blocks[Vector2::new(x as u8, z as u8)];
                    let highest_y = (highest_block_position.y as usize + highest_block_position.chunk_part_index as usize * CHUNK_SIZE) as i32;
                    let tree = STRUCTURES.get("tree").unwrap();

                    chunks3x3.insert_structure(tree, Vector3::new(x as i32, highest_y + 1, z as i32));
                }
            }
        }
        chunks3x3.center_chunk_mut().generation_stage = GenerationStage::Decoration;
    }

    fn light_emit(chunks3x3: &mut Chunks3x3) {
        let now = std::time::Instant::now();
        chunks3x3.propagate_sky_light();
        // for chunk_part_index in 0..PARTS_PER_CHUNK {
        //     chunks3x3.propagate_block_light_in_chunk_part(chunk_part_index);
        // }
        // let elapsed = now.elapsed();
        // let mut dbg = DBG.lock().unwrap();
        // dbg.0 += 1;
        // dbg.1 += elapsed;
        // if dbg.2 < elapsed {
        //     dbg.2 = elapsed;
        // }
        // if dbg.3 > elapsed {
        //     dbg.3 = elapsed;
        // }
        // println!("num: {: <5} sum: {: <8.2?} avg: {: <8.2?} max: {: <8.2?} min: {: <8.2?}", dbg.0, dbg.1, dbg.1 / dbg.0 as u32, dbg.2, dbg.3);
        chunks3x3.get_chunk_mut(Vector2::new(0, 0)).unwrap().generation_stage = GenerationStage::Light;
    }

    fn run(receiver: Receiver<ChunkGeneratorInput>, sender: Sender<ChunkGeneratorOutput>) {
        for generation_input in receiver.iter() {
            match generation_input {
                ChunkGeneratorInput::Chunk(mut chunk) => {
                    {
                        let chunk = Arc::make_mut(&mut chunk);
                        match chunk.generation_stage {
                            GenerationStage::Empty => Self::shape(chunk),
                            _ => panic!("invalid gen input")
                        }
                    }

                    sender.send(ChunkGeneratorOutput::Chunk(chunk)).unwrap();
                },
                ChunkGeneratorInput::Chunks3x3(mut area) => {
                    match area.get_chunk(Vector2::new(0, 0)).unwrap().generation_stage {
                        GenerationStage::Shape => Self::terrain(&mut area),
                        GenerationStage::Terrain => Self::decoration(&mut area),
                        GenerationStage::Decoration => Self::light_emit(&mut area),
                        _ => panic!("invalid gen input")
                    }

                    sender.send(ChunkGeneratorOutput::Chunks3x3(area)).unwrap();
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
