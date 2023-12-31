use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::components::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup)
            .add_systems(Update, (move_player, scroll_events, camera_chase));
    }
}

pub fn camera_setup(
    mut commands: Commands,
) {
    // Camera
    commands.spawn((Camera2dBundle::default(), GameCamera));
}

pub fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time_step: Res<Time>,
) {
    let mut player_transform = query.single_mut();
    let mut direction = Vec3::new(0.0, 0.0, 0.0);

    if keyboard_input.pressed(KeyCode::Left) {
        direction.x = -1.0;
    } else if keyboard_input.pressed(KeyCode::Right) {
        direction.x = 1.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        direction.y = 1.0;
    } else if keyboard_input.pressed(KeyCode::Down) {
        direction.y = -1.0;
    }

    player_transform.translation =
        player_transform.translation + (direction * 100.0 * time_step.delta_seconds());
}

pub fn camera_chase(
    player_q: Query<&Transform, With<Player>>,
    mut camera_q: Query<&mut Transform, (Without<Player>, With<GameCamera>)>,
) {
    let player_transform = player_q.single();
    let mut camera_transform = camera_q.single_mut();

    camera_transform.translation = player_transform.translation;
}

pub fn scroll_events(
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    time: Res<Time>,
) {
    let mut projection = query.single_mut();

    let mut delta = 0.0;
    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll_evr.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                delta += ev.y;
            }
            MouseScrollUnit::Pixel => {
                delta += ev.y;
            }
        }
    }

    let mut log_scale = projection.scale.ln();
    log_scale -= delta * time.delta_seconds();
    projection.scale = log_scale.exp();
}
