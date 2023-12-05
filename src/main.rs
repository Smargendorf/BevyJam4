use bevy::input::mouse::MouseWheel;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

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

use components::*;
use resources::FoodRes;
use util::*;

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const PLAYER_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const ANT_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
) {
    commands.insert_resource(FoodRes::default());

    // Player
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(PLAYER_COLOR)),
            transform: Transform::from_translation(world_map::world_map_center_3d())
                .with_scale(ANT_SIZE),
            ..default()
        },
        Player,
    ));

    for _ in 0..4 {
        let transform = Transform::from_translation(Vec3::new(
            (rng.next_u32() as i32 % 500) as f32 + world_map::world_map_center().x,
            (rng.next_u32() as i32 % 300) as f32 + world_map::world_map_center().y,
            1.0,
        ))
        .with_scale(ANT_SIZE);

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(Color::ORANGE_RED)),
                transform,
                ..default()
            },
            Food,
        ));
    }

    for _ in 0..500 {
        commands.spawn(behavior::AntBundle {
            ant: Ant::default(),
            transform: Transform::default().with_translation(Vec3::new(
                world_map::world_map_center().x,
                world_map::world_map_center().y,
                0.,
            )),
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
        .add_systems(Startup, behavior::setup_pher_tiles)
        .add_systems(
            Update,
            (
                behavior::update_ant_movement,
                behavior::spawn_pheromones,
                behavior::decay_pheromones,
            ),
        )
        .add_systems(
            Update,
            (
                // behavior::debug_ants,
                // behavior::debug_phers,
                behavior::debug_ants_minimal,
            ),
        )
        .add_event::<MouseWheel>()
        .run();
}
