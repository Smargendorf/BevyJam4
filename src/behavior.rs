use crate::*;
use bevy::{ecs::system::Despawn, math::*};
use std::f32::consts::{PI, TAU};

use world_map::*;

#[derive(Component)]
pub struct Pheromone {
    pub kind: PheromoneKind,
    pub intensity: f32,
    pub death_timer: f32,
}

#[derive(Component)]
pub struct PheromoneTileGroup(Vec<Entity>);

#[derive(Clone, Copy, Eq, PartialEq)]
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
    for (entity, mut pher) in pheromones.iter_mut() {
        let decay = time.delta_seconds() * PHEROMONE_DECAY_FACTOR;
        pher.intensity -= decay;
        pher.death_timer -= decay;

        if pher.death_timer < 0.0 {
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
const FOOD_HINT_THRESHOLD: f32 = 50.0;
const HINT_FACTOR: f32 = 0.3;

fn ant_desired_direction(
    ant: &mut Ant,
    ant_trans: &Transform,
    rng: &mut EntropyComponent<ChaCha8Rng>,
    food: &Query<&Transform, (With<Food>, Without<Ant>, Without<Pheromone>)>,
    phers_tiled: &mut Query<(&mut PheromoneTileGroup, &MapPos)>,
    phers: &Query<(&Transform, &Pheromone), (Without<Ant>, Without<Food>)>,
) -> Vec2 {
    let dir_vec = ant_trans.forward().xy();
    let ant_dir = (f32::atan2(dir_vec.y, dir_vec.x) + TAU) % TAU;

    let vision_bound_lower = ant_dir - ant.vision_arc / 2.0;
    let vision_bound_upper = ant_dir + ant.vision_arc / 2.0;

    let angle_offset = rand_uniform_f32(rng) * PI;
    let length_offset = rand_uniform_f32(rng) + 1.0 / 2.0 * RANDOM_WALK_FACTOR;
    let random_offset =
        (Quat::from_euler(EulerRot::ZYX, angle_offset, 0.0, 0.0) * (Vec3::X * length_offset)).xy();

    ant.secret_desire += random_offset * 0.5;

    // attractive force from pheromones

    let mut cum_dir = ant.secret_desire;
    let neighborhood = get_local_neighborhood(ant_trans.translation.xy());

    for (mut local_phers, tile_pos) in phers_tiled.iter_mut() {
        if !neighborhood.contains(&tile_pos.0) {
            continue;
        }

        local_phers.0.retain(|pher_ent| {
            if let Ok((pher_trans, pheromone)) = phers.get(*pher_ent) {
                if !ant_should_follow(ant.state, pheromone.kind) {
                    return true;
                }

                let to_pher = (pher_trans.translation - ant_trans.translation).xy();

                if to_pher.length() > ant.vision_range {
                    return true;
                }

                let angle = (f32::atan2(to_pher.y, to_pher.x) + TAU) % TAU;

                if !angle_within_bounds(angle, vision_bound_lower, vision_bound_upper) {
                    return true;
                }

                cum_dir += to_pher.normalize() * pheromone.intensity;

                true
            } else {
                // Entity was removed, delete here
                // Yes this is janky, but its better than making the decay code slower
                // ok maybe it isn't, TODO test this idea
                false
            }
        });
    }

    cum_dir = cum_dir.normalize();

    // attractive force from food/home

    match ant.state {
        AntState::Wandering => {
            for food in food.iter() {
                let to_food = (food.translation - ant_trans.translation).xy();

                if to_food.length() < FOOD_HINT_THRESHOLD {
                    cum_dir += to_food.normalize() * HINT_FACTOR;
                }
            }
        }
        AntState::HasFood => {
            let to_home = world_map_center() - ant_trans.translation.xy();
            cum_dir += to_home.normalize() * HINT_FACTOR;
        }
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
const MOMENTUM_WEIGHT: f32 = 1.0;

pub fn update_ant_movement(
    mut ants: Query<
        (&mut Transform, &mut Ant, &mut EntropyComponent<ChaCha8Rng>),
        (Without<Food>, Without<Pheromone>),
    >,
    pheromones: Query<(&Transform, &Pheromone), (Without<Ant>, Without<Food>)>,
    mut phers_tiled: Query<(&mut PheromoneTileGroup, &MapPos)>,
    food: Query<&Transform, (With<Food>, Without<Ant>, Without<Pheromone>)>,
    time: Res<Time>,
    z_level_q: Query<&ZLevel>,
) {
    let world_center = world_map_center();

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
                if (ant_trans.translation.xy() - world_center).length() < DETECTION_RADIUS {
                    ant.state = AntState::Wandering;
                    ant.secret_desire *= -1.0;
                    ant_trans.rotation *= Quat::from_euler(EulerRot::ZXY, PI, 0.0, 0.0);
                }
            }
        }

        let chosen_dir = Vec3::from((
            ant_desired_direction(
                &mut ant,
                &ant_trans,
                &mut rng,
                &food,
                &mut phers_tiled,
                &pheromones,
            ),
            0.0,
        ));

        ant.secret_desire = chosen_dir.xy();

        let momentum_dir = ant_trans.forward().normalize();

        let mut actual_offset = (chosen_dir + momentum_dir * MOMENTUM_WEIGHT).normalize()
            * ant.speed
            * time.delta_seconds();

        let potential_position = ant_trans.translation + actual_offset;
        let map_pos_potential = world_pos_to_two_d_index(potential_position.xy());
        let map_pos_current = world_pos_to_two_d_index(ant_trans.translation.xy());
        if let Some(z_level) = z_level_q.iter().find(|z_level| z_level.z_level == 0) {
            let is_potential_tile_walkable = z_level.is_tile_walkable(map_pos_potential);
            let world_map_size = world_map_size();
            if potential_position.x > world_map_size.x
                || potential_position.x < 0.
                || (!is_potential_tile_walkable && map_pos_potential.x != map_pos_current.x)
            {
                actual_offset.x *= -1.0;
                ant.secret_desire.x *= -1.0;
            }
            if potential_position.y > world_map_size.y
                || potential_position.y < 0.
                || (!is_potential_tile_walkable && map_pos_potential.y != map_pos_current.y)
            {
                actual_offset.y *= -1.0;
                ant.secret_desire.y *= -1.0;
            }
        }

        // Do these *before* moving
        ant_trans.rotation = ant_trans.looking_at(potential_position, Vec3::Z).rotation;
        ant_trans.translation += actual_offset;

        // TODO change ant state based on findings
    }
}

pub const ANT_POOP_INTERVAL: f32 = 5.0;
pub const PHER_PROX_DISTANCE: f32 = 5.0;
pub const PHER_NUDGE_DAMPING: f32 = 0.3;

pub fn setup_pher_tiles(mut commands: Commands) {
    for i in 0..MAP_SIZE.x {
        for j in 0..MAP_SIZE.y {
            commands.spawn((MapPos(UVec2::new(i, j)), PheromoneTileGroup(vec![])));
        }
    }
}

pub fn spawn_pheromones(
    mut commands: Commands,
    mut ants: Query<(&Transform, &mut Ant), Without<PheromoneTileGroup>>,
    mut phers: Query<(&mut Transform, &mut Pheromone), Without<Ant>>,
    mut phers_tiled: Query<(&mut PheromoneTileGroup, &MapPos)>,
    time: Res<Time>,
) {
    for (ant_trans, mut ant) in ants.iter_mut() {
        if ant.time_until_poop > 0.0 {
            ant.time_until_poop -= time.delta_seconds() * ant.speed;
            continue;
        }

        let ant_tile_pos = world_pos_to_two_d_index(ant_trans.translation.xy());

        for (mut local_phers, tile_pos) in phers_tiled.iter_mut() {
            if ant_tile_pos != tile_pos.0 {
                continue;
            }

            let result = local_phers
                .0
                .iter()
                .map(|pher_ent| (phers.get(*pher_ent), pher_ent))
                .filter_map(|(res, pher_ent)| res.map(|x| (x, pher_ent)).ok())
                .filter(|((trans, pher), _)| {
                    pher.kind == ant.state.pher_to_drop()
                        && (ant_trans.translation - trans.translation).length()
                            <= PHER_PROX_DISTANCE
                })
                .reduce(
                    |((t_closest, p_closest), e_closest), ((t_current, p_current), e_current)| {
                        if (ant_trans.translation - t_closest.translation).length()
                            > (ant_trans.translation - t_closest.translation).length()
                        {
                            ((t_current, p_current), e_current)
                        } else {
                            ((t_closest, p_closest), e_closest)
                        }
                    },
                );

            let mut found_to_edit = false;

            if let Some((_, closest_ent)) = result {
                if let Ok((mut pher_trans, mut pher)) = phers.get_mut(*closest_ent) {
                    let offset = ant_trans.translation - pher_trans.translation;
                    pher_trans.translation += (1.0 - (pher.intensity / (pher.intensity + 1.0)))
                        * offset
                        * PHER_NUDGE_DAMPING;

                    pher.intensity += 1.0;
                    pher.death_timer = 1.0;

                    found_to_edit = true;
                }
            }

            if !found_to_edit {
                let pher_ent = commands
                    .spawn(PheromoneBundle {
                        pheromone: Pheromone {
                            kind: ant.state.pher_to_drop(),
                            intensity: 1.0,
                            death_timer: 1.0,
                        },
                        transform: ant_trans.clone(),
                    })
                    .id();
                local_phers.0.push(pher_ent);
            }

            ant.time_until_poop = ANT_POOP_INTERVAL;
        }
    }
}

pub fn debug_ants(ants: Query<(&Ant, &Transform)>, mut gizmos: Gizmos) {
    for (ant, ant_trans) in ants.iter() {
        let facing = ant.secret_desire;

        let start = ant_trans.translation.xy();
        let end = ant_trans.translation.xy() + facing.normalize() * ant.vision_range;

        gizmos.line_2d(start.xy(), end.xy(), Color::WHITE);
        gizmos
            .circle_2d(ant_trans.translation.xy(), ant.vision_range, Color::WHITE)
            .segments(16);
    }
}

pub fn debug_ants_minimal(ants: Query<(&Ant, &Transform)>, mut gizmos: Gizmos) {
    for (ant, ant_trans) in ants.iter() {
        let facing = ant.secret_desire.normalize() * 2.0;

        let start = ant_trans.translation.xy() - facing;
        let end = ant_trans.translation.xy() + facing;

        gizmos.line_2d(start.xy(), end.xy(), Color::BLACK);
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
