use bevy::prelude::*;

use crate::behavior::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Ant {
    pub state: AntState,
    pub speed: f32,
    pub vision_range: f32,
    pub vision_arc: f32, // in radians
    pub time_until_poop: f32,
    pub secret_desire: Vec2,
}

impl Default for Ant {
    fn default() -> Self {
        Ant {
            state: AntState::Wandering,
            speed: 50.0,
            vision_range: 20.0,
            vision_arc: 1.5,
            time_until_poop: ANT_POOP_INTERVAL,
            secret_desire: Vec2::ZERO,
        }
    }
}

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct Food;
