use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use crate::{
    data::VoxelType,
    plugin::CameraState,
    rendering::{ChunkMaterial, ChunkTextureAtlas},
    world::{meshing, Chunk, World},
};
use bevy::{
    prelude::*, render::primitives::Aabb, tasks::AsyncComputeTaskPool, window::CursorGrabMode,
};
use futures_lite::future::{block_on, poll_once};

use super::{components::*, events::*, systems::update_nearby_chunks};

const VIEW_DISTANCE: i32 = 8;
const VIEW_DISTANCE_SQUARED: i32 = VIEW_DISTANCE * VIEW_DISTANCE;
const VIEW_DISTANCE_HALF: i32 = VIEW_DISTANCE / 2;

#[derive(Resource, Default)]
pub struct WorldQueues {
    pub chunk_load_queue: VecDeque<IVec3>,
    pub chunk_unload_queue: VecDeque<IVec3>,
}

pub struct WorldWorkerPlugin;
impl Plugin for WorldWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldQueues>()
            .add_system(despawn_chunks)
            .add_system_to_stage(CoreStage::PreUpdate, prepare_chunk_tasks)
            .add_system(apply_chunk_tasks)
            .add_system(apply_rebuild_tasks)
            .add_system(update_nearby_chunks)
            .add_system(enqueue_chunks)
            .add_system(break_tiles);
    }
}

fn despawn_chunks(
    mut commands: Commands,
    mut world: ResMut<World>,
    camera: Query<&Transform, With<CameraState>>,
    visible_query: Query<(Entity, &ChunkComponent)>,
) {
    let player = World::world_to_chunk_position(camera.single().translation.as_ivec3());

    for (entity, chunk_component) in visible_query.iter() {
        if distance(player, chunk_component.0) > VIEW_DISTANCE as f32 {
            continue;
        }

        world.remove_chunk(chunk_component.0);
        commands.entity(entity).despawn_recursive();
    }
}

fn prepare_chunk_tasks(mut commands: Commands, mut queues: ResMut<WorldQueues>) {
    let thread_pool = AsyncComputeTaskPool::get();

    if let Some(chunk_position) = queues.chunk_load_queue.pop_front() {
        let task = thread_pool.spawn(async move { Chunk::generate_at(chunk_position) });

        commands.spawn(ChunkLoadTask(task));
    }
}

fn apply_chunk_tasks(
    mut commands: Commands,
    mut world: ResMut<World>,
    mut tasks: Query<(Entity, &mut ChunkLoadTask)>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut nearby_events: EventWriter<UpdateNearbyChunks>,
    atlas: ResMut<ChunkTextureAtlas>,
) {
    for (entity, mut task) in &mut tasks {
        let chunk_task = block_on(poll_once(&mut task.0));
        let Some(chunk_task) = chunk_task else {
            continue;
        };

        commands.entity(entity).despawn();

        let Some(chunk) = chunk_task else {
            continue;
        };

        let position = chunk.position();
        let chunk = Arc::new(RwLock::new(chunk));
        chunk.write().unwrap().neighbors = world.get_chunk_neighbors(position);

        let chunk_arc_clone = chunk.clone();

        let chunk = chunk.read().unwrap();

        commands
            .spawn(ChunkComponent(chunk.position()))
            .insert(MaterialMeshBundle {
                transform: Transform::from_xyz(
                    chunk.world_position().x,
                    chunk.world_position().y,
                    chunk.world_position().z,
                ),
                material: materials.add(ChunkMaterial {
                    texture_atlas: atlas.0.clone(),
                }),
                mesh: meshes.add(meshing::generate_empty_chunk_mesh()),
                ..Default::default()
            });

        world.set_chunk(position, chunk_arc_clone);

        nearby_events.send(UpdateNearbyChunks(chunk.position()));
    }
}

fn apply_rebuild_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkRebuildTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut task) in &mut tasks {
        let mesh_task = block_on(poll_once(&mut task.0));
        let Some(mesh) = mesh_task else {
            continue;
        };

        commands.entity(entity).despawn();

        commands.entity(task.3).remove::<Aabb>();

        *meshes.get_mut(&task.2).unwrap() = mesh;
    }
}

fn break_tiles(
    mut nearby_events: EventWriter<UpdateNearbyChunks>,
    windows: Res<Windows>,
    mouse_button: Res<Input<MouseButton>>,
    query: Query<&Transform, With<CameraState>>,
    world: Res<World>,
) {
    let Some(window) = windows.get_primary() else {
        return;
    };

    if window.cursor_grab_mode() != CursorGrabMode::Confined {
        return;
    }

    let (left_pressed, right_pressed) = (
        mouse_button.pressed(MouseButton::Left),
        mouse_button.pressed(MouseButton::Right),
    );
    if !left_pressed && !right_pressed {
        return;
    }

    let transform = query.single();

    let Some(hit) = world.raytrace(transform.translation, transform.forward(), 30.0) else {
        return;
    };

    let chunk_position = World::world_to_chunk_position(hit.voxel_position);

    if left_pressed {
        world.set_voxel(VoxelType::Air, hit.voxel_position);
    }

    if right_pressed {
        let voxel_position = hit.voxel_position + hit.face.normal();
        world.set_voxel(VoxelType::Stone, voxel_position);
    }

    nearby_events.send(UpdateNearbyChunks(chunk_position));
}

fn enqueue_chunks(
    mut query: Query<(&Transform, &mut CameraState)>,
    mut queues: ResMut<WorldQueues>,
    world: Res<World>,
) {
    let (transform, mut camera_state) = query.single_mut();

    if !camera_state.should_load_chunks {
        return;
    }

    camera_state.should_load_chunks = false;

    let player_chunk_position = World::world_to_chunk_position(transform.translation.as_ivec3());
    let valid_positions =
        get_valid_chunk_positions(player_chunk_position, VIEW_DISTANCE, VIEW_DISTANCE_HALF);

    enqueue_valid_chunks(&mut queues, &world, &valid_positions);
    enqueue_invalid_chunks(&mut queues, &world, &valid_positions);
    sort_chunk_queues(queues, player_chunk_position);
}

fn get_valid_chunk_positions(
    player_chunk_position: IVec3,
    view_distance: i32,
    view_distance_half: i32,
) -> Vec<IVec3> {
    let mut valid_positions = Vec::new();
    for x in -view_distance..=view_distance {
        for z in -view_distance..=view_distance {
            if x.pow(2) + z.pow(2) >= VIEW_DISTANCE_SQUARED {
                continue;
            }

            for y in -view_distance_half..=view_distance_half {
                let offset = IVec3::new(x, y, z);
                let chunk_position = player_chunk_position + offset;
                valid_positions.push(chunk_position);
            }
        }
    }

    valid_positions
}

fn enqueue_valid_chunks(
    queues: &mut ResMut<WorldQueues>,
    world: &Res<World>,
    valid_positions: &[IVec3],
) {
    for chunk_position in valid_positions {
        if !queues.chunk_load_queue.contains(chunk_position) && !world.chunk_exists(*chunk_position)
        {
            queues.chunk_load_queue.push_back(*chunk_position);
        }
    }

    queues
        .chunk_load_queue
        .retain(|pos| valid_positions.contains(pos));
}

fn enqueue_invalid_chunks(
    queues: &mut ResMut<WorldQueues>,
    world: &Res<World>,
    valid_positions: &[IVec3],
) {
    let mut chunks_to_unload = Vec::new();
    for chunk_position in world.chunks().keys() {
        if queues.chunk_unload_queue.contains(chunk_position) {
            continue;
        }

        if valid_positions.contains(chunk_position) {
            continue;
        }

        chunks_to_unload.push(*chunk_position);
    }

    queues.chunk_unload_queue.extend(chunks_to_unload);
}

fn sort_chunk_queues(mut queues: ResMut<WorldQueues>, base_position: IVec3) {
    queues
        .chunk_load_queue
        .make_contiguous()
        .sort_unstable_by_key(|key| (base_position - *key).as_vec3().length() as i32);

    queues
        .chunk_unload_queue
        .make_contiguous()
        .sort_unstable_by_key(|key| (base_position - *key).as_vec3().length() as i32);
}

fn distance(from: IVec3, to: IVec3) -> f32 {
    (from.as_vec3() - to.as_vec3()).powf(2.0).length_squared()
}
