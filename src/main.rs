use bevy::input::mouse::MouseWheel;
use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand_core::RngCore;
use bevy_prng::ChaCha8Rng;


const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const PLAYER_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const ANT_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BALL_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);

const TIME_SCALE: f32 = 2.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct GameCamera;

#[derive(Component)]
struct Ant;

fn setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>
) {
    // Camera
    commands.spawn((Camera2dBundle::default(), GameCamera));

    // Player
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(PLAYER_COLOR)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        Player,
    ));

    for _ in 0..10
    {
        let mass = (rng.next_u32() as i32 % 200 + 15) as f32;
        let transform = Transform::from_translation(Vec3::new(
            (rng.next_u32() as i32 % 5000) as f32,
            (rng.next_u32() as i32 % 5000) as f32,
            1.0)).with_scale(Vec3::new(mass, mass, 0.0));

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(ANT_COLOR)),
                transform: transform,
                ..default()
            },
            Ant
        ));
    }
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>, 
    mut query: Query<&mut Transform, With<Player>>,
    time_step: Res<FixedTime>,)
{
    let mut player_transform = query.single_mut();
    let mut direction = Vec3::new(0.0, 0.0, 0.0);

    if keyboard_input.pressed(KeyCode::Left) {
        direction.x = -1.0;
    }
    else if keyboard_input.pressed(KeyCode::Right) {
        direction.x = 1.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        direction.y = 1.0;
    }
    else if keyboard_input.pressed(KeyCode::Down) {
        direction.y = -1.0;
    }

    player_transform.translation = player_transform.translation + (direction * 100.0 * time_step.period.as_secs_f32());
}

fn camera_chase(
    playerQuery: Query<&Transform, With<Player>>,
    mut cameraQuery: Query<&mut Transform, (Without<Player>, With<GameCamera>)>,
)
{
    let player_transform = playerQuery.single();
    let mut camera_transform = cameraQuery.single_mut();

    camera_transform.translation = player_transform.translation;
}

fn scroll_events(
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<&mut OrthographicProjection, With<Camera>>, 
    time: Res<Time>
)  {
    let mut projection = query.single_mut();

    let mut delta = 0.0;
    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll_evr.iter() {
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

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
    .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .add_systems(Startup, setup)
    .add_systems(Update, (move_player, camera_chase, scroll_events))
    .add_systems(Update, bevy::window::close_on_esc)
    .add_event::<MouseWheel>()
    .run();
}
