#![allow(unused)]
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use bevy::math::*;
use itertools::iproduct;

use crate::data::constants::*;

use super::{Chunk, ChunkNeighbors, Direction};

type ArcRwChunk = Arc<RwLock<Chunk>>;

#[derive(Debug, bevy::prelude::Resource)]
pub struct World {
    chunks: HashMap<IVec3, ArcRwChunk>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    #[inline(always)]
    pub const fn chunk_to_world_position(position: IVec3) -> Vec3 {
        Vec3::new(
            (position.x * CHUNK_SIZE_I32) as f32,
            (position.y * CHUNK_SIZE_I32) as f32,
            (position.z * CHUNK_SIZE_I32) as f32,
        )
    }

    #[inline(always)]
    pub const fn world_to_chunk_position(position: IVec3) -> IVec3 {
        IVec3::new(
            position.x.div_euclid(CHUNK_SIZE_I32),
            position.y.div_euclid(CHUNK_SIZE_I32),
            position.z.div_euclid(CHUNK_SIZE_I32),
        )
    }

    #[inline(always)]
    pub const fn world_to_chunk_voxel_position(position: IVec3) -> UVec3 {
        UVec3::new(
            position.x.rem_euclid(CHUNK_SIZE_I32) as u32,
            position.y.rem_euclid(CHUNK_SIZE_I32) as u32,
            position.z.rem_euclid(CHUNK_SIZE_I32) as u32,
        )
    }

    pub const fn chunks(&self) -> &HashMap<IVec3, ArcRwChunk> {
        &self.chunks
    }

    pub fn chunks_mut(&mut self) -> &mut HashMap<IVec3, ArcRwChunk> {
        &mut self.chunks
    }

    pub fn chunk_exists(&self, position: IVec3) -> bool {
        self.chunks.contains_key(&position)
    }

    pub fn set_chunk(&mut self, position: IVec3, chunk: Arc<RwLock<Chunk>>) {
        self.chunks.insert(position, chunk);
    }

    pub fn remove_chunk(&mut self, position: IVec3) {
        self.chunks.remove(&position);
    }

    pub fn get_chunk_arc(&self, position: IVec3) -> Arc<RwLock<Chunk>> {
        self.chunks
            .get(&position)
            .expect("failed to read chunk arc")
            .clone()
    }

    pub fn get_chunk(&self, position: IVec3) -> Option<RwLockReadGuard<Chunk>> {
        Some(
            self.chunks
                .get(&position)?
                .read()
                .expect("failed to read chunk"),
        )
    }

    pub fn get_chunk_mut(&self, position: IVec3) -> Option<RwLockWriteGuard<Chunk>> {
        Some(
            self.chunks
                .get(&position)?
                .write()
                .expect("failed to write chunk"),
        )
    }

    pub fn get_chunk_neighbors(&self, position: IVec3) -> ChunkNeighbors {
        let mut neighbors = ChunkNeighbors::default();
        for direction in Direction::LIST.iter() {
            let position = direction.direction() + position;
            if let Some(chunk) = self.chunks.get(&position) {
                let weak_pointer = Arc::downgrade(chunk);
                neighbors.neighbors.push(weak_pointer);
            }
        }

        neighbors
    }

    pub fn update_chunk_neighbors(&mut self, position: IVec3) {
        let mut chunk = self.get_chunk_mut(position).expect("failed to get chunk");
        chunk.neighbors = self.get_chunk_neighbors(position);
    }
}
