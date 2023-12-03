use crate::*;
use bevy::math::*;
use std::f32::consts::TAU;

pub enum PheromoneKind {
    HomeThisWay,
    FoodThisWay,
}

pub enum AntState {
    Wandering,
    FoundFood,
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

pub fn update_ant(
    mut ants: Query<(&mut Transform, &mut Ant)>,
    pheromones: Query<(&Transform, &Pheromone)>,
) {
    for (mut ant_trans, mut ant) in ants.iter_mut() {
        let (ant_dir, _, _) = ant_trans.rotation.to_euler(EulerRot::ZXY);

        let vision_bound_lower = ant_dir + ant.vision_arc / 2.0;
        let vision_bound_upper = ant_dir - ant.vision_arc / 2.0;

        let mut cum_dir = Vec2::ZERO;
        for (pher_trans, pheromone) in pheromones.iter() {
            let to_pher = (pher_trans.translation - ant_trans.translation).xy();

            if to_pher.length() <= ant.vision_range {
                let angle = f32::atan2(to_pher.y, to_pher.x);

                if angle_within_bounds(angle, vision_bound_lower, vision_bound_upper) {
                    cum_dir += to_pher.normalize() * pheromone.intensity;
                }
            }
        }

        // Keep along the same path
        if cum_dir == Vec2::ZERO {
            cum_dir = (ant_trans.rotation * Vec3::X).xy();
        }

        // TODO apply some random wandering to the ant's chosen direction
        let chosen_dir = cum_dir.normalize();

        ant_trans.translation = Vec3::new(chosen_dir.x, chosen_dir.y, ant_trans.translation.z)

        // TODO change ant state based on findings
        // TODO make ants drop pheromones
    }
}
