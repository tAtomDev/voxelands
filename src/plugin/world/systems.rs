use bevy::{prelude::*, tasks::AsyncComputeTaskPool};

use crate::world::{meshing, World};

use super::{components::*, events::*};

pub fn update_nearby_chunks(
    mut commands: Commands,
    mut events: EventReader<UpdateNearbyChunks>,
    chunks: Query<(Entity, &ChunkComponent)>,
) {
    for nearby in events.iter() {
        let nearby_position = nearby.0;
        for (entity, component) in &chunks {
            let chunk_position = component.0;
            if chunk_position.as_vec3().distance(nearby_position.as_vec3()) <= 1.0 {
                if let Some(mut entity) = commands.get_entity(entity) {
                    entity.insert(ChunkSouldRegenerateMesh);
                }
            }
        }
    }
}

pub fn verify_chunk_meshes(
    mut commands: Commands,
    mut world: ResMut<World>,
    query: Query<(Entity, &ChunkComponent, &Handle<Mesh>), With<ChunkSouldRegenerateMesh>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, component, mesh) in query.iter() {
        commands.entity(entity).remove::<ChunkSouldRegenerateMesh>();

        let chunk_position = component.0;

        world.update_chunk_neighbors(chunk_position);
        let chunk = world.get_chunk_arc(chunk_position);
        let task = thread_pool.spawn(async move {
            let chunk = chunk.read().unwrap();
            meshing::generate_chunk_mesh(&chunk)
        });

        commands.spawn(ChunkRebuildTask(task, chunk_position, mesh.clone(), entity));
    }
}
