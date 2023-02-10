use bevy::{prelude::*, tasks::Task};

use crate::world::Chunk;

#[derive(Component)]
pub struct ChunkComponent(pub IVec3);

#[derive(Component)]
pub struct ChunkSouldRegenerateMesh;

#[derive(Component)]
pub struct ChunkLoadTask(pub Task<Option<Chunk>>);

#[derive(Component)]
pub struct ChunkRebuildTask(pub Task<Mesh>, pub IVec3, pub Handle<Mesh>, pub Entity);
