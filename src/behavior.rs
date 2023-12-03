use crate::*;
use bevy::math::*;
use std::f32::consts::TAU;

// TODO: Does each ant need to be able to identify its own "home this way" pheromone?
#[derive(Clone, Copy)]
pub enum PheromoneKind {
    HomeThisWay,
    FoodThisWay,
}

#[derive(Clone, Copy)]
pub enum AntState {
    Wandering,
    HasFood,
}

impl AntState {
    pub fn pher_to_drop(&self) -> PheromoneKind {
        match self {
            AntState::Wandering => PheromoneKind::HomeThisWay,
            AntState::HasFood => PheromoneKind::FoodThisWay,
        }
    }
}

#[derive(Bundle)]
pub struct PheromoneBundle {
    pheromone: Pheromone,
    transform: Transform,
}

#[derive(Bundle)]
pub struct AntBundle {
    ant: Ant,
    transform: Transform,
}

const PHEROMONE_DECAY_FACTOR: f32 = 1.0;

pub fn decay_pheromones(mut pheromones: Query<&mut Pheromone>, time: Res<Time>) {
    for mut pheromone in pheromones.iter_mut() {
        pheromone.intensity -= time.delta_seconds() * PHEROMONE_DECAY_FACTOR;
    }
}

// bounds can exceed [0, tau] range - this is how "which side of the circle are we on?" can be
// answered
fn angle_within_bounds(theta: f32, bound_lower: f32, bound_upper: f32) -> bool {
    (theta >= bound_lower && theta <= bound_upper)
        || (theta >= bound_lower + TAU && theta <= bound_upper + TAU)
        || (theta >= bound_lower - TAU && theta <= bound_upper - TAU)
}

fn ant_should_follow(ant_state: AntState, pher_kind: PheromoneKind) -> bool {
    match (ant_state, pher_kind) {
        (AntState::Wandering, PheromoneKind::HomeThisWay) => false,
        (AntState::Wandering, PheromoneKind::FoodThisWay) => true,
        (AntState::HasFood, PheromoneKind::HomeThisWay) => true,
        (AntState::HasFood, PheromoneKind::FoodThisWay) => false,
    }
}

fn ant_desired_direction(
    ant: &Ant,
    ant_trans: &Transform,
    pheromones: &Query<(&Transform, &Pheromone), Without<Ant>>,
) -> Vec2 {
    let (ant_dir, _, _) = ant_trans.rotation.to_euler(EulerRot::ZXY);

    let vision_bound_lower = ant_dir + ant.vision_arc / 2.0;
    let vision_bound_upper = ant_dir - ant.vision_arc / 2.0;

    let mut cum_dir = Vec2::ZERO;
    for (pher_trans, pheromone) in pheromones.iter() {
        if !ant_should_follow(ant.state, pheromone.kind) {
            continue;
        }

        let to_pher = (pher_trans.translation - ant_trans.translation).xy();

        if to_pher.length() > ant.vision_range {
            continue;
        }

        let angle = f32::atan2(to_pher.y, to_pher.x);

        if !angle_within_bounds(angle, vision_bound_lower, vision_bound_upper) {
            continue;
        }

        // TODO: Maybe intensity shouldn't cause following to fall off?
        cum_dir += to_pher.normalize() * pheromone.intensity;
    }

    // Keep along the same path
    if cum_dir == Vec2::ZERO {
        cum_dir = (ant_trans.rotation * Vec3::X).xy();
    }

    // TODO apply some random wandering to the ant's chosen direction
    cum_dir.normalize()
}

pub fn update_ant_movement(
    mut ants: Query<(&mut Transform, &Ant), Without<Pheromone>>,
    pheromones: Query<(&Transform, &Pheromone), Without<Ant>>,
    time: Res<Time>,
) {
    for (mut ant_trans, ant) in ants.iter_mut() {
        let chosen_dir = Vec3::from((ant_desired_direction(&ant, &ant_trans, &pheromones), 0.0));
        let new_position = ant_trans.translation + chosen_dir * ant.speed * time.delta_seconds();

        // Do these *before* moving
        ant_trans.rotation = ant_trans.looking_at(new_position, Vec3::Z).rotation;
        ant_trans.translation = Vec3::new(new_position.x, new_position.y, ant_trans.translation.z)

        // TODO change ant state based on findings
        // TODO make ants drop pheromones
    }
}

pub const ANT_POOP_INTERVAL: f32 = 0.5;

pub fn spawn_pheromones(
    mut commands: Commands,
    mut ants: Query<(&Transform, &mut Ant), Without<Pheromone>>,
    time: Res<Time>,
) {
    for (ant_trans, mut ant) in ants.iter_mut() {
        if ant.time_until_poop > 0.0 {
            ant.time_until_poop -= time.delta_seconds() * ant.speed;
        } else {
            commands.spawn(PheromoneBundle {
                pheromone: Pheromone {
                    kind: ant.state.pher_to_drop(),
                    intensity: 10.0,
                },
                transform: ant_trans.clone(),
            });

            ant.time_until_poop = ANT_POOP_INTERVAL;
        }
    }
}
