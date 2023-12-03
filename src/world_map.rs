use bevy::{
    math::Vec4,
    prelude::{App, AssetServer, Camera2dBundle, Commands, Res, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
    DefaultPlugins,
};

use bevy::prelude::*;

use bevy_entitiles::{
    math::FillArea,
    render::texture::TilemapTextureDescriptor,
    tilemap::{
        map::{Tilemap, TilemapBuilder},
        tile::{TileBuilder, TileType},
    },
    EntiTilesPlugin,
};

const TILE_SIZE: Vec2 = Vec2::new(16., 16.);
const MAP_SIZE: UVec2 = UVec2::new(100, 100);

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPos>()
        .add_plugins(EntiTilesPlugin)
        .add_systems(Startup, setup)
        .add_systems(First, update_cursor_pos)
        .add_systems(Update, mouse_button_input);
    }
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        MAP_SIZE,
        TILE_SIZE,
    )
    .with_texture(
        assets_server.load("test_square.png"),
        TilemapTextureDescriptor::from_full_grid(
            UVec2 { x: 32, y: 32 },
            UVec2 { x: 2, y: 2 },
            FilterMode::Nearest,
        ),
    )
    .with_translation(Vec2 { x: 8., y: 0. })
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(0),
    );

    commands.entity(tilemap_entity).insert(tilemap);
}

#[derive(Resource)]
pub struct CursorPos(Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

fn world_pos_to_two_d_index(pos: Vec2) -> UVec2
{
    return UVec2::new((pos.x / TILE_SIZE.x) as u32, (pos.y / TILE_SIZE.y) as u32);
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>
) {
    for cursor_moved in cursor_moved_events.iter() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
}

fn mouse_button_input(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    mut tilemap_q: Query<&mut Tilemap>
){
    let mut tilemap = tilemap_q.single_mut();

    eprintln!("{}", world_pos_to_two_d_index(cursor_pos.0));
    if buttons.pressed(MouseButton::Left) {
        tilemap.set(
            &mut commands, 
            world_pos_to_two_d_index(cursor_pos.0),
            &TileBuilder::new(1).with_color(Vec4::new(0.8, 1., 0.8, 0.1))
        )
    }
}
