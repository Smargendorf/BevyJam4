use crate::*;
use bevy::{ecs::system::Despawn, math::*};
use std::f32::consts::{PI, TAU};

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
    pub ant: Ant,
    pub transform: Transform,
    pub rng: EntropyComponent<ChaCha8Rng>,
}

const PHEROMONE_DECAY_FACTOR: f32 = 0.1;

pub fn decay_pheromones(
    mut commands: Commands,
    mut pheromones: Query<(Entity, &mut Pheromone)>,
    time: Res<Time>,
) {
    for (entity, mut pheromone) in pheromones.iter_mut() {
        pheromone.intensity -= time.delta_seconds() * PHEROMONE_DECAY_FACTOR;

        if pheromone.intensity < 0.0 {
            commands.add(Despawn { entity })
        }
    }
}

// bounds can exceed [0, tau] range - this is how "which side of the circle are we on?" can be
// answered
fn angle_within_bounds(theta: f32, bound_lower: f32, bound_upper: f32) -> bool {
    !theta.is_nan()
        && ((theta >= bound_lower && theta <= bound_upper)
            || (theta >= bound_lower + TAU && theta <= bound_upper + TAU)
            || (theta >= bound_lower - TAU && theta <= bound_upper - TAU))
}

fn ant_should_follow(ant_state: AntState, pher_kind: PheromoneKind) -> bool {
    match (ant_state, pher_kind) {
        (AntState::Wandering, PheromoneKind::HomeThisWay) => false,
        (AntState::Wandering, PheromoneKind::FoodThisWay) => true,
        (AntState::HasFood, PheromoneKind::HomeThisWay) => true,
        (AntState::HasFood, PheromoneKind::FoodThisWay) => false,
    }
}

const RANDOM_WALK_FACTOR: f32 = 0.3;

fn ant_desired_direction(
    ant: &mut Ant,
    ant_trans: &Transform,
    rng: &mut EntropyComponent<ChaCha8Rng>,
    pheromones: &Query<(&Transform, &Pheromone), (Without<Ant>, Without<Food>)>,
) -> Vec2 {
    let dir_vec = (ant_trans.rotation * Vec3::Y).xy();
    let ant_dir = (f32::atan2(dir_vec.y, dir_vec.x) + TAU) % TAU;

    let vision_bound_lower = ant_dir - ant.vision_arc / 2.0;
    let vision_bound_upper = ant_dir + ant.vision_arc / 2.0;

    let angle_offset = rand_uniform_f32(rng) * PI;
    let length_offset = rand_uniform_f32(rng) + 1.0 / 2.0 * RANDOM_WALK_FACTOR;
    let random_offset =
        (Quat::from_euler(EulerRot::ZYX, angle_offset, 0.0, 0.0) * (Vec3::X * length_offset)).xy();

    ant.secret_desire += random_offset;

    let mut cum_dir = ant.secret_desire;
    for (pher_trans, pheromone) in pheromones.iter() {
        if !ant_should_follow(ant.state, pheromone.kind) {
            continue;
        }

        let to_pher = (pher_trans.translation - ant_trans.translation).xy();

        if to_pher.length() > ant.vision_range {
            continue;
        }

        let angle = (f32::atan2(to_pher.y, to_pher.x) + TAU) % TAU;

        if !angle_within_bounds(angle, vision_bound_lower, vision_bound_upper) {
            continue;
        }

        cum_dir += to_pher.normalize() * 5.0;
    }

    // Keep along the same path
    if cum_dir.length() < 1e-3 {
        cum_dir = (ant_trans.rotation * Vec3::Y).xy();
    }

    // Can happen if rotation has not yet been set
    if cum_dir.length() < 1e-3 {
        cum_dir = Vec2::X;
    }

    cum_dir.normalize()
}

const DETECTION_RADIUS: f32 = 20.0;

pub fn update_ant_movement(
    mut ants: Query<
        (&mut Transform, &mut Ant, &mut EntropyComponent<ChaCha8Rng>),
        (Without<Food>, Without<Pheromone>),
    >,
    pheromones: Query<(&Transform, &Pheromone), (Without<Ant>, Without<Food>)>,
    food: Query<&Transform, (With<Food>, Without<Ant>, Without<Pheromone>)>,
    time: Res<Time>,
) {
    for (mut ant_trans, mut ant, mut rng) in ants.iter_mut() {
        match ant.state {
            AntState::Wandering => {
                for food in food.iter() {
                    if (ant_trans.translation - food.translation).length() <= DETECTION_RADIUS {
                        ant.state = AntState::HasFood;
                        ant.secret_desire *= -1.0;
                        ant_trans.rotation *= Quat::from_euler(EulerRot::ZXY, PI, 0.0, 0.0);
                    }
                }
            }
            AntState::HasFood => {
                if ant_trans.translation.length() < DETECTION_RADIUS {
                    ant.state = AntState::Wandering;
                }
            }
        }

        let chosen_dir = Vec3::from((
            ant_desired_direction(&mut ant, &ant_trans, &mut rng, &pheromones),
            0.0,
        ));

        let mut actual_offset = chosen_dir * ant.speed * time.delta_seconds();

        let potential_position = ant_trans.translation + actual_offset;

        if potential_position.x > 500.0 || potential_position.x < -500.0 {
            actual_offset.x *= -1.0;
            ant.secret_desire.x *= -1.0;
        }
        if potential_position.y > 300.0 || potential_position.y < -300.0 {
            actual_offset.y *= -1.0;
            ant.secret_desire.y *= -1.0;
        }

        // Do these *before* moving
        ant_trans.rotation = ant_trans.looking_at(potential_position, Vec3::Z).rotation;
        ant_trans.translation += actual_offset;

        // TODO change ant state based on findings
    }
}

pub const ANT_POOP_INTERVAL: f32 = 2.0;

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
                    intensity: 1.0,
                },
                transform: ant_trans.clone(),
            });

            ant.time_until_poop = ANT_POOP_INTERVAL;
        }
    }
}

pub fn debug_ants(ants: Query<(&Ant, &Transform)>, mut gizmos: Gizmos) {
    for (_ant, ant_trans) in ants.iter() {
        let facing = ant_trans.rotation * Vec3::Y * 2.0;

        let start = ant_trans.translation - facing;
        let end = ant_trans.translation + facing;

        gizmos.line_2d(start.xy(), end.xy(), Color::WHITE);
        gizmos
            .circle_2d(ant_trans.translation.xy(), 5.0, Color::WHITE)
            .segments(16);
    }
}

pub fn debug_phers(phers: Query<(&Pheromone, &Transform)>, mut gizmos: Gizmos) {
    for (pher, pher_trans) in phers.iter() {
        gizmos
            .circle_2d(
                pher_trans.translation.xy(),
                1.0,
                match pher.kind {
                    PheromoneKind::HomeThisWay => Color::BLUE,
                    PheromoneKind::FoodThisWay => Color::GREEN,
                } * pher.intensity,
            )
            .segments(8);
    }
}
