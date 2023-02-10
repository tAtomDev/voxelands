mod data;
mod debug;
mod game;
mod rendering;
mod world;

use bevy::{asset::LoadState, prelude::*, window::PresentMode};
use debug::*;
use game::*;
//use plugin::*;
use rendering::{ChunkMaterial, ChunkTextureAtlas};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: String::from("Voxelands"),
                        present_mode: PresentMode::AutoNoVsync,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(MaterialPlugin::<ChunkMaterial>::default())
        .add_plugin(DebugPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(WorldPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_texture_atlas)
        .add_system(prepare_texture_atlas)
        .run();
}

fn setup_texture_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ChunkTextureAtlas(asset_server.load("atlas.png")));
}

fn prepare_texture_atlas(
    asset_server: Res<AssetServer>,
    atlas: ResMut<ChunkTextureAtlas>,
    mut images: ResMut<Assets<Image>>,
) {
    if asset_server.get_load_state(atlas.0.clone()) != LoadState::Loaded {
        return;
    }

    let image = images.get_mut(&atlas.0).unwrap();
    if image.texture_descriptor.size.depth_or_array_layers != 1 {
        return;
    }

    let array_layers = (image.size().y / image.size().x) as u32;
    image.reinterpret_stacked_2d_as_array(array_layers);
}
