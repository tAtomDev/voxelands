use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics, EntityCountDiagnosticsPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiContext, quick::WorldInspectorPlugin};

use crate::world::World;

use super::CameraState;

#[derive(Resource, Default)]
pub struct DebugSettings {
    wireframe_rendering: bool,
}

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WireframePlugin)
            .add_plugin(WorldInspectorPlugin)
            .add_plugin(EntityCountDiagnosticsPlugin)
            .init_resource::<DebugSettings>()
            .insert_resource(Diagnostics::default())
            .add_startup_system(setup)
            .add_system(toggle_wireframe_mode)
            .add_system(diagnostic_system)
            .add_system(update_ui)
            .add_system(ui_camera);
    }
}

fn toggle_wireframe_mode(
    keys: Res<Input<KeyCode>>,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut settings: ResMut<DebugSettings>,
) {
    if keys.just_pressed(KeyCode::F1) {
        settings.wireframe_rendering = !settings.wireframe_rendering;
        wireframe_config.global = settings.wireframe_rendering;
    }
}

const DIAGNOSTIC_FPS: DiagnosticId = DiagnosticId::from_u128(0);
const DIAGNOSTIC_FRAME_TIME: DiagnosticId = DiagnosticId::from_u128(1);

fn setup(mut diagnostics: ResMut<Diagnostics>) {
    diagnostics.add(Diagnostic::new(DIAGNOSTIC_FRAME_TIME, "frame_time", 20).with_suffix("ms"));
    diagnostics.add(Diagnostic::new(DIAGNOSTIC_FPS, "fps", 20));
}

fn diagnostic_system(mut diagnostics: ResMut<Diagnostics>, time: Res<Time>) {
    let delta_seconds = time.raw_delta_seconds_f64();
    if delta_seconds == 0.0 {
        return;
    }

    diagnostics.add_measurement(DIAGNOSTIC_FRAME_TIME, || delta_seconds * 1000.0);

    diagnostics.add_measurement(DIAGNOSTIC_FPS, || 1.0 / delta_seconds);
}

fn update_ui(mut ctx: ResMut<EguiContext>, diagnostics: Res<Diagnostics>, world: Res<World>) {
    let egui_context = ctx.ctx_mut().clone();

    egui::Window::new("UI").show(&egui_context, |ui| {
        ui.heading("Debug");

        ui.label(format!(
            "Entity count: {}",
            diagnostics
                .get_measurement(EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                .map(|d| d.value)
                .unwrap_or_default()
        ));

        ui.label(format!("Chunk count: {}", world.chunks().len()));

        ui.label(format!(
            "Frame Time: {}ms",
            diagnostics
                .get_measurement(DIAGNOSTIC_FRAME_TIME)
                .map(|d| d.value.round() as i32)
                .unwrap_or_default()
        ));

        ui.label(format!(
            "FPS: {}",
            diagnostics
                .get_measurement(DIAGNOSTIC_FPS)
                .map(|d| d.value.round() as i32)
                .unwrap_or_default()
        ))
    });
}

fn ui_camera(mut ctx: ResMut<EguiContext>, camera: Query<&Transform, With<CameraState>>) {
    let egui_context = ctx.ctx_mut().clone();
    let transform = camera.single();
    let pos = transform.translation;

    egui::Window::new("Camera UI").show(&egui_context, |ui| {
        ui.heading("Position");

        ui.label(format!(
            "[GLOBAL] X: {}, Y: {}, Z: {}",
            pos.x as i32, pos.y as i32, pos.z as i32
        ));
        ui.label(format!(
            "[WORLD] X: {}, Y: {}, Z: {}",
            (pos.x as i32).div_euclid(32),
            (pos.y as i32).div_euclid(32),
            (pos.z as i32).div_euclid(32)
        ));
    });
}
