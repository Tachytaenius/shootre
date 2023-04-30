use crate::components::*;
use crate::util::*;
use bevy::prelude::*;

pub fn collision(
    mut collider_query: Query<(&Collider, &Position, &mut Velocity, Option<&Mass>, Option<&Restitution>, Option<&Children>)>,
    child_mass_query: Query<&Mass>
) {
    let mut combinations = collider_query.iter_combinations_mut();
    while let Some([
        (
            a_collider,
            a_position,
            mut a_velocity,
            a_mass_option,
            a_restitution_option,
            a_children_option
        ), (
            b_collider,
            b_position,
            mut b_velocity,
            b_mass_option,
            b_restitution_option,
            b_children_option
        )
    ]) = combinations.fetch_next() {
        if !(a_collider.solid && b_collider.solid) {
            continue;
        }
        if collision_detection::circle_circle(a_collider.radius, a_position.value, b_collider.radius, b_position.value) {
            let mut a_mass;
            if let Some(a_mass_component) = a_mass_option {
                a_mass = a_mass_component.value;
                if let Some(children) = a_children_option {
                    for child_entity in children.iter() {
                        if let Ok(child_mass) = child_mass_query.get(*child_entity) {
                            a_mass += child_mass.value;
                        }
                    }
                }
            } else {
                a_mass = 0.0;
            }
            if a_mass == 0.0 {
                continue;
            }
            let mut b_mass;
            if let Some(b_mass_component) = b_mass_option {
                b_mass = b_mass_component.value;
                if let Some(children) = b_children_option {
                    for child_entity in children.iter() {
                        if let Ok(child_mass) = child_mass_query.get(*child_entity) {
                            b_mass += child_mass.value;
                        }
                    }
                }
            } else {
                b_mass = 0.0;
            }
            if b_mass == 0.0 {
                continue;
            }

            let a_restitution;
            if let Some(a_restitution_component) = a_restitution_option {
                a_restitution = a_restitution_component.value;
            } else {
                a_restitution = DEFAULT_RESTITUTION;
            }
            let b_restitution;
            if let Some(b_restitution_component) = b_restitution_option {
                b_restitution = b_restitution_component.value;
            } else {
                b_restitution = DEFAULT_RESTITUTION;
            }

            (a_velocity.value, b_velocity.value) = collision_resolution::circle_circle(
                 a_position.value, a_velocity.value, a_mass, a_restitution,
                b_position.value, b_velocity.value, b_mass, b_restitution
            );
        }
    }
}

pub fn apply_velocity(
    mut query: Query<(&mut Position, &Velocity)>,
    time: Res<Time>
) {
    for (mut position, velocity) in query.iter_mut() {
        position.value += velocity.value * time.delta_seconds();
    }
}

pub fn apply_angular_velocity(
    mut query: Query<(&mut Angle, &AngularVelocity)>,
    time: Res<Time>
) {
    for (mut angle, angular_velocity) in query.iter_mut() {
        angle.value += angular_velocity.value * time.delta_seconds();
    }
}

pub fn _monitor_conservation(query: Query<(&Velocity, &Mass)>) {
    let mut energy = 0.0;
    let mut momentum = Vec2::ZERO;
    for (velocity, mass) in query.iter() {
        energy += mass.value / 2.0 * velocity.value.length_squared();
        momentum += velocity.value * mass.value;
    }
    println!("Energy: {}, Momentum: {}", energy, momentum);
}

#[cfg(debug_assertions)]
pub fn check_consistent_grounded_flying_state(
    query: Query<(
        Option<&Grounded>,
        Option<&Flying>
    )>
) {
    for (grounded_option, flying_option) in query.iter() {
        assert!(!(grounded_option.is_some() && flying_option.is_some()));
        if let Some(grounded) = grounded_option {
            if grounded.standing {
                assert!(grounded.floored_recovery_timer.is_none());
            }
        }
    }
}

pub fn manage_flyers(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Velocity,
        Option<&FlyingRecoveryRate>,
        Option<&Levitates>,
        Option<&Gait>,
        Option<&RegroundThreshold>
    ), With<Flying>>,
    time: Res<Time>
) {
    for (
        entity,
        mut velocity,
        flying_recovery_rate_option,
        levitates_option,
        gait_option,
        reground_threshold_option
    ) in query.iter_mut() {
        let old_speed = velocity.value.length();
        let speed_reduction;
        if let Some(flying_recovery_rate) = flying_recovery_rate_option {
            speed_reduction = flying_recovery_rate.value;
        } else {
            speed_reduction = DEFAULT_FLYING_RECOVERY_RATE;
        }
        let new_speed = (old_speed - speed_reduction * time.delta_seconds()).max(0.0);

        if old_speed > 0.0 && new_speed != old_speed {
            velocity.value = velocity.value.normalize() * new_speed;
        }

        if let None = levitates_option {
            // Manage stopping flying
            let stop_flying_threshold;
            if let Some(reground_threshold) = reground_threshold_option {
                stop_flying_threshold = reground_threshold.value;
            } else {
                stop_flying_threshold = DEFAULT_REGROUND_THRESHOLD;
            }
            if new_speed <= stop_flying_threshold {
                let floored_recovery_time;
                if let Some(gait) = gait_option {
                    floored_recovery_time = Some(gait.floored_recovery_time);
                } else {
                    floored_recovery_time = None;
                }
                commands.entity(entity).remove::<Flying>();
                commands.entity(entity).insert(Grounded {
                    standing: false, // If ordering is as intended, a floored recovery time(r) of 0 should cause the entity to stand immediately
                    floored_recovery_timer: floored_recovery_time
                });
            }
        }
    }
}

pub fn manage_flooreds(
    mut query: Query<&mut Grounded>,
    time: Res<Time>
) {
    for mut grounded in query.iter_mut() {
        if let Some(old_timer_state) = grounded.floored_recovery_timer {
            let new_timer_state = (old_timer_state - time.delta_seconds()).max(0.0);
            if new_timer_state > 0.0 {
                grounded.floored_recovery_timer = Some(new_timer_state);
            } else {
                grounded.floored_recovery_timer = None;
                grounded.standing = true;
            }
        }
    }
}

pub fn angular_friction(
    mut query: Query<(&mut AngularVelocity, Option<&FlooredAngularFriction>, Option<&UnflooredAngularFriction>, Option<&Grounded>)>,
    time: Res<Time>
) {
    for (mut angular_velocity, floored_angular_friction_option, unfloored_angular_friction_option, grounded_option) in query.iter_mut() {
        let floored;
        if let Some(grounded) = grounded_option {
            floored = !grounded.standing;
        } else {
            floored = true;
        }

        let friction;
        if floored {
            if let Some(floored_angular_friction) = floored_angular_friction_option {
                friction = floored_angular_friction.value;
            } else {
                friction = DEFAULT_FLOORED_ANGULAR_FRICTION;
            }
        } else {
            if let Some(unfloored_angular_friction) = unfloored_angular_friction_option {
                friction = unfloored_angular_friction.value;
            } else {
                friction = DEFAULT_UNFLOORED_ANGULAR_FRICTION;
            }
        }

        angular_velocity.value = angular_velocity.value.signum() * (angular_velocity.value.abs() - friction * time.delta_seconds()).max(0.0); // Doesn't need to be proper_signum
    }
}

pub fn floor_friction(
    mut query: Query<(&Grounded, Option<&FloorFriction>, &mut Velocity)>,
    time: Res<Time>
) {
    for (grounded, floor_friction_option, mut velocity) in query.iter_mut() {
        if !grounded.standing {
            let friction;
            if let Some(floor_friction) = floor_friction_option {
                friction = floor_friction.value;
            } else {
                friction = DEFAULT_FLOOR_FRICTION;
            }
            let old_speed = velocity.value.length();
            let new_speed = (old_speed - friction * time.delta_seconds()).max(0.0);
            if old_speed > 0.0 && new_speed != old_speed {
                velocity.value = velocity.value.normalize() * new_speed;
            }
        }
    }
}

pub fn tripping(
    mut commands: Commands,
    query: Query<(Entity, Option<&TripThreshold>, &Velocity), With<Grounded>>
) {
    for (entity, trip_threshold_option, velocity) in query.iter() {
        let trip_threshold;
        if let Some(trip_threshold_component) = trip_threshold_option {
            trip_threshold = trip_threshold_component.value;
        } else {
            trip_threshold = DEFAULT_TRIP_THRESHOLD;
        }
        if velocity.value.length() > trip_threshold {
            commands.entity(entity).remove::<Grounded>();
            commands.entity(entity).insert(Flying);
        }
    }
}
