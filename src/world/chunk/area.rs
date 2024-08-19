use std::{collections::VecDeque, fmt::Debug, mem::MaybeUninit, sync::Arc};

use cgmath::{Vector2, Vector3};
use hashbrown::{HashMap, HashSet};

use crate::{block::{light::{LightLevel, LightNode, LIGHT_LEVEL_MAX_VALUE}, Block}, world::{chunk::chunk_part::CHUNK_SIZE_I32, structure::Structure, CHUNK_HEIGHT, PARTS_PER_CHUNK}, OBSTRUCTS_LIGHT_CACHE};

use super::{chunk_map::ChunkMap, chunk_part::{ChunkPart, CHUNK_SIZE}, Chunk};

pub struct Area {
    pub chunks: [Arc<Chunk>; 9]
}

impl Debug for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("area")
        .field("center_chunk", &self.center_chunk().position)
        .finish()
    }
}

impl Area {
    #[inline]
    fn chunk_index(offset: Vector2<i32>) -> Option<usize> {
        let index = offset.x + offset.y * 3 + 4;
        if index < 0 || index > 8 { return None; }
        Some(index as usize)
    }

    #[inline]
    fn get_chunk_part_and_local_position(&self, position: Vector3<i32>) -> Option<(&ChunkPart, Vector3<usize>)> {
        let chunk_part_index = position.y.div_euclid(CHUNK_SIZE_I32);
        if chunk_part_index < 0 || chunk_part_index >= PARTS_PER_CHUNK as i32 { return None; }

        let chunk_offset = position.xz().map(|f| f.div_euclid(CHUNK_SIZE as i32));
        let Some(chunk) = self.get_chunk(chunk_offset) else { return None; };

        let local_position = position.map(|f| f.rem_euclid(CHUNK_SIZE as i32) as usize);
        let chunk_part = &chunk.parts[chunk_part_index as usize];

        Some((chunk_part, local_position))
    }

    #[inline]
    fn get_chunk_offset_and_chunk_part_index(&self, position: Vector3<i32>) -> Option<(Vector2<i32>, usize)> {
        if position.y < 0 || position.y >= CHUNK_HEIGHT as i32 { return None; }
        let chunk_offset = position.xz().map(|f| f.div_euclid(CHUNK_SIZE_I32));
        Some((chunk_offset, position.y as usize / CHUNK_SIZE))
    }

    #[inline]
    fn get_chunk_part_mut_and_local_position(&mut self, position: Vector3<i32>) -> Option<(&mut ChunkPart, Vector3<usize>)> {
        let chunk_part_index = position.y.div_euclid(CHUNK_SIZE_I32);
        if chunk_part_index < 0 || chunk_part_index >= PARTS_PER_CHUNK as i32 { return None; }

        let chunk_offset = position.xz().map(|f| f.div_euclid(CHUNK_SIZE as i32));
        let Some(chunk) = self.get_chunk_mut(chunk_offset) else { return None; };

        let local_position = position.map(|f| f.rem_euclid(CHUNK_SIZE as i32) as usize);
        let chunk_part = &mut chunk.parts[chunk_part_index as usize];

        Some((chunk_part, local_position))
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
        let Some((chunk_part, local_position)) = self.get_chunk_part_and_local_position(position) else { return None; };
        chunk_part.get_block(local_position)
    }

    #[inline]
    pub fn set_block(&mut self, position: Vector3<i32>, block: Block) {
        let Some((chunk_part, local_position)) = self.get_chunk_part_mut_and_local_position(position) else { return; };
        chunk_part.set_block(local_position.into(), block);
    }
    
    #[inline]
    pub fn get_light_level(&self, position: Vector3<i32>) -> Option<&LightLevel> {
        let Some((chunk_part, local_position)) = self.get_chunk_part_and_local_position(position) else { return None; };
        Some(&chunk_part.light_level_layers[local_position])
    }

    #[inline]
    pub fn get_sky_light_level(&self, position: Vector3<i32>) -> Option<u8> {
        let Some((chunk_part, local_position)) = self.get_chunk_part_and_local_position(position) else { return None; };
        Some(chunk_part.light_level_layers[local_position].get_sky())
    }

    #[inline]
    pub fn get_block_light_level(&self, position: Vector3<i32>) -> Option<u8> {
        let Some((chunk_part, local_position)) = self.get_chunk_part_and_local_position(position) else { return None; };
        Some(chunk_part.light_level_layers[local_position].get_block())
    }
    
    #[inline]
    pub fn set_light_level(&mut self, position: Vector3<i32>, light_level: LightLevel) {
        let Some((chunk_part, local_position)) = self.get_chunk_part_mut_and_local_position(position) else { return; };
        chunk_part.light_level_layers.set_light_level(local_position, light_level);
    }

    #[inline]
    pub fn set_sky_light_level(&mut self, position: Vector3<i32>, level: u8) {
        let Some((chunk_part, local_position)) = self.get_chunk_part_mut_and_local_position(position) else { return; };
        chunk_part.set_sky_light_level(local_position, level);
    }

    #[inline]
    pub fn set_block_light_level(&mut self, position: Vector3<i32>, level: u8) {
        let Some((chunk_part, local_position)) = self.get_chunk_part_mut_and_local_position(position) else { return; };
        chunk_part.set_block_light_level(local_position, level);
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
            self.step_block_light_node_propagation(light_node, &mut light_node_queue);
        }
    }

    pub fn propagate_sky_light(&mut self) {
        let mut light_node_queue = std::collections::VecDeque::with_capacity(CHUNK_SIZE * CHUNK_SIZE);

        let mut min_y = None;
        let chunk = self.center_chunk_mut();
        'outer: for (chunk_part_index, chunk_part ) in chunk.parts.iter().enumerate().rev() {
            let y_offset = chunk_part_index * CHUNK_SIZE;
            for (i, layer) in chunk_part.block_layers.0.iter().enumerate().rev() {
                match layer {
                    crate::world::chunk::chunk_part::BlockLayer::Compressed(block_pallet_id) => {
                        let block = &chunk_part.block_pallet.get(block_pallet_id).unwrap().block;
                        if OBSTRUCTS_LIGHT_CACHE.get(*block.id() as usize) { break 'outer; }
                        min_y = Some(i + y_offset);
                    },
                    _ => break 'outer
                }
            }
        }
        if let Some(min_y) = min_y {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    light_node_queue.push_back(LightNode::new(x as i8 + 1, min_y as i16, z as i8, 14));
                    light_node_queue.push_back(LightNode::new(x as i8, min_y as i16, z as i8 + 1, 14));
                    light_node_queue.push_back(LightNode::new(x as i8 - 1, min_y as i16, z as i8, 14));
                    light_node_queue.push_back(LightNode::new(x as i8, min_y as i16 - 1, z as i8, 15));
                    light_node_queue.push_back(LightNode::new(x as i8, min_y as i16, z as i8 - 1, 14));
                }
            }

            for y in min_y..CHUNK_HEIGHT {
                let chunk_part_index = y / CHUNK_SIZE; // can just divide because y > 0
                let chunk_part = &mut chunk.parts[chunk_part_index];
                let local_y = y % CHUNK_SIZE; // can non euclid rem y > 0
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        chunk_part.light_level_layers.set_light_level(Vector3 { x, y: local_y, z }, LightLevel::new(0, 15).unwrap());
                    }
                }
            }
        }

        while let Some(light_node) = light_node_queue.pop_front() {
            self.step_sky_light_node_propagation(light_node, &mut light_node_queue);
        }
    }

    #[inline]
    fn step_sky_light_node_propagation(&mut self, light_node: LightNode, light_node_queue: &mut VecDeque<LightNode>) {
        let position = Vector3 { x: light_node.x as i32, y: light_node.y as i32, z: light_node.z as i32 };
        let Some(block) = self.get_block(position) else { return; };

        let obstructs_light = OBSTRUCTS_LIGHT_CACHE.get(*block.id() as usize);
        if obstructs_light { return; }

        let Some(current_light_level) = self.get_sky_light_level(position) else { return; };
        if light_node.level <= current_light_level { return; }

        self.set_sky_light_level(position, light_node.level);
        if light_node.level == 1 { return; }
        let new_level = light_node.level - 1;
        let downwards_level = light_node.level - (light_node.level != LIGHT_LEVEL_MAX_VALUE) as u8;

        light_node_queue.extend([
            LightNode::new(light_node.x + 1, light_node.y, light_node.z, new_level),
            LightNode::new(light_node.x, light_node.y + 1, light_node.z, new_level),
            LightNode::new(light_node.x, light_node.y, light_node.z + 1, new_level),
            LightNode::new(light_node.x - 1, light_node.y, light_node.z, new_level),
            LightNode::new(light_node.x, light_node.y - 1, light_node.z, downwards_level),
            LightNode::new(light_node.x, light_node.y, light_node.z - 1, new_level),
        ]);
    }


    #[inline]
    fn step_block_light_node_propagation(&mut self, light_node: LightNode, light_node_queue: &mut VecDeque<LightNode>) {
        let position = Vector3 { x: light_node.x as i32, y: light_node.y as i32, z: light_node.z as i32 };
        let Some(block) = self.get_block(position) else { return; };

        let obstructs_light = OBSTRUCTS_LIGHT_CACHE.get(*block.id() as usize);
        if obstructs_light { return; }

        let Some(current_light_level) = self.get_block_light_level(position) else { return; };
        if light_node.level <= current_light_level { return; }

        self.set_block_light_level(position, light_node.level);
        if light_node.level == 1 { return; }
        let new_level = light_node.level - 1;

        light_node_queue.extend([
            LightNode::new(light_node.x + 1, light_node.y, light_node.z, new_level),
            LightNode::new(light_node.x, light_node.y + 1, light_node.z, new_level),
            LightNode::new(light_node.x, light_node.y, light_node.z + 1, new_level),
            LightNode::new(light_node.x - 1, light_node.y, light_node.z, new_level),
            LightNode::new(light_node.x, light_node.y - 1, light_node.z, new_level),
            LightNode::new(light_node.x, light_node.y, light_node.z - 1, new_level),
        ]);
    }

    #[inline]
    pub fn update_sky_light_at(&mut self, position: Vector3<i32>, afflicted_chunk_parts: &mut HashSet<(Vector2<i32>, usize)>) {
        let mut light_node_removal_queue: VecDeque<LightNode> = VecDeque::new();
        let mut light_node_propagation_queue: VecDeque<LightNode> = VecDeque::new();
        
        let Some(level) = self.get_sky_light_level(position) else { return; };
        light_node_removal_queue.push_back(LightNode::new(position.x as i8, position.y as i16, position.z as i8, level));
        self.set_sky_light_level(position, 0);

        #[inline]
        fn remove_in_direction(area: &mut Area, direction: Vector3<i32>, light_node_position: Vector3<i32>, light_node_level: u8, light_node_removal_queue: &mut VecDeque<LightNode>, light_node_propagation_queue: &mut VecDeque<LightNode>) {
            let neighbor_position= light_node_position + direction;
            let Some(neighbor_sky_light_level) = area.get_sky_light_level(neighbor_position) else { return; };
            if neighbor_sky_light_level == 0 { return; }
            if neighbor_sky_light_level < light_node_level {
                area.set_sky_light_level(neighbor_position, 0);
                light_node_removal_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level));
            } else if neighbor_sky_light_level >= light_node_level {
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8 + 1, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8 + 1, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8 - 1, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8 - 1, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16 + 1, neighbor_position.z as i8, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16 - 1, neighbor_position.z as i8, neighbor_sky_light_level - (neighbor_sky_light_level != LIGHT_LEVEL_MAX_VALUE) as u8));
            }
        }

        #[inline]
        fn remove_downwards(area: &mut Area, light_node_position: Vector3<i32>, light_node_level: u8, light_node_removal_queue: &mut VecDeque<LightNode>, light_node_propagation_queue: &mut VecDeque<LightNode>) {
            let neighbor_position= light_node_position + Vector3::new(0, -1, 0);
            let Some(neighbor_sky_light_level) = area.get_sky_light_level(neighbor_position) else { return; };
            if neighbor_sky_light_level == 0 { return; }
            if neighbor_sky_light_level == LIGHT_LEVEL_MAX_VALUE {
                area.set_sky_light_level(neighbor_position, 0);
                light_node_removal_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level));
            } else if neighbor_sky_light_level < light_node_level {
                area.set_sky_light_level(neighbor_position, 0);
                light_node_removal_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level));
            } else if neighbor_sky_light_level >= light_node_level {
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8 + 1, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8 + 1, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8 - 1, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8 - 1, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16 - 1, neighbor_position.z as i8, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16 + 1, neighbor_position.z as i8, neighbor_sky_light_level - 1));
            }
        }

        while let Some(light_node) = light_node_removal_queue.pop_front() {
            let light_node_position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            let Some((chunk_offset, chunk_part_index)) = self.get_chunk_offset_and_chunk_part_index(light_node_position) else { continue; };
            let Some(chunk) = self.get_chunk(chunk_offset) else { continue; };
            afflicted_chunk_parts.insert((chunk.position, chunk_part_index));

            remove_downwards(self, light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(1, 0, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, 1, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, 0, 1), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(-1, 0, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, 0, -1), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
        }

        let mut visited_nodes = HashMap::new();
        while let Some(light_node) = light_node_propagation_queue.pop_front() {
            let light_node_position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            match visited_nodes.entry(((light_node.x as u8 as u32)) | ((light_node.y as u16 as u32) << 8) | ((light_node.z as u8 as u32) << 24)) {
                hashbrown::hash_map::Entry::Occupied(occupied) => {
                    let level = *occupied.get();
                    if light_node.level <= level { continue; }
                },
                hashbrown::hash_map::Entry::Vacant(vacant) => {
                    vacant.insert(light_node.level);
                }
            }
            let Some((chunk_offset, chunk_part_index)) = self.get_chunk_offset_and_chunk_part_index(light_node_position) else { continue; };
            let Some(chunk) = self.get_chunk(chunk_offset) else { continue; };
            afflicted_chunk_parts.insert((chunk.position, chunk_part_index));

            self.step_sky_light_node_propagation(light_node, &mut light_node_propagation_queue);
        }
    }

    #[inline]
    pub fn remove_block_light_at(&mut self, position: Vector3<i32>, afflicted_chunk_parts: &mut HashSet<(Vector2<i32>, usize)>) {
        let mut light_node_removal_queue: VecDeque<LightNode> = VecDeque::new();
        let mut light_node_propagation_queue: VecDeque<LightNode> = VecDeque::new();
        
        let Some(level) = self.get_block_light_level(position) else { return; };
        light_node_removal_queue.push_back(LightNode::new(position.x as i8, position.y as i16, position.z as i8, level));
        self.set_block_light_level(position, 0);

        #[inline]
        fn remove_in_direction(area: &mut Area, direction: Vector3<i32>, light_node_position: Vector3<i32>, light_node_level: u8, light_node_removal_queue: &mut VecDeque<LightNode>, light_node_propagation_queue: &mut VecDeque<LightNode>) {
            let neighbor_position= light_node_position + direction;
            let Some(neighbor_sky_light_level) = area.get_block_light_level(neighbor_position) else { return; };
            if neighbor_sky_light_level < light_node_level && neighbor_sky_light_level != 0 {
                area.set_block_light_level(neighbor_position, 0);
                light_node_removal_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level));
            } else if neighbor_sky_light_level >= light_node_level {
                if neighbor_sky_light_level == 0 { return; }
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8 + 1, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8 + 1, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8 - 1, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8 - 1, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16 + 1, neighbor_position.z as i8, neighbor_sky_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16 - 1, neighbor_position.z as i8, neighbor_sky_light_level - 1));
            }
        }

        while let Some(light_node) = light_node_removal_queue.pop_front() {
            let light_node_position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            let Some((chunk_offset, chunk_part_index)) = self.get_chunk_offset_and_chunk_part_index(light_node_position) else { continue; };
            let Some(chunk) = self.get_chunk(chunk_offset) else { continue; };
            afflicted_chunk_parts.insert((chunk.position, chunk_part_index));

            remove_in_direction(self, Vector3::new(1, 0, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, 1, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, 0, 1), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(-1, 0, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, -1, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, 0, -1), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
        }

        let mut visited_nodes = HashMap::new();
        while let Some(light_node) = light_node_propagation_queue.pop_front() {
            let light_node_position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            match visited_nodes.entry(((light_node.x as u8 as u32)) | ((light_node.y as u16 as u32) << 8) | ((light_node.z as u8 as u32) << 24)) {
                hashbrown::hash_map::Entry::Occupied(occupied) => {
                    let level = *occupied.get();
                    if light_node.level <= level { continue; }
                },
                hashbrown::hash_map::Entry::Vacant(vacant) => {
                    vacant.insert(light_node.level);
                }
            }
            let Some((chunk_offset, chunk_part_index)) = self.get_chunk_offset_and_chunk_part_index(light_node_position) else { continue; };
            let Some(chunk) = self.get_chunk(chunk_offset) else { continue; };
            afflicted_chunk_parts.insert((chunk.position, chunk_part_index));

            self.step_block_light_node_propagation(light_node, &mut light_node_propagation_queue);
        }
    }

    pub fn propagate_block_light_at(&mut self, position: Vector3<i32>, afflicted_chunk_parts: &mut HashSet<(Vector2<i32>, usize)>) {
        let mut light_node_queue = VecDeque::new();
        let Some(block_level) = self.get_block_light_level(position) else { return; };
        if block_level <= 1 { return; }
        light_node_queue.push_back(LightNode::new(position.x as i8 + 1, position.y as i16, position.z as i8, block_level - 1));
        light_node_queue.push_back(LightNode::new(position.x as i8, position.y as i16 + 1, position.z as i8, block_level - 1));
        light_node_queue.push_back(LightNode::new(position.x as i8, position.y as i16, position.z as i8 + 1, block_level - 1));
        light_node_queue.push_back(LightNode::new(position.x as i8 - 1, position.y as i16, position.z as i8, block_level - 1));
        light_node_queue.push_back(LightNode::new(position.x as i8, position.y as i16 - 1, position.z as i8, block_level - 1));
        light_node_queue.push_back(LightNode::new(position.x as i8, position.y as i16, position.z as i8 - 1, block_level - 1));

        while let Some(light_node) = light_node_queue.pop_front() {
            self.step_block_light_node_propagation(light_node, &mut light_node_queue);
            let light_node_position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            let Some((chunk_offset, chunk_part_index)) = self.get_chunk_offset_and_chunk_part_index(light_node_position) else { continue; };
            let Some(chunk) = self.get_chunk(chunk_offset) else { continue; };
            afflicted_chunk_parts.insert((chunk.position, chunk_part_index));
        }
    }

    #[inline]
    pub fn update_block_light_at(&mut self, position: Vector3<i32>, afflicted_chunk_parts: &mut HashSet<(Vector2<i32>, usize)>) {
        let mut light_node_removal_queue: VecDeque<LightNode> = VecDeque::new();
        let mut light_node_propagation_queue: VecDeque<LightNode> = VecDeque::new();
        
        let Some(level) = self.get_block_light_level(position) else { return; };
        light_node_removal_queue.push_back(LightNode::new(position.x as i8, position.y as i16, position.z as i8, level));

        #[inline]
        fn remove_in_direction(area: &mut Area, direction: Vector3<i32>, light_node_position: Vector3<i32>, light_node_level: u8, light_node_removal_queue: &mut VecDeque<LightNode>, light_node_propagation_queue: &mut VecDeque<LightNode>) {
            let neighbor_position= light_node_position + direction;
            let Some(neighbor_block_light_level) = area.get_block_light_level(neighbor_position) else { return; };
            if neighbor_block_light_level < light_node_level && neighbor_block_light_level != 0 {
                area.set_block_light_level(neighbor_position, 0);
                light_node_removal_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_block_light_level));
            } else if neighbor_block_light_level >= light_node_level {
                if neighbor_block_light_level == 0 { return; }
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8 + 1, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_block_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8 + 1, neighbor_block_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8 - 1, neighbor_position.y as i16, neighbor_position.z as i8, neighbor_block_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16, neighbor_position.z as i8 - 1, neighbor_block_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16 + 1, neighbor_position.z as i8, neighbor_block_light_level - 1));
                light_node_propagation_queue.push_back(LightNode::new(neighbor_position.x as i8, neighbor_position.y as i16 - 1, neighbor_position.z as i8, neighbor_block_light_level - 1));
            }
        }

        while let Some(light_node) = light_node_removal_queue.pop_front() {
            let light_node_position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            let Some((chunk_offset, chunk_part_index)) = self.get_chunk_offset_and_chunk_part_index(light_node_position) else { continue; };
            let Some(chunk) = self.get_chunk(chunk_offset) else { continue; };
            afflicted_chunk_parts.insert((chunk.position, chunk_part_index));

            remove_in_direction(self, Vector3::new(1, 0, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, 1, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, 0, 1), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(-1, 0, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, -1, 0), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
            remove_in_direction(self, Vector3::new(0, 0, -1), light_node_position, light_node.level, &mut light_node_removal_queue, &mut light_node_propagation_queue);
        }

        let mut visited_nodes = HashMap::new();
        while let Some(light_node) = light_node_propagation_queue.pop_front() {
            let light_node_position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            match visited_nodes.entry(((light_node.x as u8 as u32)) | ((light_node.y as u16 as u32) << 8) | ((light_node.z as u8 as u32) << 24)) {
                hashbrown::hash_map::Entry::Occupied(occupied) => {
                    let level = *occupied.get();
                    if light_node.level <= level { continue; }
                },
                hashbrown::hash_map::Entry::Vacant(vacant) => {
                    vacant.insert(light_node.level);
                }
            }
            let Some((chunk_offset, chunk_part_index)) = self.get_chunk_offset_and_chunk_part_index(light_node_position) else { continue; };
            let Some(chunk) = self.get_chunk(chunk_offset) else { continue; };
            afflicted_chunk_parts.insert((chunk.position, chunk_part_index));

            self.step_block_light_node_propagation(light_node, &mut light_node_propagation_queue);
        }
    }
}