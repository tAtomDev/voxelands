use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use crate::{
    plugin::CameraState,
    world::{meshing::generate_chunk_mesh, Chunk, World},
};
use bevy::{prelude::*, tasks::AsyncComputeTaskPool, utils::FloatOrd};
use futures_lite::future::{block_on, poll_once};

use super::{components::*, events::*};

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
            .add_system(prepare_chunk_tasks)
            .add_system(apply_chunk_tasks)
            .add_system(prepare_rebuild_tasks)
            .add_system(apply_rebuild_tasks)
            .add_system(enqueue_chunks);
    }
}

fn despawn_chunks(mut queues: ResMut<WorldQueues>, mut events: EventWriter<DespawnChunkEvent>) {
    while let Some(chunk_position) = queues.chunk_unload_queue.pop_front() {
        events.send(DespawnChunkEvent(chunk_position));
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
    mut tasks: Query<(Entity, &mut ChunkLoadTask)>,
    mut events: EventWriter<SpawnChunkEvent>,
) {
    for (entity, mut task) in &mut tasks {
        commands.entity(entity).despawn();

        let chunk_task = block_on(poll_once(&mut task.0));
        let Some(chunk) = chunk_task else {
            return;
        };

        let Some(chunk) = chunk else {
            return;
        };

        events.send(SpawnChunkEvent(Arc::new(RwLock::new(chunk))));
    }
}

fn prepare_rebuild_tasks(
    mut commands: Commands,
    mut events: EventReader<RebuildChunkEvent>,
    world: Res<World>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for event in events.iter() {
        let (chunk_position, mesh) = (event.0, event.1.clone());

        let chunk = world.get_chunk_arc(chunk_position);
        chunk.write().unwrap().neighbors = world.get_chunk_neighbors(chunk_position);
        let task = thread_pool.spawn(async move {
            let chunk = chunk.read().unwrap();
            generate_chunk_mesh(&chunk)
        });

        commands.spawn(ChunkRebuildTask(task, mesh));
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
            return;
        };

        commands.entity(entity).despawn();

        *meshes.get_mut(&task.1).unwrap() = mesh;
    }
}

fn enqueue_chunks(
    mut query: Query<(&Transform, &mut CameraState)>,
    mut queues: ResMut<WorldQueues>,
    world: Res<World>,
    input: Res<Input<KeyCode>>,
) {
    let (transform, mut camera_state) = query.single_mut();

    if !camera_state.should_load_chunks || !input.just_pressed(KeyCode::Space) {
        return;
    }

    camera_state.should_load_chunks = false;

    let player_chunk_position = World::world_to_chunk_position(transform.translation.as_ivec3());

    let mut positions_to_load = Vec::new();
    let mut valid_positions: Vec<IVec3> = vec![];

    const VIEW_DISTANCE: i32 = 8;
    const VIEW_DISTANCE_SQUARED: i32 = VIEW_DISTANCE * VIEW_DISTANCE;

    for x in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for y in -VIEW_DISTANCE..=VIEW_DISTANCE {
            for z in -VIEW_DISTANCE..=VIEW_DISTANCE {
                if x * x + y * y + z * z >= VIEW_DISTANCE_SQUARED {
                    continue;
                }

                let chunk_position = player_chunk_position + IVec3::new(x, y, z);

                if !world.chunk_exists(chunk_position) {
                    positions_to_load.push(chunk_position);
                }
                valid_positions.push(chunk_position);
            }
        }
    }

    enqueue_valid_chunks(
        &mut queues,
        positions_to_load,
        valid_positions,
        world.chunks().keys().copied().collect(),
    );
    sort_chunk_queues(&mut queues, player_chunk_position.as_vec3());
}

fn enqueue_valid_chunks(
    queues: &mut ResMut<WorldQueues>,
    positions_to_load: Vec<IVec3>,
    valid_positions: Vec<IVec3>,
    all_positions: Vec<IVec3>,
) {
    queues.chunk_load_queue = positions_to_load.clone().into();

    // Remove from unload queue
    queues.chunk_unload_queue = all_positions
        .iter()
        .copied()
        .filter(|p| !valid_positions.contains(p))
        .collect();
}

fn sort_chunk_queues(queues: &mut ResMut<WorldQueues>, base_position: Vec3) {
    queues
        .chunk_load_queue
        .make_contiguous()
        .sort_unstable_by_key(|key| FloatOrd(key.as_vec3().distance(base_position)));

    queues
        .chunk_unload_queue
        .make_contiguous()
        .sort_unstable_by_key(|key| FloatOrd(key.as_vec3().distance(base_position)));
}
