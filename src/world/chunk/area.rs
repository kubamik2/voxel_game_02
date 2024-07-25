use std::{mem::MaybeUninit, sync::Arc};

use cgmath::{Vector2, Vector3};
use hashbrown::HashMap;

use crate::{block::{light::{LightLevel, LightNode}, Block}, world::{structure::Structure, PARTS_PER_CHUNK}, BLOCK_LIST, BLOCK_MAP, OBSTRUCTS_LIGHT_CACHE};

use super::{chunk_map::ChunkMap, chunk_part::CHUNK_SIZE, Chunk};

pub struct Area {
    pub chunks: [Arc<Chunk>; 9]
}

impl Area {
    #[inline]
    fn chunk_index(offset: Vector2<i32>) -> Option<usize> {
        let index = offset.x + offset.y * 3 + 4;
        if index < 0 || index > 8 { return None; }
        Some(index as usize)
    }

    #[inline]
    pub fn get_chunk(&self, offset: Vector2<i32>) -> Option<&Chunk> {
        let Some(index) = Self::chunk_index(offset) else { return None; };
        Some(&self.chunks[index])
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, offset: Vector2<i32>) -> Option<&mut Chunk>{
        let Some(index) = Self::chunk_index(offset) else { return None; };
        let chunk = &mut self.chunks[index];

        assert!(Arc::strong_count(chunk) == 1);

        let chunk_mut = Arc::make_mut(chunk);
        Some(chunk_mut)
    }

    #[inline]
    pub fn get_block(&self, position: Vector3<i32>) -> Option<&Block> {
        let chunk_part_index = position.y / CHUNK_SIZE as i32;
        if chunk_part_index < 0 || chunk_part_index >= PARTS_PER_CHUNK as i32 { return None; }

        let chunk_offset = position.xz().map(|f| f.div_euclid(CHUNK_SIZE as i32));
        let Some(chunk) = self.get_chunk(chunk_offset) else { return None; };

        let position_in_chunk_part = position.map(|f| f.rem_euclid(CHUNK_SIZE as i32) as usize);
        let chunk_part = &chunk.parts[chunk_part_index as usize];
        chunk_part.get_block(position_in_chunk_part)
    }

    #[inline]
    pub fn set_block(&mut self, position: Vector3<i32>, block: Block) {
        let chunk_part_index = position.y / CHUNK_SIZE as i32;
        if chunk_part_index < 0 || chunk_part_index >= PARTS_PER_CHUNK as i32 { return; }

        let chunk_offset = position.xz().map(|f| f.div_euclid(CHUNK_SIZE as i32));
        let Some(chunk) = self.get_chunk_mut(chunk_offset) else { return; };

        let position_in_chunk = position.map(|f| f.rem_euclid(CHUNK_SIZE as i32) as usize);
        let chunk_part = &mut chunk.parts[chunk_part_index as usize];

        chunk_part.set_block(position_in_chunk.into(), block);
    }

    pub fn insert_structure(&mut self, structure: &Structure, origin_point: Vector3<i32>) {
        for (position, block) in structure.blocks().iter().cloned() {
            self.set_block(position + origin_point, block);
        }
    }

    pub fn new(chunk_map: &mut ChunkMap, center_chunk_position: Vector2<i32>) -> Option<Self> {
        for z in -1..=1 {
            for x in -1..=1 {
                if !chunk_map.contains_key(center_chunk_position + Vector2::new(x, z)) { return None; }
            }
        }
        
        let mut chunks: [MaybeUninit<Arc<Chunk>>; 9] = std::array::from_fn(|_| MaybeUninit::uninit());
        let mut index = 0;
        for z in -1..=1 {
            for x in -1..=1 {
                chunks[index] = MaybeUninit::new(chunk_map.remove(center_chunk_position + Vector2::new(x, z)).unwrap());
                index += 1;
            }
        }

        let chunks = unsafe { std::mem::transmute::<_, [Arc<Chunk>; 9]>(chunks) };

        Some(Self { chunks })
    }

    #[inline]
    pub fn center_chunk(&self) -> &Chunk {
        &self.chunks[4]
    }
    
    #[inline]
    pub fn center_chunk_mut(&mut self) -> &mut Chunk {
        assert!(Arc::strong_count(&self.chunks[4]) == 1);
        Arc::make_mut(&mut self.chunks[4])
    }
    
    #[inline]
    pub fn propagate_block_light_in_chunk_part(&mut self, chunk_part_index: usize) {
        assert!(chunk_part_index < PARTS_PER_CHUNK);

        let mut light_node_queue = std::collections::VecDeque::new();
        light_node_queue.extend(self.get_chunk(Vector2::new(0, 0)).unwrap().parts[chunk_part_index].light_emitters.iter().cloned());

        while let Some(light_node) = light_node_queue.pop_front() {
            let light_node_position = Vector3::new(light_node.x as i16, light_node.y as i16, light_node.z as i16);
            let chunk_part_index_offset = light_node_position.y.div_euclid(CHUNK_SIZE as i16);
            let chunk_part_index = chunk_part_index as i16 + chunk_part_index_offset;
            if chunk_part_index.is_negative() || chunk_part_index >= PARTS_PER_CHUNK as i16 { continue; }

            let chunk_offset = light_node_position.xz().map(|f| f.div_euclid(CHUNK_SIZE as i16) as i32);
            let local_position = light_node_position.map(|f| f.rem_euclid(CHUNK_SIZE as i16) as usize);

            let chunk = self.get_chunk_mut(chunk_offset).unwrap();
            let chunk_part = &mut chunk.parts[chunk_part_index as usize];
            let Some(block) = chunk_part.get_block(local_position) else { continue; };

            let obstructs_light = OBSTRUCTS_LIGHT_CACHE[*block.id() as usize];
            if obstructs_light { continue; }

            let mut current_light_level = *chunk_part.light_level_layers.get_light_level(local_position);
            if light_node.level <= current_light_level.get_block() { continue; }

            current_light_level.set_block(light_node.level);
            chunk_part.light_level_layers.set_light_level(local_position, current_light_level);
            if light_node.level == 1 { continue; }
            let new_level = light_node.level - 1;

            light_node_queue.extend([
                LightNode::new(light_node.x + 1, light_node.y, light_node.z, new_level),
                LightNode::new(light_node.x, light_node.y + 1, light_node.z, new_level),
                LightNode::new(light_node.x, light_node.y, light_node.z + 1, new_level),
                LightNode::new(light_node.x - 1, light_node.y, light_node.z, new_level),
                LightNode::new(light_node.x, light_node.y - 1, light_node.z, light_node.level),
                LightNode::new(light_node.x, light_node.y, light_node.z - 1, new_level),
            ]);
        }
    }
    
    const SKY_LIGHT_NODES: [LightNode; CHUNK_SIZE * CHUNK_SIZE] = {
        let mut arr = [MaybeUninit::uninit(); CHUNK_SIZE * CHUNK_SIZE];
        let mut i = 0;
        while i < CHUNK_SIZE * CHUNK_SIZE {
            let x = (i % CHUNK_SIZE) as i8;
            let z = (i / CHUNK_SIZE) as i8;
            arr[i] = MaybeUninit::new(LightNode { x, y: CHUNK_SIZE as i16 - 1, z, level: 15 });
            i += 1;
        }

        unsafe { std::mem::transmute::<_, [LightNode; CHUNK_SIZE * CHUNK_SIZE]>(arr) }
    };

    #[inline]
    pub fn propagate_sky_light(&mut self) {
        let chunk_part_index = PARTS_PER_CHUNK - 1;
        let mut light_node_queue = std::collections::VecDeque::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
        light_node_queue.extend(Self::SKY_LIGHT_NODES);

        while let Some(light_node) = light_node_queue.pop_front() {
            let light_node_position = Vector3::new(light_node.x as i16, light_node.y as i16, light_node.z as i16);
            let chunk_part_index_offset = light_node_position.y.div_euclid(CHUNK_SIZE as i16);
            let chunk_part_index = chunk_part_index as i16 + chunk_part_index_offset;
            if chunk_part_index.is_negative() || chunk_part_index >= PARTS_PER_CHUNK as i16 { continue; }

            let chunk_offset = light_node_position.xz().map(|f| f.div_euclid(CHUNK_SIZE as i16) as i32);
            let local_position = light_node_position.map(|f| f.rem_euclid(CHUNK_SIZE as i16) as usize);

            let chunk = self.get_chunk_mut(chunk_offset).unwrap();
            let chunk_part = &mut chunk.parts[chunk_part_index as usize];
            
            let Some(block) = chunk_part.get_block(local_position) else { continue; };

            let obstructs_light = OBSTRUCTS_LIGHT_CACHE[*block.id() as usize];
            if obstructs_light { continue; }

            let mut current_light_level = *chunk_part.light_level_layers.get_light_level(local_position);
            if light_node.level <= current_light_level.get_sky() { continue; }

            current_light_level.set_sky(light_node.level);
            chunk_part.light_level_layers.set_light_level(local_position, current_light_level);
            if light_node.level == 1 { continue; }
            let new_level = light_node.level - 1;

            light_node_queue.extend([
                LightNode::new(light_node.x + 1, light_node.y, light_node.z, new_level),
                LightNode::new(light_node.x, light_node.y + 1, light_node.z, new_level),
                LightNode::new(light_node.x, light_node.y, light_node.z + 1, new_level),
                LightNode::new(light_node.x - 1, light_node.y, light_node.z, new_level),
                LightNode::new(light_node.x, light_node.y - 1, light_node.z, light_node.level),
                LightNode::new(light_node.x, light_node.y, light_node.z - 1, new_level),
            ]);
        }
    }
}