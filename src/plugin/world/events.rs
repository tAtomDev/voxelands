use std::sync::{Arc, RwLock};

use bevy::prelude::*;

use crate::world::Chunk;

pub struct SpawnChunkEvent(pub Arc<RwLock<Chunk>>);
pub struct DespawnChunkEvent(pub IVec3);
pub struct RebuildChunkEvent(pub IVec3, pub Handle<Mesh>);

pub struct UpdateNearbyChunks(pub IVec3);
