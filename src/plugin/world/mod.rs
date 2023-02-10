pub mod components;
pub mod events;
mod systems;
pub mod worker;

use bevy::prelude::*;

use crate::world::World;

use self::{events::*, worker::*};
use systems::*;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnChunkEvent>()
            .add_event::<DespawnChunkEvent>()
            .add_event::<RebuildChunkEvent>()
            .add_event::<UpdateNearbyChunks>()
            .add_plugin(WorldWorkerPlugin)
            .init_resource::<World>()
            .add_system(verify_chunk_meshes);
    }
}
