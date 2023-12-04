use std::collections::HashMap;

use bevy::{
    math::Vec4,
    prelude::{App, AssetServer, Startup, UVec2, Vec2},
    render::render_resource::FilterMode,
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

const TUNNEL_COLOR: Vec4 = Vec4::new(0.15, 0.1, 0., 1.);
const QUEEN_CHAMBER_COLOR: Vec4 = Vec4::new(0.73, 0.12, 63., 1.);
const FOOD_STORAGE_COLOR: Vec4 = Vec4::new(0.2, 0.73, 0.12, 1.);
const BUILDING_TILE_INDEX: u32 = 1;

const HOVER_COLOR: Vec4 = Vec4::new(0., 0., 0., 0.1);

const NORMAL_COLOR: Vec4 = Vec4::new(1., 1., 1., 1.);
const NORMAL_TILE_INDEX: u32 = 1;

#[derive(Component)]
pub struct HoveredTile;

#[derive(PartialEq,Eq,Hash,Clone, Copy)]
enum BuildingType
{
    Tunnel,
    QueenChamber,
    FoodStorage
}

#[derive(Component)]
pub struct SelectedBuilding {
    selected_type: BuildingType
}

#[derive(Component)]
pub struct Building(BuildingType);

#[derive(Component)]
pub struct TileColor(Vec4);

#[derive(Component)]
pub struct TileIndex(u32);

#[derive(Component)]
pub struct MapPos(UVec2);

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPos>()
            .add_plugins(EntiTilesPlugin)
            .add_systems(Startup, setup)
            .add_systems(First, (update_cursor_pos, reset_hovered_tiles, change_selected_building_type))
            .add_systems(Update, mouse_button_input);
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
        .with_translation(Vec2 { x: 8., y: 0. })
        .build(&mut commands);

    tilemap.fill_rect(
        &mut commands,
        FillArea::full(&tilemap),
        &TileBuilder::new(0),
    );

    commands.entity(tilemap_entity).insert(tilemap);

    commands.spawn(SelectedBuilding {
        selected_type: BuildingType::Tunnel
    });
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

fn world_pos_to_two_d_index(pos: Vec2) -> UVec2 {
    return UVec2::new((pos.x / TILE_SIZE.x) as u32, (pos.y / TILE_SIZE.y) as u32);
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
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

fn reset_hovered_tiles(
    mut commands: Commands,
    mut tilemap_q: Query<&mut Tilemap>,
    hovered_tiles_q: Query<(Entity, &MapPos, &TileColor, &TileIndex), With<HoveredTile>>
) {
    let mut tilemap = tilemap_q.single_mut();
    for (entity, hovered_tile_pos, tile_color, tile_index) in hovered_tiles_q.iter() {
        tilemap.set(
            &mut commands,
            hovered_tile_pos.0,
            &TileBuilder::new(tile_index.0).with_color(tile_color.0),
        );

        commands.entity(entity).remove::<HoveredTile>();
    } 
}

fn mouse_button_input(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    mut tilemap_q: Query<&mut Tilemap>,
    mut tile_q: Query<(Entity, &MapPos, &mut TileColor, &mut TileIndex, Option<&mut Building>)>,
    selected_building_q: Query<&mut SelectedBuilding>
) {
    let selected_building = selected_building_q.single();
    let mut building_type_to_tile_color = HashMap::from([
        (BuildingType::Tunnel, TUNNEL_COLOR),
        (BuildingType::QueenChamber, QUEEN_CHAMBER_COLOR),
        (BuildingType::FoodStorage, FOOD_STORAGE_COLOR),
    ]);

    let mut tilemap = tilemap_q.single_mut();

    // calculate the position of the cursor in tile map coords
    let cursor_map_pos = world_pos_to_two_d_index(cursor_pos.0);

    let mut new_tile_state = false;
    let mut new_tile_color = NORMAL_COLOR;
    let mut new_tile_index = NORMAL_TILE_INDEX;

    let display_tile_color: Vec4;
    let mut display_tile_index = NORMAL_TILE_INDEX;

    if buttons.pressed(MouseButton::Left) {
        new_tile_color = *building_type_to_tile_color.entry(selected_building.selected_type).or_default();
        display_tile_color = *building_type_to_tile_color.entry(selected_building.selected_type).or_default();
        new_tile_index = BUILDING_TILE_INDEX;
        display_tile_index = BUILDING_TILE_INDEX;
        new_tile_state = true;
    } 
    else {
        display_tile_color = HOVER_COLOR;
    }

    tilemap.set(
        &mut commands,
        cursor_map_pos,
        &TileBuilder::new(display_tile_index).with_color(display_tile_color),
    );

    // first check to see if we already have an tile to use
    for (entity, tile_pos, mut tile_color, mut tile_index, _) in tile_q.iter_mut() {
        if tile_pos.0 == cursor_map_pos {
            if buttons.pressed(MouseButton::Left)
            {
                commands.entity(entity).insert(Building(selected_building.selected_type));
            }
            else
            {
                commands.entity(entity).insert(HoveredTile);
            }
            
            if new_tile_state {
                tile_color.0 = new_tile_color;
                tile_index.0 = new_tile_index;
            }

            return;
        }
    }

    // if we got here then we didn't have a tile already so we have to spawn one
    if buttons.pressed(MouseButton::Left)
    {
        commands.spawn((
            Building(BuildingType::Tunnel), 
            MapPos(cursor_map_pos),
            TileColor(new_tile_color),
            TileIndex(new_tile_index)
        ));
    }
    else
    {
        commands.spawn((
            HoveredTile, 
            MapPos(cursor_map_pos),
            TileColor(new_tile_color),
            TileIndex(new_tile_index)
        ));
    }
}

pub fn change_selected_building_type(
    keyboard_input: Res<Input<KeyCode>>,
    mut selected_building_q: Query<&mut SelectedBuilding>
) {
    let mut selected_building = selected_building_q.single_mut();
    if keyboard_input.pressed(KeyCode::Key1) {
        selected_building.selected_type = BuildingType::Tunnel;
    }
    else if keyboard_input.pressed(KeyCode::Key2) {
        selected_building.selected_type = BuildingType::FoodStorage;
    }
    else if keyboard_input.pressed(KeyCode::Key3) {
        selected_building.selected_type = BuildingType::QueenChamber;
    }
}
