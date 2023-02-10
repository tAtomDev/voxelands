#![allow(unused)]
const EPSILON: f32 = -1e-6f32;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use bevy::math::*;
use itertools::iproduct;

use crate::data::{
    constants::*,
    voxel_face::{VoxelFace, FACES},
    VoxelType,
};

use super::Chunk;

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
    pub const fn world_to_chunk_voxel_position(position: IVec3) -> IVec3 {
        IVec3::new(
            position.x.rem_euclid(CHUNK_SIZE_I32),
            position.y.rem_euclid(CHUNK_SIZE_I32),
            position.z.rem_euclid(CHUNK_SIZE_I32),
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

    pub fn set_chunk(&mut self, position: IVec3, chunk: Chunk) {
        self.chunks.insert(position, Arc::new(RwLock::new(chunk)));
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

    pub fn get_voxel(&self, position: IVec3) -> VoxelType {
        let chunk_position = World::world_to_chunk_position(position);
        let Some(chunk) = self.get_chunk(chunk_position) else {
            return VoxelType::Air;
        };

        let voxel_position = World::world_to_chunk_voxel_position(position);
        chunk.get_voxel(voxel_position)
    }

    pub fn set_voxel(&self, voxel_type: VoxelType, position: IVec3) {
        let chunk_position = World::world_to_chunk_position(position);
        let Some(mut chunk) = self.get_chunk_mut(chunk_position) else {
            return;
        };

        let voxel_position = World::world_to_chunk_voxel_position(position);
        chunk.set_voxel(voxel_type, voxel_position);
    }

    pub fn raytrace(&self, position: Vec3, direction: Vec3, range: f32) -> Option<RaytraceResult> {
        let direction = direction.normalize_or_zero();
        let start = position - direction * 0.5;
        let end = position + direction * (range + 0.5);

        let min = IVec3::min(start.as_ivec3(), end.as_ivec3()) - 1;
        let max = IVec3::max(start.as_ivec3(), end.as_ivec3()) + 1;

        let mut result: Option<RaytraceResult> = None;

        for (x, y, z) in iproduct!(min.x..=max.x, min.y..=max.y, min.z..=max.z) {
            let voxel_position = IVec3::new(x, y, z);
            let voxel_type = self.get_voxel(voxel_position);
            if voxel_type == VoxelType::Air {
                continue;
            }

            for face in FACES {
                let normal = face.normal().as_vec3();
                let divisor = Vec3::dot(normal, direction);

                // Ignore back faces
                if divisor >= EPSILON {
                    continue;
                }

                let plane_normal = normal * normal;
                let voxel_size = Vec3::splat(0.5);
                let d = -(Vec3::dot(voxel_position.as_vec3(), plane_normal)
                    + Vec3::dot(voxel_size, normal));
                let numerator = Vec3::dot(plane_normal, position) + d;

                let distance = f32::abs(-numerator / divisor);
                let point = position + distance * direction;

                if (point.x < (x as f32) - voxel_size.x + EPSILON
                    || point.x > (x as f32) + voxel_size.x - EPSILON
                    || point.y < (y as f32) - voxel_size.y + EPSILON
                    || point.y > (y as f32) + voxel_size.y - EPSILON
                    || point.z < (z as f32) - voxel_size.z + EPSILON
                    || point.z > (z as f32) + voxel_size.z - EPSILON)
                {
                    continue;
                }

                if distance <= range && (result.is_none() || result.unwrap().distance > distance) {
                    result = Some(RaytraceResult {
                        distance,
                        face,
                        point,
                        voxel_type,
                        voxel_position,
                    });
                }
            }
        }

        result
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RaytraceResult {
    pub face: VoxelFace,
    pub voxel_type: VoxelType,
    pub voxel_position: IVec3,
    pub distance: f32,
    pub point: Vec3,
}
