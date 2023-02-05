use std::sync::{Arc, RwLock, Weak};

use bevy::prelude::{IVec3, Vec3};
use rand::Rng;

use crate::data::{constants::*, VoxelType};

use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    Back,
    Front,
}

impl From<IVec3> for Direction {
    fn from(value: IVec3) -> Self {
        let xyz = (value.x, value.y, value.z);
        match xyz {
            (-1, 0, 0) => Direction::Left,
            (1, 0, 0) => Direction::Right,
            (0, 1, 0) => Direction::Up,
            (0, -1, 0) => Direction::Down,
            (0, 0, 1) => Direction::Front,
            (0, 0, -1) => Direction::Back,
            _ => panic!("{xyz:?}: invalid direction"),
        }
    }
}

impl Direction {
    pub const LIST: [Direction; 6] = [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
        Direction::Front,
        Direction::Back,
    ];

    pub const fn direction(&self) -> IVec3 {
        match *self {
            Direction::Left => IVec3::new(-1, 0, 0),
            Direction::Right => IVec3::new(1, 0, 0),
            Direction::Up => IVec3::new(0, 1, 0),
            Direction::Down => IVec3::new(0, -1, 0),
            Direction::Front => IVec3::new(0, 0, 1),
            Direction::Back => IVec3::new(0, 0, -1),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChunkNeighbors {
    pub neighbors: Vec<Weak<RwLock<Chunk>>>,
}

impl std::fmt::Display for ChunkNeighbors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data: Vec<String> = self
            .neighbors
            .iter()
            .map(|n| {
                n.upgrade()
                    .unwrap()
                    .read()
                    .unwrap()
                    .world_position
                    .to_string()
            })
            .collect();
        write!(f, "{:?}", data)
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    voxels: VoxelMap,
    world_position: IVec3,
    pub neighbors: ChunkNeighbors,
    pub should_regenerate_mesh: bool,
}

impl Chunk {
    pub fn new(position: IVec3, neighbors: ChunkNeighbors) -> Chunk {
        Chunk {
            voxels: VoxelMap::new(),
            world_position: position,
            neighbors,
            should_regenerate_mesh: false,
        }
    }

    pub const fn position(&self) -> IVec3 {
        self.world_position
    }

    pub const fn world_position(&self) -> Vec3 {
        World::chunk_to_world_position(self.world_position)
    }

    pub fn generate_at(world_position: IVec3, neighbors: ChunkNeighbors) -> Option<Chunk> {
        let mut chunk = Chunk::new(world_position, neighbors);
        let mut empty_chunk = true;
        let mut rng = rand::thread_rng();

        for voxel_position in chunk.iter_voxels() {
            let position = chunk.world_position().as_ivec3() + voxel_position;
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

        if empty_chunk {
            None
        } else {
            Some(chunk)
        }
    }

    pub const fn iter_voxels(&self) -> voxel_map::VoxelIterator {
        self.voxels.iter()
    }

    fn get_neighbor_at_voxel_position(
        &self,
        position: IVec3,
    ) -> Option<(IVec3, Arc<RwLock<Chunk>>)> {
        if VoxelMap::is_within_bounds(position) || self.neighbors.neighbors.is_empty() {
            return None;
        }

        let relative_neighbor_voxel_position = IVec3::new(
            position.x.rem_euclid(CHUNK_SIZE_I32),
            position.y.rem_euclid(CHUNK_SIZE_I32),
            position.z.rem_euclid(CHUNK_SIZE_I32),
        );

        let relative_chunk_neighbor_position = IVec3::new(
            wrap_position(position.x),
            wrap_position(position.y),
            wrap_position(position.z),
        );

        if relative_chunk_neighbor_position == IVec3::ZERO {
            return None;
        }

        let chunk_world_position = relative_chunk_neighbor_position + self.world_position;

        let mut result = None;
        for neighbor in self.neighbors.neighbors.iter() {
            let Some(neighbor) = neighbor.upgrade() else {
                continue;
            };

            let chunk = neighbor.read().unwrap();
            if chunk.world_position == chunk_world_position {
                result = Some((relative_neighbor_voxel_position, neighbor.clone()));
                break;
            }
        }

        result
    }

    pub const fn get_voxel(&self, position: IVec3) -> VoxelType {
        self.voxels.get_at(position)
    }

    pub fn is_transparent_at(&self, position: IVec3) -> bool {
        if let Some((voxel_position, neighbor)) = self.get_neighbor_at_voxel_position(position) {
            let neighbor = neighbor.read().expect("failed to read neighbor");
            neighbor.get_voxel(voxel_position).is_transparent()
        } else {
            self.voxels.get_at(position).is_transparent()
        }
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
