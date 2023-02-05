use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_inspector_egui::bevy_egui::EguiContext;

use crate::world::World;

#[derive(Component)]
pub struct CameraState {
    pub sensibility: f32,
    pub fov: f32,
    pub speed: f32,
    pub rotation_velocity: Vec2,
    pub rotation: Vec2,
    pub velocity: Vec3,
    pub should_load_chunks: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            sensibility: 70.0,
            fov: 70.0,
            speed: 120.0,
            rotation_velocity: Vec2::ZERO,
            rotation: Vec2::ZERO,
            velocity: Vec3::ZERO,
            should_load_chunks: true,
        }
    }
}
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MouseMotion>()
            .add_startup_system(build_camera)
            .add_system(rotate_camera)
            .add_system(cursor_grab)
            .add_system(move_camera);
    }
}

fn build_camera(mut commands: Commands) {
    commands
        .spawn(CameraState::default())
        .insert(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 32.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: CameraState::default().fov,
                ..Default::default()
            }),
            ..Default::default()
        });
}

fn move_camera(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut CameraState)>,
) {
    let (mut transform, mut state) = query.single_mut();
    let start_chunk_position = World::world_to_chunk_position(transform.translation.as_ivec3());
    let is_running = keys.pressed(KeyCode::LShift);

    let forward = transform.forward().normalize_or_zero();
    let right = forward.cross(Vec3::Y);

    let mut direction = Vec3::ZERO;
    if keys.pressed(KeyCode::W) {
        direction += forward;
    } else if keys.pressed(KeyCode::S) {
        direction -= forward;
    }

    if keys.pressed(KeyCode::A) {
        direction -= right;
    } else if keys.pressed(KeyCode::D) {
        direction += right;
    }

    let velocity =
        direction.normalize_or_zero() * state.speed * if is_running { 10.0 } else { 1.0 };
    state.velocity = velocity;

    transform.translation += velocity * time.delta_seconds();

    let end_chunk_position = World::world_to_chunk_position(transform.translation.as_ivec3());

    if start_chunk_position != end_chunk_position {
        state.should_load_chunks = true;
    }
}

fn rotate_camera(
    mut windows: ResMut<Windows>,
    mut query: Query<(&mut Transform, &mut CameraState)>,
    mut motion_event: EventReader<MouseMotion>,
) {
    let Some(window) = windows.get_primary_mut() else {
        return;
    };

    let (mut camera_transform, mut camera_state) = query.single_mut();

    for ev in motion_event.iter() {
        match window.cursor_grab_mode() {
            CursorGrabMode::None => (),
            _ => {
                let window_scale = window.height().min(window.width());

                let mouse_rotation =
                    ev.delta * (camera_state.sensibility * 0.000001) * window_scale;
                camera_state.rotation_velocity.y += mouse_rotation.y.to_radians();
                camera_state.rotation_velocity.x += mouse_rotation.x.to_radians();

                window.set_cursor_position(Vec2::new(window.width() / 2.0, window.height() / 2.0));
            }
        }
    }

    let velocity = camera_state.rotation_velocity;
    camera_state.rotation -= velocity;
    camera_state.rotation_velocity = camera_state.rotation_velocity.lerp(Vec2::ZERO, 0.5);

    camera_state.rotation.y = camera_state.rotation.y.clamp(-1.54, 1.54);

    camera_transform.rotation = Quat::from_axis_angle(Vec3::Y, camera_state.rotation.x)
        * Quat::from_axis_angle(Vec3::X, camera_state.rotation.y);
}

fn cursor_grab(
    mut windows: ResMut<Windows>,
    mut egui: ResMut<EguiContext>,
    keys: Res<Input<KeyCode>>,
    mouse_button: Res<Input<MouseButton>>,
) {
    let Some(window) = windows.get_primary_mut() else {
        return;
    };

    if !window.is_focused() {
        return toggle_grab_cursor(window, false);
    }

    let egui_context = egui.ctx_mut();

    if !egui_context.wants_pointer_input() && mouse_button.just_pressed(MouseButton::Left) {
        toggle_grab_cursor(window, true);
    } else if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(window, false);
    }
}

fn toggle_grab_cursor(window: &mut Window, toggle: bool) {
    match toggle {
        true => {
            window.set_cursor_grab_mode(CursorGrabMode::Confined);
            window.set_cursor_visibility(false);
        }
        _ => {
            window.set_cursor_grab_mode(CursorGrabMode::None);
            window.set_cursor_visibility(true);
        }
    }
}
