use bevy::{prelude::*, window::CursorGrabMode};

use crate::{data::VoxelType, game::CameraState, world::World};

use super::{
    data::*, generation_plugin::ChunkGenerationPlugin, loading_plugin::ChunkLoadingPlugin,
    meshing_plugin::ChunkMeshingPlugin,
};

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkCommandQueue>()
            .init_resource::<ChunkEntities>()
            .init_resource::<DirtyChunks>()
            .init_resource::<World>()
            .add_plugin(ChunkLoadingPlugin)
            .add_plugin(ChunkGenerationPlugin)
            .add_plugin(ChunkMeshingPlugin)
            .add_system(place_and_remove_voxels);
    }
}

fn place_and_remove_voxels(
    mut dirty_chunks: ResMut<DirtyChunks>,
    mut world: ResMut<World>,
    windows: Res<Windows>,
    mouse_button: Res<Input<MouseButton>>,
    query: Query<&Transform, With<CameraState>>,
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

    dirty_chunks.mark_dirty(chunk_position);
}
