use bevy::prelude::*;

use crate::behavior::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Ant {
    pub state: AntState,
    pub vision_range: f32,
    pub vision_arc: f32, // in radians
}

impl Default for Ant {
    fn default() -> Self {
        Ant {
            state: AntState::Wandering,
            vision_range: 1.0,
            vision_arc: 2.0,
        }
    }
}

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Pheromone {
    pub kind: PheromoneKind,
    pub intensity: f32,
}
