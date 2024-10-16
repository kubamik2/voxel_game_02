use std::{collections::VecDeque, fmt::Debug, mem::MaybeUninit, sync::Arc};

use cgmath::{num_traits::Euclid, Vector2, Vector3};
use hashbrown::{HashMap, HashSet};

use crate::{block::{light::{LightLevel, LIGHT_LEVEL_MAX_VALUE}, Block, FaceDirection}, chunk_position::ChunkPosition, global_vector::GlobalVecU, world::{chunk::chunk_part::CHUNK_SIZE_I32, structure::Structure, CHUNK_HEIGHT, PARTS_PER_CHUNK}, BLOCK_LIST};

use super::{chunk_map::{ChunkMapLock, ChunkMap}, chunk_part::{chunk_part_position::ChunkPartPosition, ChunkPart, CHUNK_SIZE, CHUNK_SIZE_U32}, Chunk};

pub struct Chunks3x3 {
    pub chunks: Box<[Chunk; 9]>,
}

impl Debug for Chunks3x3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("chunks3x3")
        .field("center_chunk", &self.center_chunk().position)
        .finish()
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct LightNode {
    x: i8,
    z: i8,
    y: i16,
}

impl LightNode {
    pub const fn new(position: Vector3<i32>) -> Self {
        Self {
            x: position.x as i8,
            y: position.y as i16,
            z: position.z as i8,
        }
    }
}

impl Chunks3x3 {
    #[inline]
    const fn chunk_index(offset: Vector2<i32>) -> Option<usize> {
        let index = offset.x + offset.y * 3 + 4;
        if index < 0 || index > 8 { return None; }
        Some(index as usize)
    }

    #[inline]
    pub fn get_chunk(&self, offset: Vector2<i32>) -> Option<&Chunk> {
        let index = Self::chunk_index(offset)?;
        Some(&self.chunks[index])
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, offset: Vector2<i32>) -> Option<&mut Chunk>{
        let index = Self::chunk_index(offset)?;
        let chunk = &mut self.chunks[index];

        // assert!(Arc::strong_count(chunk) == 1);

        // let chunk_mut = Arc::make_mut(chunk);
        // Some(chunk_mut)
        Some(chunk)
    }

    #[inline]
    fn get_chunk_and_chunk_position(&self, position: Vector3<i32>) -> Option<(&Chunk, ChunkPosition)> {
        if position.y < 0 || position.y > CHUNK_HEIGHT as i32 { return None; }

        let chunk_offset = Vector2::new(
            (position.x & !(0b11111_i32)) >> 5,
            (position.z & !(0b11111_i32)) >> 5,
        );
        let chunk = self.get_chunk(chunk_offset)?;
        let chunk_part_index = ((position.y & !(0b11111_i32)) >> 5) as u32;

        let chunk_part_position = Vector3::new(
            (position.x & 0b11111) as u32,
            (position.y & 0b11111) as u32,
            (position.z & 0b11111) as u32,
        );

        Some((chunk, unsafe { ChunkPosition::new_unchecked(chunk_part_position, chunk_part_index as usize) }))
    }

    #[inline]
    fn get_chunk_mut_and_chunk_position(&mut self, position: Vector3<i32>) -> Option<(&mut Chunk, ChunkPosition)> {
        if position.y < 0 || position.y > CHUNK_HEIGHT as i32 { return None; }

        let chunk_offset = Vector2::new(
            (position.x & !(0b11111_i32)) >> 5,
            (position.z & !(0b11111_i32)) >> 5,
        );
        let chunk = self.get_chunk_mut(chunk_offset)?;
        let chunk_part_index = ((position.y & !(0b11111_i32)) >> 5) as u32;

        let chunk_part_position = Vector3::new(
            (position.x & 0b11111) as u32,
            (position.y & 0b11111) as u32,
            (position.z & 0b11111) as u32,
        );

        Some((chunk, unsafe { ChunkPosition::new_unchecked(chunk_part_position, chunk_part_index as usize) }))
    }

    #[inline]
    pub fn get_block(&self, position: Vector3<i32>) -> Option<&Block> {
        let (chunk, local_chunk_position) = self.get_chunk_and_chunk_position(position)?;
        Some(chunk.get_block(local_chunk_position))
    }

    #[inline]
    pub fn set_block(&mut self, position: Vector3<i32>, block: Block) {
        let Some((chunk, local_chunk_position)) = self.get_chunk_mut_and_chunk_position(position) else { return; };
        chunk.set_block(local_chunk_position, block);
    }

    #[inline]
    pub fn get_sky_light_level(&self, position: Vector3<i32>) -> Option<u8> {
        let (chunk, local_chunk_position) = self.get_chunk_and_chunk_position(position)?;
        Some(chunk.get_sky_light_level(local_chunk_position))
    }

    #[inline]
    pub fn get_block_light_level(&self, position: Vector3<i32>) -> Option<u8> {
        let (chunk, local_position) = self.get_chunk_and_chunk_position(position)?;
        Some(chunk.get_block_light_level(local_position))
    }
    
    #[inline]
    pub fn set_sky_light_level(&mut self, position: Vector3<i32>, level: u8) {
        let Some((chunk, local_position)) = self.get_chunk_mut_and_chunk_position(position) else { return; };
        chunk.set_sky_light_level(local_position, level);
    }

    #[inline]
    pub fn set_block_light_level(&mut self, position: Vector3<i32>, level: u8) {
        let Some((chunk, local_position)) = self.get_chunk_mut_and_chunk_position(position) else { return; };
        chunk.set_block_light_level(local_position, level);
    }

    pub fn insert_structure(&mut self, structure: &Structure, origin_point: Vector3<i32>) {
        for (position, block) in structure.blocks().iter().cloned() {
            self.set_block(position + origin_point, block);
        }
    }

    pub fn new(chunk_map: &mut ChunkMap, center_chunk_position: Vector2<i32>) -> Option<Self> {
        for z in -1..=1 {
            for x in -1..=1 {
                if !chunk_map.contains_position(&(center_chunk_position + Vector2::new(x, z))) { return None; }
            }
        }
        
        let mut chunks: Box<[MaybeUninit<Chunk>; 9]> = Box::new(std::array::from_fn(|_| MaybeUninit::uninit()));
        let mut index = 0;
        for z in -1..=1 {
            for x in -1..=1 {
                chunks[index] = MaybeUninit::new(Arc::into_inner(chunk_map.remove(&(center_chunk_position + Vector2::new(x, z))).unwrap()).unwrap());
                index += 1;
            }
        }

        let mut chunks = unsafe { std::mem::transmute::<_, Box<[Chunk; 9]>>(chunks) };
        for chunk in chunks.iter_mut() {
            for part in chunk.parts.iter_mut() {
                part.light_level_layers.uncompress();
            }
        }

        Some(Self { chunks })
    }

    pub fn return_to_chunk_map(self, chunk_map: &mut ChunkMapLock) {
        let mut chunk_map = chunk_map.write();
        for mut chunk in self.chunks.into_iter() {
            chunk.maintain_parts();
            chunk_map.insert(chunk);
        }
    }

    #[inline]
    pub fn center_chunk(&self) -> &Chunk {
        &self.chunks[4]
    }
    
    #[inline]
    pub fn center_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.chunks[4]
    }

    #[inline]
    fn step_block_light_propagation_towards(&mut self, position: Vector3<i32>, direction: Vector3<i32>, light_level: u8, propagation_queue: &mut VecDeque<LightNode>) {
        let neighbor_position = position + direction;
        let Some((chunk, chunk_position)) = self.get_chunk_mut_and_chunk_position(neighbor_position) else { return; };
        let neighbor_block = chunk.get_block(chunk_position);
        let neighbor_block_light_level = chunk.get_block_light_level(chunk_position);
        
        let attenuation = unsafe { neighbor_block.properties().light_attenuation.from_direction(direction).unwrap_unchecked() };
        let new_neighbor_block_light_level = light_level.saturating_sub(attenuation + 1);

        if neighbor_block_light_level >= new_neighbor_block_light_level { return; }

        chunk.set_block_light_level(chunk_position, new_neighbor_block_light_level);
        propagation_queue.push_back(LightNode::new(neighbor_position));
    }

    #[inline]
    pub fn propagate_block_light_at(&mut self, position: Vector3<i32>) {
        let mut propagation_queue = VecDeque::new();
        let light_level = self.get_block_light_level(position).unwrap();

        self.step_block_light_propagation_towards(position, Vector3::new(1, 0, 0), light_level, &mut propagation_queue);
        self.step_block_light_propagation_towards(position, Vector3::new(-1, 0, 0), light_level, &mut propagation_queue);
        self.step_block_light_propagation_towards(position, Vector3::new(0, 0, 1), light_level, &mut propagation_queue);
        self.step_block_light_propagation_towards(position, Vector3::new(0, 0, -1), light_level, &mut propagation_queue);
        self.step_block_light_propagation_towards(position, Vector3::new(0, 1, 0), light_level, &mut propagation_queue);
        self.step_block_light_propagation_towards(position, Vector3::new(0, -1, 0), light_level, &mut propagation_queue);

        let mut visited_nodes: HashMap<LightNode, u8> = HashMap::new();
        while let Some(light_node) = propagation_queue.pop_front() {
            let position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            let light_level = self.get_block_light_level(position).unwrap();

            if light_level <= 1 { continue; }
            
            match visited_nodes.entry(light_node) {
                hashbrown::hash_map::Entry::Occupied(mut occupied) => {
                    let level = *occupied.get();
                    if level < light_level {
                        occupied.insert(light_level);
                    } else { continue; }
                },
                hashbrown::hash_map::Entry::Vacant(vacant) => {
                    vacant.insert(light_level);
                }
            }

            self.step_block_light_propagation_towards(position, Vector3::new(1, 0, 0), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(-1, 0, 0), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(0, 0, 1), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(0, 0, -1), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(0, 1, 0), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(0, -1, 0), light_level, &mut propagation_queue);
        }
    }   

    #[inline]
    fn is_block_light_level_supported_at(&self, position: Vector3<i32>) -> bool {
        let light_level = self.get_block_light_level(position).unwrap();
        let block = self.get_block(position).unwrap();

        if light_level == 0 || block.properties().emitted_light == light_level { return true; }

        let directions = [Vector3 { x: 1, y: 0, z: 0 }, Vector3 { x: -1, y: 0, z: 0 }, Vector3 { x: 0, y: 0, z: 1 }, Vector3 { x: 0, y: 0, z: -1 }, Vector3 { x: 0, y: 1, z: 0 }, Vector3 { x: 0, y: -1, z: 0 }, ];
        let light_attenuation = block.properties().light_attenuation;
        for direction in directions {
            let neighbor_position = position + direction;
            let Some(neighbor_block_light_level) = self.get_block_light_level(neighbor_position) else { continue; };
            if neighbor_block_light_level == light_level + 1 + light_attenuation.from_direction(direction * -1).unwrap() { return true; }
        }

        false
    }

    #[inline]
    fn step_block_light_removal_towards(&mut self, position: Vector3<i32>, direction: Vector3<i32>, removal_queue: &mut VecDeque<LightNode>, propagation_queue: &mut VecDeque<LightNode>) {
        let neighbor_position = position + direction;
        let Some(light_level) = self.get_block_light_level(neighbor_position) else { return; };

        if light_level == 0 { return; }

        if self.is_block_light_level_supported_at(neighbor_position) {
            propagation_queue.push_back(LightNode::new(neighbor_position));
        } else {
            removal_queue.push_back(LightNode::new(neighbor_position));
            let emitted_light = self.get_block(neighbor_position).unwrap().properties().emitted_light;
            if emitted_light > 0 {
                propagation_queue.push_back(LightNode::new(neighbor_position));
                self.set_block_light_level(neighbor_position, emitted_light);
            } else {
                self.set_block_light_level(neighbor_position, 0);
            }
        }
    }

    #[inline]
    pub fn remove_block_light_at(&mut self, position: Vector3<i32>) {
        let mut propagation_queue = VecDeque::new();
        let mut removal_queue = VecDeque::new();

        self.step_block_light_removal_towards(position, Vector3::new(1, 0, 0), &mut removal_queue, &mut propagation_queue);
        self.step_block_light_removal_towards(position, Vector3::new(-1, 0, 0), &mut removal_queue, &mut propagation_queue);
        self.step_block_light_removal_towards(position, Vector3::new(0, 0, 1), &mut removal_queue, &mut propagation_queue);
        self.step_block_light_removal_towards(position, Vector3::new(0, 0, -1), &mut removal_queue, &mut propagation_queue);
        self.step_block_light_removal_towards(position, Vector3::new(0, 1, 0), &mut removal_queue, &mut propagation_queue);
        self.step_block_light_removal_towards(position, Vector3::new(0, -1, 0), &mut removal_queue, &mut propagation_queue);

        while let Some(light_node) = removal_queue.pop_back() {
            let position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);

            self.step_block_light_removal_towards(position, Vector3::new(1, 0, 0), &mut removal_queue, &mut propagation_queue);
            self.step_block_light_removal_towards(position, Vector3::new(-1, 0, 0), &mut removal_queue, &mut propagation_queue);
            self.step_block_light_removal_towards(position, Vector3::new(0, 0, 1), &mut removal_queue, &mut propagation_queue);
            self.step_block_light_removal_towards(position, Vector3::new(0, 0, -1), &mut removal_queue, &mut propagation_queue);
            self.step_block_light_removal_towards(position, Vector3::new(0, 1, 0), &mut removal_queue, &mut propagation_queue);
            self.step_block_light_removal_towards(position, Vector3::new(0, -1, 0), &mut removal_queue, &mut propagation_queue);
        }

        let mut visited_nodes: HashMap<LightNode, u8> = HashMap::new();
        while let Some(light_node) = propagation_queue.pop_front() {
            let position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            let Some(light_level) = self.get_block_light_level(position) else { continue; };

            if light_level <= 1 { continue; }

            match visited_nodes.entry(light_node) {
                hashbrown::hash_map::Entry::Occupied(mut occupied) => {
                    let level = *occupied.get();
                    if level < light_level {
                        occupied.insert(light_level);
                    } else { continue; }
                },
                hashbrown::hash_map::Entry::Vacant(vacant) => {
                    vacant.insert(light_level);
                }
            }

            self.step_block_light_propagation_towards(position, Vector3::new(1, 0, 0), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(-1, 0, 0), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(0, 0, 1), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(0, 0, -1), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(0, 1, 0), light_level, &mut propagation_queue);
            self.step_block_light_propagation_towards(position, Vector3::new(0, -1, 0), light_level, &mut propagation_queue);
        }
    }


    #[inline]
    fn is_sky_light_level_supported_at(&self, position: Vector3<i32>) -> bool {
        let light_level = self.get_sky_light_level(position).unwrap();
        let block = self.get_block(position).unwrap();

        let directions = [Vector3 { x: 1, y: 0, z: 0 }, Vector3 { x: -1, y: 0, z: 0 }, Vector3 { x: 0, y: 0, z: 1 }, Vector3 { x: 0, y: 0, z: -1 }, Vector3 { x: 0, y: -1, z: 0 }, ];
        let light_attenuation = block.properties().light_attenuation;
        for direction in directions {
            let neighbor_position = position + direction;
            let Some(neighbor_sky_light_level) = self.get_sky_light_level(neighbor_position) else { continue; };
            if neighbor_sky_light_level == light_level + 1 + light_attenuation.from_direction(direction * -1).unwrap() { return true; }
        }

        let neighbor_position = position + Vector3::unit_y();
        let Some(neighbor_sky_light_level) = self.get_sky_light_level(neighbor_position) else { return false; };
        if (neighbor_sky_light_level == LIGHT_LEVEL_MAX_VALUE && light_level == LIGHT_LEVEL_MAX_VALUE)
        || neighbor_sky_light_level == light_level + 1 + light_attenuation.from_direction(Vector3::new(0, -1, 0)).unwrap() {
            return true;
        }

        false
    }

    #[inline]
    fn step_sky_light_removal_towards(&mut self, position: Vector3<i32>, direction: Vector3<i32>, removal_queue: &mut VecDeque<LightNode>, propagation_queue: &mut VecDeque<LightNode>) {
        let neighbor_position = position + direction;
        let Some(neighbor_light_level) = self.get_sky_light_level(neighbor_position) else { return; };

        if neighbor_light_level == 0 { return; }

        if self.is_sky_light_level_supported_at(neighbor_position) {
            propagation_queue.push_back(LightNode::new(neighbor_position));
        } else {
            removal_queue.push_back(LightNode::new(neighbor_position));
            self.set_sky_light_level(neighbor_position, 0);
        }
    }

    #[inline]
    fn step_sky_light_propagation_towards(&mut self, position: Vector3<i32>, direction: Vector3<i32>, light_level: u8, propagation_queue: &mut VecDeque<LightNode>) {
        let neighbor_position = position + direction;
        let Some((chunk, chunk_position)) = self.get_chunk_mut_and_chunk_position(neighbor_position) else { return; };

        let neighbor_light_level = chunk.get_sky_light_level(chunk_position);
        let neighbor_block = chunk.get_block(chunk_position);

        let attenuation = unsafe { neighbor_block.properties().light_attenuation.from_direction(direction).unwrap_unchecked() };
        let new_neighbor_sky_light_level = light_level.saturating_sub(attenuation + 1);

        if neighbor_light_level >= new_neighbor_sky_light_level { return; }

        chunk.set_sky_light_level(chunk_position, new_neighbor_sky_light_level);

        if new_neighbor_sky_light_level == 1 { return; }

        propagation_queue.push_back(LightNode::new(neighbor_position));
    }

    #[inline]
    fn step_sky_light_propagation_downwards(&mut self, position: Vector3<i32>, light_level: u8, propagation_queue: &mut VecDeque<LightNode>) {
        let neighbor_position = position - Vector3::unit_y();
        let Some((chunk, chunk_position)) = self.get_chunk_mut_and_chunk_position(neighbor_position) else { return; };

        let neighbor_light_level = chunk.get_sky_light_level(chunk_position);
        let neighbor_block = chunk.get_block(chunk_position);

        let attenuation = neighbor_block.properties().light_attenuation.from_direction(Vector3::unit_y()).unwrap();
        let new_neighbor_sky_light_level = light_level.saturating_sub(attenuation + (light_level != LIGHT_LEVEL_MAX_VALUE) as u8);

        if neighbor_light_level >= new_neighbor_sky_light_level { return; }

        chunk.set_sky_light_level(chunk_position, new_neighbor_sky_light_level);

        if new_neighbor_sky_light_level == 1 { return; }

        propagation_queue.push_back(LightNode::new(neighbor_position));
    }

    pub fn update_sky_light_level_at(&mut self, position: Vector3<i32>) {
        let mut propagation_queue = VecDeque::new();
        let mut removal_queue = VecDeque::new();

        self.step_sky_light_removal_towards(position, Vector3::new(1, 0, 0), &mut removal_queue, &mut propagation_queue);
        self.step_sky_light_removal_towards(position, Vector3::new(-1, 0, 0), &mut removal_queue, &mut propagation_queue);
        self.step_sky_light_removal_towards(position, Vector3::new(0, 0, 1), &mut removal_queue, &mut propagation_queue);
        self.step_sky_light_removal_towards(position, Vector3::new(0, 0, -1), &mut removal_queue, &mut propagation_queue);
        self.step_sky_light_removal_towards(position, Vector3::new(0, 1, 0), &mut removal_queue, &mut propagation_queue);
        self.step_sky_light_removal_towards(position, Vector3::new(0, -1, 0), &mut removal_queue, &mut propagation_queue);

        while let Some(light_node) = removal_queue.pop_front() {
            let position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            self.step_sky_light_removal_towards(position, Vector3::new(1, 0, 0), &mut removal_queue, &mut propagation_queue);
            self.step_sky_light_removal_towards(position, Vector3::new(-1, 0, 0), &mut removal_queue, &mut propagation_queue);
            self.step_sky_light_removal_towards(position, Vector3::new(0, 0, 1), &mut removal_queue, &mut propagation_queue);
            self.step_sky_light_removal_towards(position, Vector3::new(0, 0, -1), &mut removal_queue, &mut propagation_queue);
            self.step_sky_light_removal_towards(position, Vector3::new(0, 1, 0), &mut removal_queue, &mut propagation_queue);
            self.step_sky_light_removal_towards(position, Vector3::new(0, -1, 0), &mut removal_queue, &mut propagation_queue);
        }

        while let Some(light_node) = propagation_queue.pop_front() {
            let position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            let light_level = unsafe { self.get_sky_light_level(position).unwrap_unchecked() };

            self.step_sky_light_propagation_downwards(position, light_level, &mut propagation_queue);
            self.step_sky_light_propagation_towards(position, Vector3::new(1, 0, 0), light_level, &mut propagation_queue);
            self.step_sky_light_propagation_towards(position, Vector3::new(-1, 0, 0), light_level, &mut propagation_queue);
            self.step_sky_light_propagation_towards(position, Vector3::new(0, 0, 1), light_level, &mut propagation_queue);
            self.step_sky_light_propagation_towards(position, Vector3::new(0, 0, -1), light_level, &mut propagation_queue);
            self.step_sky_light_propagation_towards(position, Vector3::new(0, 1, 0), light_level, &mut propagation_queue);
        }
    }

    pub fn propagate_sky_light(&mut self) {
        let mut propagation_queue = VecDeque::new();
        let mut min_y = None;
        let chunk = self.center_chunk_mut();

        'outer: for (chunk_part_index, chunk_part) in chunk.parts.iter().enumerate().rev() {
            let y_offset = chunk_part_index * CHUNK_SIZE;
            for (i, layer) in chunk_part.block_layers.0.iter().enumerate().rev() {
                match layer {
                    super::chunk_part::BlockLayer::Compressed(block_pallet_id) => {
                        let block = &chunk_part.block_pallet.get(block_pallet_id).unwrap().block;
                        if block.properties().light_attenuation.from_direction(Vector3::unit_y()).unwrap() > 0 { break 'outer; }
                        min_y = Some(i + y_offset);
                    },
                    super::chunk_part::BlockLayer::Uncompressed(_) => break 'outer
                }
            }
        }

        if let Some(min_y) = min_y {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    propagation_queue.push_back(LightNode::new(Vector3::new(x as i32, min_y as i32, z as i32)));
                }
            }

            for y in min_y..CHUNK_HEIGHT {
                let chunk_part_index = y / CHUNK_SIZE; // can just divide because y > 0
                let chunk_part = &mut chunk.parts[chunk_part_index];
                let local_y = y % CHUNK_SIZE; // can non euclid rem y > 0
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let position = unsafe { ChunkPartPosition::new_unchecked(Vector3::new(x as u32, local_y as u32, z as u32)) };
                        chunk_part.set_sky_light_level(position, LIGHT_LEVEL_MAX_VALUE);
                    }
                }
            }
        }

        while let Some(light_node) = propagation_queue.pop_front() {
            let position = Vector3::new(light_node.x as i32, light_node.y as i32, light_node.z as i32);
            let Some(light_level) = self.get_sky_light_level(position) else { continue; };

            self.step_sky_light_propagation_towards(position, Vector3::new(1, 0, 0), light_level, &mut propagation_queue);
            self.step_sky_light_propagation_towards(position, Vector3::new(-1, 0, 0), light_level, &mut propagation_queue);
            self.step_sky_light_propagation_towards(position, Vector3::new(0, 0, 1), light_level, &mut propagation_queue);
            self.step_sky_light_propagation_towards(position, Vector3::new(0, 0, -1), light_level, &mut propagation_queue);
            self.step_sky_light_propagation_towards(position, Vector3::new(0, 1, 0), light_level, &mut propagation_queue);
            self.step_sky_light_propagation_downwards(position, light_level, &mut propagation_queue);
        }
    }
}
