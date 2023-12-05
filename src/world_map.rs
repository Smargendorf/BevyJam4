use std::collections::HashMap;

use bevy::{math::Vec4, render::render_resource::FilterMode};

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

pub const TILE_SIZE: Vec2 = Vec2::new(16., 16.);
pub const MAP_SIZE: UVec2 = UVec2::new(100, 100);

pub fn world_map_size() -> Vec2 {
    return TILE_SIZE * (MAP_SIZE.as_vec2());
}

pub fn world_map_center() -> Vec2 {
    return world_map_size() / 2.;
}
pub fn world_map_center_3d() -> Vec3 {
    let center = world_map_center();
    return Vec3::new(center.x, center.y, 0.);
}

const TUNNEL_COLOR: Vec4 = Vec4::new(0.15, 0.1, 0., 1.);
const QUEEN_CHAMBER_COLOR: Vec4 = Vec4::new(0.73, 0.12, 63., 1.);
const FOOD_STORAGE_COLOR: Vec4 = Vec4::new(0.2, 0.73, 0.12, 1.);

const HOVER_COLOR: Vec4 = Vec4::new(0., 0., 0., 0.1);

const NORMAL_COLOR: Vec4 = Vec4::new(1., 1., 1., 1.);
const NORMAL_TILE_INDEX: u32 = 1;

#[derive(Component)]
pub struct HoveredTile;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum BuildingType {
    None,
    Tunnel,
    QueenChamber,
    FoodStorage,
}

#[derive(Component)]
pub struct SelectedZLevel(u32);

#[derive(Component)]
pub struct TileMapZLevel(u32);

#[derive(Component)]
pub struct SelectedBuilding {
    selected_type: BuildingType,
}

#[derive(Component)]
pub struct Building(BuildingType);

#[derive(Component)]
pub struct ZLevel {
    z_level: u32,
    buildings: Vec<BuildingType>,
}

#[derive(Component)]
pub struct MapPos(pub UVec2);

#[derive(Component)]
pub struct BuildingTypeToColorMap(HashMap<BuildingType, Vec4>);

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPos>()
            .add_plugins(EntiTilesPlugin)
            .add_systems(Startup, setup)
            .add_systems(
                First,
                (
                    update_cursor_pos,
                    reset_hovered_tiles,
                    change_selected_building_type,
                    change_selected_z_level,
                ),
            )
            .add_systems(Update, (mouse_building, mouse_hover));
    }
}

fn setup(mut commands: Commands, assets_server: Res<AssetServer>) {
    let (_, mut tilemap) = TilemapBuilder::new(TileType::Square, MAP_SIZE, TILE_SIZE)
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

    commands.spawn(SelectedBuilding {
        selected_type: BuildingType::Tunnel,
    });

    commands.spawn(SelectedZLevel(0));

    let mut buildings = Vec::new();
    buildings.resize(
        MAP_SIZE.x as usize * MAP_SIZE.y as usize,
        BuildingType::None,
    );
    commands.spawn(ZLevel {
        z_level: 0,
        buildings: buildings,
    });

    commands.spawn(BuildingTypeToColorMap(HashMap::from([
        (BuildingType::None, NORMAL_COLOR),
        (BuildingType::Tunnel, TUNNEL_COLOR),
        (BuildingType::QueenChamber, QUEEN_CHAMBER_COLOR),
        (BuildingType::FoodStorage, FOOD_STORAGE_COLOR),
    ])));
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

pub fn get_local_neighborhood(world_pos: Vec2) -> Vec<UVec2> {
    vec![
        Vec2::new(-1.0, -1.0),
        Vec2::new(0.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(-1.0, 0.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
    ]
    .iter()
    .map(|offset| world_pos + *offset)
    .filter(|npos| {
        npos.x >= 0.0
            && npos.x < MAP_SIZE.x as f32 * TILE_SIZE.x
            && npos.y >= 0.0
            && npos.y < MAP_SIZE.y as f32 * TILE_SIZE.y
    })
    .map(|npos| world_pos_to_two_d_index(npos))
    .collect()
}

pub fn world_pos_to_two_d_index(pos: Vec2) -> UVec2 {
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
    hovered_tiles_q: Query<(Entity, &MapPos), With<HoveredTile>>,
    selected_z_level_q: Query<&SelectedZLevel>,
    z_level_q: Query<&ZLevel>,
    building_type_color_map_q: Query<&BuildingTypeToColorMap>,
) {
    // find buildings for z level
    let selected_z_level = selected_z_level_q.single().0;
    let mut tilemap = tilemap_q.single_mut();
    let building_color_map = building_type_color_map_q.single();
    for z_level in z_level_q.iter() {
        if z_level.z_level != selected_z_level {
            continue;
        }

        for (entity, hovered_tile_pos) in hovered_tiles_q.iter() {
            if let Some(tilemap_index) = two_d_index_to_one_d_index(hovered_tile_pos.0.xy()) {
                let building_type = z_level.buildings[tilemap_index];
                let color = building_color_map.0.get(&building_type).unwrap();
                tilemap.set(
                    &mut commands,
                    hovered_tile_pos.0.xy(),
                    &TileBuilder::new(0).with_color(*color),
                );

                commands.entity(entity).despawn();
            }
        }
        break;
    }
}

fn mouse_hover(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    mut tilemap_q: Query<&mut Tilemap>,
) {
    if buttons.pressed(MouseButton::Left) {
        return;
    }

    let mut tilemap = tilemap_q.single_mut();
    let cursor_map_pos = world_pos_to_two_d_index(cursor_pos.0);
    tilemap.set(
        &mut commands,
        cursor_map_pos.xy(),
        &TileBuilder::new(NORMAL_TILE_INDEX).with_color(HOVER_COLOR),
    );

    commands.spawn((HoveredTile, MapPos(cursor_map_pos)));
}

fn mouse_building(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    mut tilemap_q: Query<&mut Tilemap>,
    selected_building_q: Query<&mut SelectedBuilding>,
    current_z_level_q: Query<&SelectedZLevel>,
    building_type_color_map_q: Query<&BuildingTypeToColorMap>,
    mut z_level_q: Query<&mut ZLevel>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let selected_z_level = current_z_level_q.single();
    let cursor_map_pos = world_pos_to_two_d_index(cursor_pos.0);

    let selected_building: &SelectedBuilding = selected_building_q.single();
    let building_color_map = building_type_color_map_q.single();
    let new_tile_color: Vec4 = building_color_map
        .0
        .get(&selected_building.selected_type)
        .cloned()
        .unwrap();

    let mut tilemap = tilemap_q.single_mut();

    tilemap.set(
        &mut commands,
        cursor_map_pos.xy(),
        &TileBuilder::new(NORMAL_TILE_INDEX).with_color(new_tile_color),
    );

    for mut z_level in z_level_q.iter_mut() {
        if z_level.z_level != selected_z_level.0 {
            continue;
        }

        if let Some(index) = two_d_index_to_one_d_index(cursor_map_pos.xy()) {
            z_level.buildings[index] = selected_building.selected_type;
        }
    }
}

fn change_selected_building_type(
    keyboard_input: Res<Input<KeyCode>>,
    mut selected_building_q: Query<&mut SelectedBuilding>,
) {
    let mut selected_building = selected_building_q.single_mut();
    if keyboard_input.pressed(KeyCode::Key1) {
        selected_building.selected_type = BuildingType::Tunnel;
    } else if keyboard_input.pressed(KeyCode::Key2) {
        selected_building.selected_type = BuildingType::FoodStorage;
    } else if keyboard_input.pressed(KeyCode::Key3) {
        selected_building.selected_type = BuildingType::QueenChamber;
    }
}

fn one_d_index_to_two_d_index(index: usize) -> UVec2 {
    return UVec2::new(index as u32 % MAP_SIZE.x, index as u32 / MAP_SIZE.x);
}

fn two_d_index_to_one_d_index(index: UVec2) -> Option<usize> {
    if index.x < MAP_SIZE.x && index.y < MAP_SIZE.y {
        Some((index.y * MAP_SIZE.x + index.x) as usize)
    } else {
        None
    }
}

fn change_selected_z_level(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut selected_z_level_q: Query<&mut SelectedZLevel>,
    mut tilemap_q: Query<&mut Tilemap>,
    z_level_q: Query<&ZLevel>,
    building_type_color_map_q: Query<&BuildingTypeToColorMap>,
) {
    let mut tilemap = tilemap_q.single_mut();
    let mut selected_z_level = selected_z_level_q.single_mut();
    let building_color_map = building_type_color_map_q.single();

    if keyboard_input.just_pressed(KeyCode::BracketRight) {
        selected_z_level.0 += 1;
    } else if keyboard_input.just_pressed(KeyCode::BracketLeft) {
        // dont go below 0
        if selected_z_level.0 == 0 {
            return;
        }

        selected_z_level.0 -= 1;
    } else {
        // no change return early
        return;
    }

    eprint!("Z:{}", selected_z_level.0);

    // the zlevel changed so rerender the tile map

    // first blank the whole tilemap
    let fill_area = FillArea::full(&tilemap);
    tilemap.fill_rect(&mut commands, fill_area, &TileBuilder::new(0));

    // try to find the buildings with the matching z level
    let mut found_z_layer = false;
    for z_level in z_level_q.iter() {
        if z_level.z_level != selected_z_level.0 {
            continue;
        }

        found_z_layer = true;

        for i_building in 0..z_level.buildings.len() {
            let building_type = z_level.buildings[i_building];
            let color = building_color_map.0.get(&building_type).cloned().unwrap();

            tilemap.set(
                &mut commands,
                one_d_index_to_two_d_index(i_building),
                &TileBuilder::new(0).with_color(color),
            );
        }
    }

    // if we didn't find an existing z layer make one
    if !found_z_layer {
        let mut buildings = Vec::new();
        buildings.resize(
            MAP_SIZE.x as usize * MAP_SIZE.y as usize,
            BuildingType::None,
        );
        commands.spawn(ZLevel {
            z_level: selected_z_level.0,
            buildings: buildings,
        });
    }
}
