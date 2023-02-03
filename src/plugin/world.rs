use bevy::prelude::*;
use itertools::iproduct;

use crate::{
    rendering::*,
    world::{meshing::generate_chunk_mesh, World},
};

use super::CameraState;

struct SpawnChunkEvent(IVec3);
struct RebuildChunkEvent(IVec3, Handle<Mesh>);

#[derive(Component)]
struct ChunkComponent(IVec3);

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnChunkEvent>()
            .add_event::<RebuildChunkEvent>()
            .init_resource::<World>()
            .add_system(spawn_chunks)
            .add_system(spawn_chunks_on_space_press)
            .add_system(verify_chunk_meshes)
            .add_system(rebuild_chunk_meshes);
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
        let chunk_position = event.0;
        let Some(chunk) = world.generate_chunk(chunk_position) else {
            return;
        };

        commands
            .spawn(ChunkComponent(chunk_position))
            .insert(MaterialMeshBundle {
                transform: Transform::from_xyz(
                    chunk.world_position().x,
                    chunk.world_position().y,
                    chunk.world_position().z,
                ),
                mesh: meshes.add(generate_chunk_mesh(&chunk)),
                material: materials.add(ChunkMaterial {
                    texture_atlas: atlas.0.clone(),
                }),
                ..Default::default()
            });
    }
}

fn spawn_chunks_on_space_press(
    mut events: EventWriter<SpawnChunkEvent>,
    input: Res<Input<KeyCode>>,
    camera_query: Query<&Transform, With<CameraState>>,
) {
    let transform = camera_query.single();
    let chunk_position =
        World::world_to_chunk_position(transform.translation.as_ivec3()) + IVec3::NEG_Y;
    if input.pressed(KeyCode::Space) {
        for (x, y, z) in iproduct!(-1..=1, -1..=1, -1..=1) {
            events.send(SpawnChunkEvent(chunk_position + IVec3::new(x, y, z)));
        }
    }
}

fn verify_chunk_meshes(
    mut events: EventWriter<RebuildChunkEvent>,
    query: Query<(&ChunkComponent, &Handle<Mesh>)>,
    world: ResMut<World>,
) {
    for (component, mesh) in query.iter() {
        let Some(mut chunk) = world.get_chunk_mut(component.0) else {
            continue;
        };

        if chunk.should_regenerate_mesh {
            events.send(RebuildChunkEvent(chunk.position(), mesh.clone()));
            chunk.should_regenerate_mesh = false;
            chunk.neighbors = world.get_chunk_neighbors(chunk.position());
        }
    }
}

fn rebuild_chunk_meshes(
    mut events: EventReader<RebuildChunkEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    world: Res<World>,
) {
    for event in events.iter() {
        let (chunk_position, handle) = (event.0, event.1.clone());

        let Some(chunk) = world.get_chunk(chunk_position) else {
            continue;
        };

        let mesh = meshes.get_mut(&handle).expect("valid mesh handle");
        *mesh = generate_chunk_mesh(&chunk);
    }
}
