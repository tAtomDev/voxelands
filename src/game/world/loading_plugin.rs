use bevy::{prelude::*, utils::FloatOrd};

use crate::{game::CameraState, world::World};

use super::data::*;

pub fn destroy_chunks(
    mut commands: Commands,
    mut chunk_command_queue: ResMut<ChunkCommandQueue>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut world: ResMut<World>,
) {
    for position in chunk_command_queue.destroy.drain(..) {
        let entity = chunk_entities.detach_entity(&position).unwrap();
        commands.entity(entity).despawn();

        world.remove_chunk(position);
    }
}

pub fn create_chunks(
    mut commands: Commands,
    mut chunk_command_queue: ResMut<ChunkCommandQueue>,
    mut chunk_entities: ResMut<ChunkEntities>,
) {
    for position in chunk_command_queue.create.drain(..) {
        let entity = commands.spawn(ChunkComponent(position));
        chunk_entities.attach_entity(position, entity.id());
    }
}

const VIEW_DISTANCE: i32 = 8;
const HALF_VIEW_DISTANCE: i32 = 4;

pub fn update_chunks_within_view_distance(
    mut camera: Query<(&Transform, &mut CameraState)>,
    mut chunk_command_queue: ResMut<ChunkCommandQueue>,
    chunk_entities: Res<ChunkEntities>,
) {
    let (transform, mut state) = camera.single_mut();
    if !state.should_load_chunks {
        return;
    }

    state.should_load_chunks = false;

    let player_position = World::world_to_chunk_position(transform.translation.as_ivec3());

    let mut within_view_distance_positions = vec![];

    chunk_command_queue.create.clear();
    chunk_command_queue.destroy.clear();

    for x in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for z in -VIEW_DISTANCE..=VIEW_DISTANCE {
            if x.pow(2) + z.pow(2) >= VIEW_DISTANCE.pow(2) {
                continue;
            }

            for y in -HALF_VIEW_DISTANCE..=HALF_VIEW_DISTANCE {
                let chunk_position = player_position + IVec3::new(x, y, z);

                if chunk_entities.entity(&chunk_position).is_none() {
                    chunk_command_queue.create.push(chunk_position);
                }

                within_view_distance_positions.push(chunk_position);
            }
        }
    }

    for loaded_chunk_position in chunk_entities.keys() {
        if !within_view_distance_positions.contains(loaded_chunk_position) {
            chunk_command_queue.destroy.push(*loaded_chunk_position);
        }
    }

    chunk_command_queue
        .create
        .sort_unstable_by_key(|key| FloatOrd(key.as_vec3().distance(player_position.as_vec3())));
}

pub fn clear_dirty_chunks(mut dirty_chunks: ResMut<DirtyChunks>) {
    dirty_chunks.clear();
}

pub struct ChunkLoadingPlugin;
impl Plugin for ChunkLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::Update,
            ChunkLoadingStage,
            SystemStage::parallel()
                .with_system(update_chunks_within_view_distance)
                .with_system(create_chunks.after(update_chunks_within_view_distance)),
        )
        .add_system_to_stage(CoreStage::Last, destroy_chunks)
        .add_system_to_stage(CoreStage::Last, clear_dirty_chunks);
    }
}
