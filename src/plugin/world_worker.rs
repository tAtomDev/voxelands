use std::{collections::VecDeque, sync::Arc};

use crate::world::{Chunk, World};
use bevy::{prelude::*, tasks::Task, utils::FloatOrd};

use super::{CameraState, DespawnChunkEvent, SpawnChunkEvent};

#[derive(Resource, Default)]
pub struct WorldQueues {
    pub chunk_load_queue: VecDeque<Arc<Chunk>>,
    pub chunk_unload_queue: VecDeque<IVec3>,
}

#[derive(Component)]
pub struct MeshTask {
    pub task: Task<Vec<Mesh>>,
}

pub struct WorldWorkerPlugin;
impl Plugin for WorldWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldQueues>()
            .add_system(enqueue_chunks)
            .add_system(spawn_chunks)
            .add_system(despawn_chunks);
    }
}

//fn prepare_mesh_tasks(mut queues: ResMut<WorldQueues>) {
//    while let Some(chunk) = queues.chunk_load_queue.pop_back()
//}

fn spawn_chunks(mut queues: ResMut<WorldQueues>, mut events: EventWriter<SpawnChunkEvent>) {
    let Some(chunk) = queues.chunk_load_queue.pop_front() else {
        return;
    };

    events.send(SpawnChunkEvent(chunk));
}

fn despawn_chunks(mut queues: ResMut<WorldQueues>, mut events: EventWriter<DespawnChunkEvent>) {
    let Some(chunk_position) = queues.chunk_unload_queue.pop_front() else {
        return;
    };

    events.send(DespawnChunkEvent(chunk_position));
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

    let mut positions_within_view_distance: Vec<IVec3> = vec![];
    const VIEW_DISTANCE: i32 = 4;
    const VIEW_DISTANCE_SQUARED: f32 = (VIEW_DISTANCE * VIEW_DISTANCE) as f32;

    for x in -VIEW_DISTANCE..VIEW_DISTANCE {
        for y in -VIEW_DISTANCE..VIEW_DISTANCE {
            for z in -VIEW_DISTANCE..VIEW_DISTANCE {
                let chunk_position = player_chunk_position + IVec3::new(x, y, z);

                if (chunk_position - player_chunk_position).as_vec3().length_squared() > VIEW_DISTANCE_SQUARED {
                    continue;
                }

                enqueue_chunk_at(&mut queues, chunk_position, &world);

                positions_within_view_distance.push(chunk_position);
            }
        }
    }

    unload_chunks(&mut queues, positions_within_view_distance, world);
    sort_chunk_load_queue(&mut queues, player_chunk_position);
    sort_chunk_unload_queue(&mut queues, player_chunk_position);
}

fn enqueue_chunk_at(queues: &mut ResMut<WorldQueues>, position: IVec3, world: &Res<World>) -> bool {
    if queues
        .chunk_load_queue
        .iter()
        .any(|c| c.position() == position)
    {
        return false;
    }

    let Some(chunk) = world.generate_chunk(position) else {
        return false;
    };

    println!("Enqueued chunk at position {}", position);

    let chunk = Arc::new(chunk);
    queues.chunk_load_queue.push_back(chunk);

    if let Some(unload_queue_index) = queues
        .chunk_unload_queue
        .iter()
        .position(|c| c == &position)
    {
        queues.chunk_unload_queue.remove(unload_queue_index);
    }

    true
}

fn sort_chunk_load_queue(queues: &mut ResMut<WorldQueues>, base_position: IVec3) {
    queues
        .chunk_load_queue
        .make_contiguous()
        .sort_unstable_by_key(|key| {
            FloatOrd(key.position().as_vec3().distance(base_position.as_vec3()))
        });
}

fn sort_chunk_unload_queue(queues: &mut ResMut<WorldQueues>, base_position: IVec3) {
    queues
        .chunk_unload_queue
        .make_contiguous()
        .sort_unstable_by_key(|key| FloatOrd(base_position.as_vec3().distance(key.as_vec3())));
}

pub fn unload_chunks(
    queues: &mut ResMut<WorldQueues>,
    valid_positions: Vec<IVec3>,
    world: Res<World>,
) {
    for chunk_position in world.chunks().keys() {
        if !valid_positions.contains(chunk_position)
            && !queues.chunk_unload_queue.contains(chunk_position)
        {
            let position = queues
                .chunk_load_queue
                .iter()
                .position(|c| c.position() == *chunk_position);
            if let Some(index) = position {
                queues.chunk_load_queue.remove(index);
            } else {
                queues.chunk_unload_queue.push_back(*chunk_position);
            }
        }
    }
}
