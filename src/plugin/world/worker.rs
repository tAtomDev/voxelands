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

use super::{components::*, events::*, systems::update_nearby_chunks};

const VIEW_DISTANCE: i32 = 8;
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
            .add_system(prepare_chunk_tasks)
            .add_system(apply_chunk_tasks)
            .add_system(prepare_rebuild_tasks)
            .add_system(apply_rebuild_tasks)
            .add_system(update_nearby_chunks)
            .add_system(enqueue_chunks);
    }
}

fn despawn_chunks(
    mut commands: Commands,
    mut query: Query<&Transform, With<CameraState>>,
    mut world: ResMut<World>,
    visible_query: Query<(Entity, &ChunkComponent)>,
) {
    let transform = query.single_mut();

    let center = World::world_to_chunk_position(transform.translation.as_ivec3());

    visible_query.for_each(|(entity, chunk_component)| {
        let chunk_position = chunk_component.0;
        if distance(center - chunk_position) >= VIEW_DISTANCE {
            world.remove_chunk(chunk_position);
            commands.entity(entity).despawn_recursive();
        }
    });
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
    mut world: ResMut<World>,
) {
    for (entity, mut task) in &mut tasks {
        let chunk_task = block_on(poll_once(&mut task.0));
        let Some(chunk_task) = chunk_task else {
            return;
        };

        commands.entity(entity).despawn();

        let Some(chunk) = chunk_task else {
            return;
        };

        let position = chunk.position();
        let chunk = Arc::new(RwLock::new(chunk));

        world.set_chunk(position, chunk.clone());

        events.send(SpawnChunkEvent(chunk.clone()));
    }
}

fn prepare_rebuild_tasks(
    mut commands: Commands,
    mut events: EventReader<RebuildChunkEvent>,
    mut world: ResMut<World>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for event in events.iter() {
        let (chunk_position, mesh) = (event.0, event.1.clone());
        if !world.chunk_exists(chunk_position) {
            return;
        }

        world.update_chunk_neighbors(chunk_position);
        let chunk = world.get_chunk_arc(chunk_position);
        let task = thread_pool.spawn(async move {
            let chunk = chunk.read().unwrap();
            generate_chunk_mesh(&chunk)
        });

        commands.spawn(ChunkRebuildTask(task, chunk_position, mesh));
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

        *meshes.get_mut(&task.2).unwrap() = mesh;
    }
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

    let mut valid_positions = Vec::new();

    for x in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for z in -VIEW_DISTANCE..=VIEW_DISTANCE {
            for y in -VIEW_DISTANCE_HALF..=VIEW_DISTANCE_HALF {
                let offset = IVec3::new(x, y, z);
                if x * x + z * z >= VIEW_DISTANCE * VIEW_DISTANCE {
                    continue;
                }

                let chunk_position = player_chunk_position + offset;

                if !queues.chunk_load_queue.contains(&chunk_position)
                    && !world.chunk_exists(chunk_position)
                {
                    queues.chunk_load_queue.push_back(chunk_position);
                }

                valid_positions.push(chunk_position);
            }
        }
    }

    queues
        .chunk_load_queue
        .retain(|pos| valid_positions.contains(pos));

    sort_chunk_queues(&mut queues, player_chunk_position.as_vec3());
}

fn sort_chunk_queues(queues: &mut ResMut<WorldQueues>, base_position: Vec3) {
    queues
        .chunk_load_queue
        .make_contiguous()
        .sort_unstable_by_key(|key| FloatOrd(key.as_vec3().distance(base_position)));
}

fn distance(p: IVec3) -> i32 {
    p.max_element().abs().max(p.min_element().abs())
}
