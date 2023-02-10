use std::collections::{hash_map::Keys, HashMap, HashSet};

use bevy::{prelude::*, tasks::Task};

use crate::world::Chunk;

#[derive(StageLabel, Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkLoadingStage;

#[derive(StageLabel, Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkGenerationStage;

#[derive(StageLabel, Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkMeshingPrepareStage;
#[derive(StageLabel, Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkMeshingStage;

#[derive(Component, Default)]
pub struct ChunkComponent(pub IVec3);

#[derive(Component)]
pub struct TerrainGenerationTask(pub Task<Chunk>);

#[derive(Component)]
pub struct ChunkMeshingTask(pub Task<Mesh>);

#[derive(Resource, Default)]
pub struct ChunkCommandQueue {
    pub create: Vec<IVec3>,
    pub destroy: Vec<IVec3>,
}

#[derive(Resource, Default)]
pub struct DirtyChunks(HashSet<IVec3>);

impl DirtyChunks {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn mark_dirty(&mut self, chunk: IVec3) {
        self.0.insert(chunk);
    }

    pub fn iter_dirty(&self) -> impl Iterator<Item = &IVec3> {
        self.0.iter()
    }

    //pub fn num_dirty(&self) -> usize {
    //    self.0.len()
    //}
}

#[derive(Resource, Default)]
pub struct ChunkEntities(HashMap<IVec3, Entity>);

impl ChunkEntities {
    pub fn keys(&self) -> Keys<IVec3, Entity> {
        self.0.keys()
    }

    pub fn entity(&self, position: &IVec3) -> Option<Entity> {
        self.0.get(position).copied()
    }

    pub fn attach_entity(&mut self, position: IVec3, entity: Entity) {
        self.0.insert(position, entity);
    }

    pub fn detach_entity(&mut self, position: &IVec3) -> Option<Entity> {
        self.0.remove(position)
    }
}
