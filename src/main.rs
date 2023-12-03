use bevy::input::mouse::MouseWheel;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use bevy_entitiles::EntiTilesPlugin;

use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use rand_core::RngCore;

mod behavior;
mod camera;
mod components;
mod resources;
mod sprite;
mod util;
mod world_map;

use camera::*;
use components::*;
use resources::Food;
use util::*;

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const PLAYER_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const ANT_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);

const CAMERA_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const ANT_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);

const TIME_SCALE: f32 = 2.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
) {
    commands.insert_resource(Food::default());

    // Player
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(PLAYER_COLOR)),
            transform: Transform::from_translation(CAMERA_STARTING_POSITION).with_scale(ANT_SIZE),
            ..default()
        },
        Player,
    ));

    for _ in 0..32 {
        let transform = Transform::from_translation(Vec3::new(
            (rng.next_u32() as i32 % 500) as f32,
            (rng.next_u32() as i32 % 300) as f32,
            1.0,
        ))
        .with_scale(ANT_SIZE);

        commands.spawn(behavior::AntBundle {
            ant: Ant::default(),
            transform,
            rng: rng.fork_rng(),
        });
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::default())
        .add_plugins(camera::CameraPlugin)
        .add_plugins(sprite::AnimationTestPlugin)
        .add_plugins(world_map::WorldMapPlugin)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            FixedUpdate,
            (behavior::update_ant_movement, behavior::spawn_pheromones),
        )
        .add_systems(
            Update,
            (
                behavior::debug_ants,
                behavior::debug_phers,
                behavior::decay_pheromones,
            ),
        )
        .add_event::<MouseWheel>()
        .run();
}
