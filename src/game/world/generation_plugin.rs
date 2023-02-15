use bevy::{prelude::*, tasks::*};
use futures_lite::future::{block_on, poll_once};

use crate::world::{Chunk, World};

use super::data::*;

fn queue_chunk_terrain_generation(
    mut commands: Commands,
    new_chunks: Query<(Entity, &ChunkComponent), Added<ChunkComponent>>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    for (entity, chunk_component) in &new_chunks {
        let chunk_position = chunk_component.0;
        let task = task_pool.spawn(async move {
            let chunk = Chunk::generate_at(chunk_position).unwrap();
            chunk
        });

        commands.entity(entity).insert(TerrainGenerationTask(task));
    }
}

fn process_chunk_terrain_generation(
    mut commands: Commands,
    mut generating_chunks: Query<(Entity, &ChunkComponent, &mut TerrainGenerationTask)>,
    mut dirty_chunks: ResMut<DirtyChunks>,
    mut world: ResMut<World>,
) {
    for (entity, chunk_component, mut task) in &mut generating_chunks {
        if let Some(chunk) = block_on(poll_once(&mut task.0)) {
            commands.entity(entity).remove::<TerrainGenerationTask>();

            let chunk_position = chunk_component.0;
            world.set_chunk(chunk_position, chunk);
            dirty_chunks.mark_dirty(chunk_position);
        }
    }
}

pub struct ChunkGenerationPlugin;
impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            ChunkLoadingStage,
            ChunkGenerationStage,
            SystemStage::parallel()
                .with_system(queue_chunk_terrain_generation)
                .with_system(
                    process_chunk_terrain_generation.after(queue_chunk_terrain_generation),
                ),
        );
    }
}
