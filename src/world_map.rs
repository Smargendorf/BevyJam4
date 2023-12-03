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
const MAP_SIZE: UVec2 = UVec2::new(20, 10);

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPos>()
            .add_plugins(EntiTilesPlugin)
            .add_systems(Startup, setup)
            .add_systems(First, update_cursor_pos);
    }
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(TileType::Square, MAP_SIZE, TILE_SIZE)
        .with_texture(
            assets_server.load("test_square.png"),
            TilemapTextureDescriptor::from_full_grid(
                UVec2 { x: 32, y: 32 },
                UVec2 { x: 2, y: 2 },
                FilterMode::Nearest,
            ),
        )
        .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(0),
    );

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(UVec2 { x: 2, y: 2 }, Some(UVec2 { x: 10, y: 7 }), &tilemap),
        &TileBuilder::new(1).with_color(Vec4::new(0.8, 1., 0.8, 0.1)),
    );

    commands.entity(tilemap_entity).insert(tilemap);

    let (tilemap_entity, mut tilemap) = TilemapBuilder::new(
        TileType::Square,
        UVec2 { x: 20, y: 10 },
        Vec2 { x: 16., y: 16. },
    )
    .with_translation(Vec2 { x: 0., y: -300. })
    .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(0).with_color(Vec4::new(1., 1., 0., 1.)),
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

fn TwoDIndexToOneDIndex(index: UVec2) -> usize {
    let linear_index = (index.y * MAP_SIZE.x + index.x) as usize;
    return linear_index;
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.iter() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
                // eprintln!("{}", cursor_pos.0);
                // eprintln!("{}", UVec2::new(cursor_pos.0.x as u32, cursor_pos.0.y as u32));
                // eprintln!("{}", TwoDIndexToOneDIndex(UVec2::new(cursor_pos.0.x as u32, cursor_pos.0.y as u32)));
            }
        }
    }
}

fn mouse_button_input(mut tilemap_q: Query<&mut Tilemap>) {
    //let mut tilemap = tilemap_q.single_mut();
    //tilemap.tiles;
}

// // fn mouse_button_input(
// //     mut commands: Commands,
// //     cursor_pos: Res<CursorPos>,
// //     buttons: Res<Input<MouseButton>>,
// //     tilemap_q: Query<(
// //         &TilemapSize,
// //         &TilemapGridSize,
// //         &TilemapType,
// //         &TileStorage,
// //         &Transform,
// //     )>,
// // ){
// //     if buttons.just_pressed(MouseButton::Left) {
// //         for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
// //             // Grab the cursor position from the `Res<CursorPos>`
// //             let cursor_pos: Vec2 = cursor_pos.0;
// //             // We need to make sure that the cursor's world position is correct relative to the map
// //             // due to any map transformation.
// //             let cursor_in_map_pos: Vec2 = {
// //                 // Extend the cursor_pos vec3 by 0.0 and 1.0
// //                 let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
// //                 let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
// //                 cursor_in_map_pos.xy()
// //             };
// //             // Once we have a world position we can transform it into a possible tile position.
// //             if let Some(tile_pos) =
// //                 TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
// //             {
// //                 eprint!("{}, {} |", tile_pos.x, tile_pos.y);
// //                 if let Some(mut tile_entity) = tile_storage.get(&tile_pos) {
// //                     //tile_entity.
// //                     //tile_entity.get_mut::<Position>().unwrap();
// //                     commands.entity(tile_entity).add(|w:&mut EntityWorldMut| {

// //                     });
// //                 }
// //             }
// //         }
// //     }
// // }
