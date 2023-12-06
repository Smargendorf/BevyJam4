use std::collections::HashMap;
use std::fmt;
use std::ops::{Index, IndexMut};
use strum_macros::EnumIter;

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
pub const MAP_SIZE: UVec2 = UVec2::new(50, 50);
pub const MAP_DATA_SIZE: usize = (MAP_SIZE.x * MAP_SIZE.y) as usize;

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

const HOVER_COLOR: Vec4 = Vec4::new(0., 0., 0., 0.1);
const NORMAL_COLOR: Vec4 = Vec4::new(1., 1., 1., 1.);
const NORMAL_TILE_INDEX: u32 = 0;

#[derive(Component)]
pub struct HoveredTile;

#[derive(PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum BuildingType {
    None,
    Tunnel,
    QueenChamber,
    FoodStorage,
}

impl fmt::Display for BuildingType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildingType::None => write!(f, "None"),
            BuildingType::Tunnel => write!(f, "Tunnel"),
            BuildingType::QueenChamber => write!(f, "Queen"),
            BuildingType::FoodStorage => write!(f, "Food"),
        }
    }
}

#[derive(Component)]
pub struct SelectedZLevel(pub i32);

#[derive(Component)]
pub struct TileMapZLevel(u32);

#[derive(Component)]
pub struct SelectedBuilding {
    pub selected_type: BuildingType,
}

#[derive(Component)]
pub struct ZLevel {
    pub z_level: i32,
    pub tiles: Vec<TileState>, // needs to be heap allocated
}

impl Index<UVec2> for ZLevel {
    type Output = TileState;

    fn index(&self, index: UVec2) -> &Self::Output {
        &self.tiles[two_d_index_to_one_d_index(index).unwrap()]
    }
}

impl IndexMut<UVec2> for ZLevel {
    fn index_mut(&mut self, index: UVec2) -> &mut Self::Output {
        &mut self.tiles[two_d_index_to_one_d_index(index).unwrap()]
    }
}

impl ZLevel {
    fn with_level(level: i32) -> ZLevel {
        let default_tile = TileState::default();
        let mut tiles = vec![];
        for _ in 0..MAP_DATA_SIZE {
            tiles.push(default_tile.clone());
        }
        ZLevel {
            z_level: level,
            tiles,
        }
    }

    pub fn set_area(&mut self, area: URect, building_type: BuildingType) {
        for x in area.min.x..area.max.x {
            for y in area.min.y..area.max.y {
                if let Some(i) = two_d_index_to_one_d_index(UVec2::new(x, y)) {
                    self.tiles[i].building = building_type;
                }
            }
        }
    }

    pub fn is_tile_walkable(&self, pos: UVec2) -> bool {
        match two_d_index_to_one_d_index(pos) {
            Some(i) => return self.tiles[i].building == BuildingType::Tunnel,
            None => return false,
        }
    }
}

#[derive(Clone)]
pub struct TileState {
    pub building: BuildingType,
    pub pher_refs: Vec<Entity>,
}

impl Default for TileState {
    fn default() -> Self {
        Self {
            building: BuildingType::None,
            pher_refs: vec![],
        }
    }
}

#[derive(Component)]
pub struct MapPos(pub UVec2);

#[derive(Component)]
pub struct BuildingTypeToColorMap(HashMap<BuildingType, Vec4>);

#[derive(Component)]
pub struct BuildingTypeToTileIndexMap(HashMap<BuildingType, u32>);

#[derive(Resource)]
pub struct CursorPos(Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

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
    commands.spawn(BuildingTypeToTileIndexMap(HashMap::from([
        (BuildingType::None, 0),
        (BuildingType::Tunnel, 1),
        (BuildingType::QueenChamber, 3),
        (BuildingType::FoodStorage, 2),
    ])));

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
        &TileBuilder::new(NORMAL_TILE_INDEX),
    );

    commands.spawn(SelectedBuilding {
        selected_type: BuildingType::Tunnel,
    });

    commands.spawn(SelectedZLevel(0));

    let mut tiles = Vec::new();
    tiles.resize(
        MAP_SIZE.x as usize * MAP_SIZE.y as usize,
        TileState::default(),
    );

    let mut z_level = ZLevel::with_level(0);
    let starting_tunnel_size = UVec2::new(5, 5);

    z_level.set_area(
        URect::from_corners(
            MAP_SIZE / 2 - starting_tunnel_size,
            MAP_SIZE / 2 + starting_tunnel_size,
        ),
        BuildingType::Tunnel,
    );
    commands.spawn(z_level);

    tilemap.fill_rect(
        &mut commands,
        FillArea::new(
            MAP_SIZE / 2 - starting_tunnel_size,
            Some(starting_tunnel_size * 2),
            &tilemap,
        ),
        &TileBuilder::new(1),
    );
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
    building_type_tile_index_map_q: Query<&BuildingTypeToTileIndexMap>,
) {
    // find buildings for z level
    let selected_z_level = selected_z_level_q.single().0;
    let mut tilemap = tilemap_q.single_mut();
    let building_tile_map = building_type_tile_index_map_q.single();
    for z_level in z_level_q.iter() {
        if z_level.z_level != selected_z_level {
            continue;
        }

        for (entity, hovered_tile_pos) in hovered_tiles_q.iter() {
            if let Some(tilemap_index) = two_d_index_to_one_d_index(hovered_tile_pos.0.xy()) {
                let building = z_level.tiles[tilemap_index].building;
                let tile = building_tile_map.0.get(&building).unwrap();
                tilemap.set(
                    &mut commands,
                    hovered_tile_pos.0.xy(),
                    &TileBuilder::new(*tile).with_color(NORMAL_COLOR),
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
    building_type_tile_index_map_q: Query<&BuildingTypeToTileIndexMap>,
    mut z_level_q: Query<&mut ZLevel>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let selected_z_level = current_z_level_q.single();
    let cursor_map_pos = world_pos_to_two_d_index(cursor_pos.0);

    let selected_building: &SelectedBuilding = selected_building_q.single();
    let building_tile_map = building_type_tile_index_map_q.single();

    let tile = building_tile_map
        .0
        .get(&selected_building.selected_type)
        .unwrap();

    let mut tilemap = tilemap_q.single_mut();

    tilemap.set(
        &mut commands,
        cursor_map_pos.xy(),
        &TileBuilder::new(*tile).with_color(NORMAL_COLOR),
    );

    for mut z_level in z_level_q.iter_mut() {
        if z_level.z_level != selected_z_level.0 {
            continue;
        }

        if let Some(index) = two_d_index_to_one_d_index(cursor_map_pos.xy()) {
            z_level.tiles[index].building = selected_building.selected_type;
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
        selected_building.selected_type = BuildingType::QueenChamber;
    } else if keyboard_input.pressed(KeyCode::Key3) {
        selected_building.selected_type = BuildingType::FoodStorage;
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
    building_type_tile_index_map_q: Query<&BuildingTypeToTileIndexMap>,
) {
    let mut tilemap = tilemap_q.single_mut();
    let mut selected_z_level = selected_z_level_q.single_mut();
    let building_tile_map = building_type_tile_index_map_q.single();

    if keyboard_input.just_pressed(KeyCode::BracketRight) {
        selected_z_level.0 += 1;
    } else if keyboard_input.just_pressed(KeyCode::BracketLeft) {
        selected_z_level.0 -= 1;
    } else {
        // no change return early
        return;
    }

    eprint!("Z:{}", selected_z_level.0);

    // the zlevel changed so rerender the tile map

    // first blank the whole tilemap
    let fill_area = FillArea::full(&tilemap);
    let tile_empty = building_tile_map
        .0
        .get(&BuildingType::None)
        .cloned()
        .unwrap();
    tilemap.fill_rect(&mut commands, fill_area, &TileBuilder::new(tile_empty));

    // try to find the buildings with the matching z level
    let mut found_z_layer = false;
    for z_level in z_level_q.iter() {
        if z_level.z_level != selected_z_level.0 {
            continue;
        }

        found_z_layer = true;

        for i_tile in 0..z_level.tiles.len() {
            let building_type = z_level.tiles[i_tile].building;
            let tile = building_tile_map.0.get(&building_type).cloned().unwrap();

            tilemap.set(
                &mut commands,
                one_d_index_to_two_d_index(i_tile),
                &TileBuilder::new(tile).with_color(NORMAL_COLOR),
            );
        }
    }

    // if we didn't find an existing z layer make one
    if !found_z_layer {
        let mut tiles = Vec::new();
        tiles.resize(
            MAP_SIZE.x as usize * MAP_SIZE.y as usize,
            TileState::default(),
        );
        commands.spawn(ZLevel::with_level(selected_z_level.0));
    }
}
