use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    winit::WinitSettings,
};

use strum::IntoEnumIterator;

use crate::world_map::*;

#[derive(Component)]
pub struct ZLevelLabel;

#[derive(Component)]
pub struct BuildingTypeButton(BuildingType);

pub struct WorldUIPlugin;
impl Plugin for WorldUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(Update, (building_type_selection_update, update_z_level_ui, button_system));
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // left vertical fill (border)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_shrink: 1.0,
                        width: Val::Percent(50.),
                        border: UiRect::all(Val::Px(2.)),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0.65, 0.65, 0.65, 0.4).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // text
                    parent.spawn((
                        TextBundle::from_section(
                            "Z:",
                            TextStyle {
                                //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 30.0,
                                color: Color::rgb(0.1, 0.1, 0.1),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(5.)),
                            align_self: AlignSelf::Start,
                            ..default()
                        }),
                        Label,
                        ZLevelLabel,
                    ));
                    
                    for building_type in BuildingType::iter() {
                        parent.spawn(
                            (ButtonBundle {
                                style: Style {
                                    height: Val::Px(65.0),
                                    border: UiRect::all(Val::Px(5.0)),
                                    // horizontally center child text
                                    justify_content: JustifyContent::Center,
                                    // vertically center child text
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                border_color: BorderColor(Color::BLACK),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            BuildingTypeButton(building_type)
                        ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    building_type.to_string(),
                                    TextStyle {
                                        //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 40.0,
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                        ..default()
                                    },
                                ));
                            });
                    }
                });
        });
}

fn update_z_level_ui(
    selected_z_level_q: Query<&SelectedZLevel>,
    mut z_level_label_q: Query<&mut Text, With<ZLevelLabel>>,
) {
    let mut label = z_level_label_q.single_mut();
    let z_level: &SelectedZLevel = selected_z_level_q.single();
    if let Some(text) = label.sections.first_mut() {
        text.value = format!("Z:{}", z_level.0);
    }
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &BuildingTypeButton,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut selected_building_q: Query<&mut SelectedBuilding>,
) {
    let mut selected_building = selected_building_q.single_mut();
    for (interaction, mut color, mut border_color, children, building_type) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                text.sections[0].value = building_type.0.to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
                selected_building.selected_type = building_type.0;
            }
            Interaction::Hovered => {
                text.sections[0].value = building_type.0.to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                text.sections[0].value = building_type.0.to_string();
                
                if selected_building.selected_type== building_type.0 {
                    *color = PRESSED_BUTTON.into();
                }
                else {
                    *color = NORMAL_BUTTON.into();
                }

                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn building_type_selection_update(
    selected_building_q: Query<&SelectedBuilding>,
    mut building_type_button_query: Query<(&mut BackgroundColor, &BuildingTypeButton), With<Button>>
) {
    let selected_building = selected_building_q.single();
    for (mut color, building_type) in &mut building_type_button_query {
        if selected_building.selected_type == building_type.0 {
            //println!("{} {}", selected_building.selected_type, building_type.0);
            *color = PRESSED_BUTTON.into();
        }
        else {
            *color = NORMAL_BUTTON.into();
        }
    }
}