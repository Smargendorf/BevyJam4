use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Food(u64);
