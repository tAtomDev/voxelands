use bevy::{prelude::*, render::view::NoFrustumCulling};

use crate::{
    rendering::{ChunkMaterial, ChunkTextureAtlas},
    world::meshing,
};

use super::{components::*, events::*};

pub fn spawn_chunks(
    mut commands: Commands,
    mut events: EventReader<SpawnChunkEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut nearby_events: EventWriter<UpdateNearbyChunks>,
    atlas: ResMut<ChunkTextureAtlas>,
) {
    for event in events.iter() {
        let chunk = event.0.clone();
        let readable_chunk = chunk.read().unwrap();

        commands
            .spawn(ChunkComponent(readable_chunk.position()))
            .insert(MaterialMeshBundle {
                transform: Transform::from_xyz(
                    readable_chunk.world_position().x,
                    readable_chunk.world_position().y,
                    readable_chunk.world_position().z,
                ),
                material: materials.add(ChunkMaterial {
                    texture_atlas: atlas.0.clone(),
                }),
                mesh: meshes.add(meshing::generate_empty_chunk_mesh()),
                ..Default::default()
            })
            .insert(NoFrustumCulling);

        nearby_events.send(UpdateNearbyChunks(readable_chunk.position()));
    }
}

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
                commands.entity(entity).insert(ChunkSouldRegenerateMesh);
            }
        }
    }
}

pub fn verify_chunk_meshes(
    mut commands: Commands,
    mut events: EventWriter<RebuildChunkEvent>,
    query: Query<(Entity, &ChunkComponent, &Handle<Mesh>), With<ChunkSouldRegenerateMesh>>,
) {
    for (entity, component, mesh) in query.iter() {
        commands.entity(entity).remove::<ChunkSouldRegenerateMesh>();

        events.send(RebuildChunkEvent(component.0, mesh.clone()));
    }
}
