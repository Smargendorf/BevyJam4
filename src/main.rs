use bevy::input::mouse::MouseWheel;
use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use bevy_rand::prelude::*;
use rand_core::RngCore;
use bevy_prng::ChaCha8Rng;

mod camera;
mod components;

use self::components::*;
use self::camera::*;

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const PLAYER_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const ANT_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BALL_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);

const TIME_SCALE: f32 = 2.0;

fn setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>
) {
    // Player
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(PLAYER_COLOR)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        components::Player,
    ));

    for _ in 0..10
    {
        let mass = (rng.next_u32() as i32 % 200 + 15) as f32;
        let transform = Transform::from_translation(Vec3::new(
            (rng.next_u32() as i32 % 500) as f32,
            (rng.next_u32() as i32 % 500) as f32,
            1.0)).with_scale(Vec3::new(mass, mass, 0.0));

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(ANT_COLOR)),
                transform: transform,
                ..default()
            },
            components::Ant
        ));
    }
}

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
    .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .add_systems(Startup, (setup, camera_setup))
    .add_systems(Update, (move_player, camera_chase, scroll_events))
    .add_systems(Update, bevy::window::close_on_esc)
    .add_event::<MouseWheel>()
    .run();
}
