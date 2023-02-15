use std::sync::Arc;

use bevy::{prelude::*, render::primitives::Aabb, tasks::AsyncComputeTaskPool};
use futures_lite::future;

use crate::{
    data::constants::*,
    rendering::*,
    world::{meshing, World},
};

use super::data::*;

fn prepare_new_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    new_chunks: Query<(Entity, &ChunkComponent), Added<ChunkComponent>>,
    texture_atlas: Res<ChunkTextureAtlas>,
) {
    for (entity, chunk_component) in new_chunks.iter() {
        let chunk_position = chunk_component.0;
        let chunk_world_position = (chunk_position * CHUNK_SIZE_I32).as_vec3();

        commands
            .entity(entity)
            .insert(MaterialMeshBundle {
                transform: Transform::from_translation(chunk_world_position),
                material: materials.add(ChunkMaterial {
                    texture_atlas: texture_atlas.0.clone_weak(),
                }),
                mesh: meshes.add(meshing::generate_empty_chunk_mesh()),
                visibility: Visibility::INVISIBLE,
                ..Default::default()
            })
            .insert(Aabb::from_min_max(
                Vec3::ZERO,
                Vec3::splat(CHUNK_SIZE_I32 as f32),
            ));
    }
}

fn queue_chunk_meshing(
    mut commands: Commands,
    dirty_chunks: Res<DirtyChunks>,
    chunk_entities: Res<ChunkEntities>,
    world: Res<World>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    for chunk_position in dirty_chunks.iter_dirty() {
        let entity = chunk_entities.entity(chunk_position).unwrap();
        let chunk = world.get_chunk(*chunk_position).unwrap().clone();

        let task = task_pool.spawn(async move { meshing::generate_chunk_mesh(&chunk) });

        commands.entity(entity).insert(ChunkMeshingTask(task));
    }
}

fn process_mesh_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<
        (
            Entity,
            &Handle<Mesh>,
            &mut ChunkMeshingTask,
            &mut Visibility,
        ),
        With<ChunkComponent>,
    >,
) {
    for (entity, handle, mut task, mut visibility) in &mut query {
        if let Some(mesh) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).remove::<ChunkMeshingTask>();
            *meshes.get_mut(handle).unwrap() = mesh;
            visibility.is_visible = true;
        }
    }
}

pub struct ChunkMeshingPlugin;
impl Plugin for ChunkMeshingPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            ChunkGenerationStage,
            ChunkMeshingPrepareStage,
            SystemStage::single(prepare_new_chunks),
        )
        .add_stage_after(
            ChunkMeshingPrepareStage,
            ChunkMeshingStage,
            SystemStage::parallel()
                .with_system(queue_chunk_meshing)
                .with_system(process_mesh_tasks.after(queue_chunk_meshing)),
        );
    }
}
