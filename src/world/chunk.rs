#![allow(unused)]

use bevy::prelude::{IVec3, Vec3};
use rand::Rng;

use crate::data::{constants::*, VoxelType};

use super::*;

#[derive(Debug, Clone)]
pub struct Chunk {
    voxels: VoxelMap,
    world_position: IVec3,
}

impl Chunk {
    pub fn new(position: IVec3) -> Chunk {
        Chunk {
            voxels: VoxelMap::new(),
            world_position: position,
        }
    }

    #[inline(always)]
    pub const fn position(&self) -> IVec3 {
        self.world_position
    }

    #[inline(always)]
    pub const fn world_position(&self) -> Vec3 {
        World::chunk_to_world_position(self.world_position)
    }

    pub fn generate_at(world_position: IVec3) -> Option<Chunk> {
        let mut chunk = Chunk::new(world_position);
        let mut empty_chunk = true;
        let mut rng = rand::thread_rng();

        let chunk_world_position = chunk.world_position().as_ivec3();

        for voxel_position in chunk.iter_voxels() {
            let position = chunk_world_position + voxel_position;
            let h = 16 + rng.gen_range(-4..=4);
            if position.y > h {
                chunk.set_voxel(VoxelType::Air, voxel_position);
            } else if position.y == h {
                chunk.set_voxel(VoxelType::Grass, voxel_position);
                empty_chunk = false;
            } else {
                chunk.set_voxel(VoxelType::Dirt, voxel_position);
                empty_chunk = false;
            }
        }

        //if empty_chunk {
        //    None
        //} else {
        Some(chunk)
        //}
    }

    pub const fn iter_voxels(&self) -> voxel_map::VoxelIterator {
        self.voxels.iter()
    }

    pub const fn get_voxel(&self, position: IVec3) -> VoxelType {
        self.voxels.get_at(position)
    }

    pub fn is_transparent_at(&self, position: IVec3) -> bool {
        self.voxels.get_at(position).is_transparent()
    }

    pub fn set_voxel(&mut self, voxel: VoxelType, position: IVec3) {
        self.voxels.set_at(voxel, position)
    }
}

#[inline(always)]
const fn wrap_position(x: i32) -> i32 {
    if x < 0 {
        -1
    } else if x >= CHUNK_SIZE_I32 {
        1
    } else {
        0
    }
}
