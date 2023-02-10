use bevy::prelude::*;

use crate::world::World;

use super::{
    data::*, generation_plugin::ChunkGenerationPlugin, loading_plugin::ChunkLoadingPlugin,
    meshing_plugin::ChunkMeshingPlugin,
};

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkCommandQueue>()
            .init_resource::<ChunkEntities>()
            .init_resource::<DirtyChunks>()
            .init_resource::<World>()
            .add_plugin(ChunkLoadingPlugin)
            .add_plugin(ChunkGenerationPlugin)
            .add_plugin(ChunkMeshingPlugin);
    }
}
