pub mod components;
pub mod events;
pub mod worker;

use bevy::{prelude::*, render::view::NoFrustumCulling};

use crate::{
    rendering::*,
    world::{meshing, World},
};

use self::{components::*, events::*, worker::*};

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnChunkEvent>()
            .add_event::<DespawnChunkEvent>()
            .add_event::<RebuildChunkEvent>()
            .add_plugin(WorldWorkerPlugin)
            .init_resource::<World>()
            .add_system(spawn_chunks)
            .add_system(despawn_chunks)
            .add_system(verify_chunk_meshes)
            .add_system(debug);
    }
}

fn debug(
    mut commands: Commands,
    mut world: ResMut<World>,
    input: Res<Input<KeyCode>>,
    query: Query<Entity, With<ChunkComponent>>,
) {
    if input.just_released(KeyCode::F) {
        let chunks = world.chunks().clone();
        let iter = chunks.keys().clone();
        for position in iter {
            world.update_chunk_neighbors(*position);
        }

        for entity in &query {
            commands.entity(entity).insert(ChunkSouldRegenerateMesh);
        }
    }
}

fn spawn_chunks(
    mut events: EventReader<SpawnChunkEvent>,
    mut commands: Commands,
    mut world: ResMut<World>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    atlas: ResMut<ChunkTextureAtlas>,
) {
    for event in events.iter() {
        let chunk = event.0.clone();
        let readable_chunk = chunk.read().unwrap();

        world.set_chunk(readable_chunk.position(), chunk.clone());

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
            .insert(NoFrustumCulling)
            .insert(ChunkSouldRegenerateMesh);
    }
}

fn despawn_chunks(
    mut events: EventReader<DespawnChunkEvent>,
    mut world: ResMut<World>,
    mut commands: Commands,
    query: Query<(Entity, &ChunkComponent)>,
) {
    for event in events.iter() {
        let chunk_position = event.0;
        for (entity, component) in &query {
            if component.0 == chunk_position {
                world.remove_chunk(chunk_position);
                commands.entity(entity).despawn_recursive();
                break;
            }
        }
    }
}

fn verify_chunk_meshes(
    mut commands: Commands,
    mut events: EventWriter<RebuildChunkEvent>,
    query: Query<(Entity, &ChunkComponent, &Handle<Mesh>), With<ChunkSouldRegenerateMesh>>,
    world: ResMut<World>,
) {
    for (entity, component, mesh) in query.iter() {
        let Some(chunk) = world.get_chunk(component.0) else {
            commands.entity(entity).despawn_recursive();
            continue;
        };

        commands.entity(entity).remove::<ChunkSouldRegenerateMesh>();

        events.send(RebuildChunkEvent(chunk.position(), mesh.clone()));
    }
}
