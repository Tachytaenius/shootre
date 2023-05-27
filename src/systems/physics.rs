use crate::components::*;
use crate::util::*;
use crate::systems::startup::{TILEMAP_OFFSET, TILE_SIZE};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub fn collision(
    mut collider_query: Query<(&Collider, &mut Position, &mut Velocity, Option<&Mass>, Option<&Restitution>, Option<&Children>, Option<(&mut Hits, &HitForceThreshold)>)>,
    child_mass_query: Query<&Mass>,
    wall_tilemap_query: Query<(&TilemapSize, &TileStorage), With<WallTilemap>>,
    time: Res<Time>
) {
    // Entity-level collisions
    let (tilemap_size, tile_storage) = wall_tilemap_query.get_single().unwrap();
    for (
        entity_collider,
        mut entity_position,
        mut entity_velocity,
        _, // entity_mass_option,
        entity_restitution_option,
        _, // entity_children_option,
        _ // mut entity_hit_related_option
    ) in collider_query.iter_mut() {
        // Non-solid colliders still can't pass through walls, so no guard clause here

        // Get mass and restitution
        // Mass is unused with hits being scrapped to a "TODO"
        // let mut entity_mass;
        // if let Some(entity_mass_component) = entity_mass_option {
        //     entity_mass = entity_mass_component.value;
        //     if let Some(children) = entity_children_option {
        //         for child_entity in children.iter() {
        //             if let Ok(child_mass) = child_mass_query.get(*child_entity) {
        //                 entity_mass += child_mass.value;
        //             }
        //         }
        //     }
        // } else {
        //     entity_mass = 0.0;
        // }
        let entity_restitution = if let Some(entity_restitution_component) = entity_restitution_option {
            entity_restitution_component.value
        } else {
            DEFAULT_RESTITUTION
        };

        // Do x axis
        entity_position.value.x += entity_velocity.value.x * time.delta_seconds(); // Apply x velocity
        // Ignoring the -2 and +2, this "nearby tiles only" calculation probably changes things when the circle's circumference lies exactly on a tile edge
        // Widening the x by 2 tiles on either side should be enough for any issues with tiles moving the entity out of the zone of checked tiles
        let lower_x = (((entity_position.value.x - TILEMAP_OFFSET.x + TILE_SIZE / 2.0 - entity_collider.radius) / TILE_SIZE).floor() - 2.0).max(0.0).min((tilemap_size.x - 1) as f32) as u32;
        let upper_x = (((entity_position.value.x - TILEMAP_OFFSET.x + TILE_SIZE / 2.0 + entity_collider.radius) / TILE_SIZE).floor() + 2.0).max(0.0).min((tilemap_size.x - 1) as f32) as u32;
        let lower_y = (((entity_position.value.y - TILEMAP_OFFSET.y + TILE_SIZE / 2.0 - entity_collider.radius) / TILE_SIZE).floor()      ).max(0.0).min((tilemap_size.y - 1) as f32) as u32;
        let upper_y = (((entity_position.value.y - TILEMAP_OFFSET.y + TILE_SIZE / 2.0 + entity_collider.radius) / TILE_SIZE).floor()      ).max(0.0).min((tilemap_size.y - 1) as f32) as u32;
        for x in lower_x..=upper_x {
            for y in lower_y..=upper_y {
                // Guard clauses
                if tile_storage.get(&TilePos {x: x, y: y}).is_none() {
                    continue;
                }
                if !collision_detection::circle_aabb(
                    entity_collider.radius,
                    entity_position.value,
                    TILE_SIZE,
                    TILE_SIZE,
                    Vec2::new(x as f32, y as f32) * TILE_SIZE - Vec2::splat(TILE_SIZE / 2.0) + TILEMAP_OFFSET
                ) {
                    continue;
                }

                let ( // Tilemap velocity/position changes are always zero
                    (entity_velocity_change, _),
                    (entity_position_change, _)
                ) = collision_resolution::circle_aabb(
                    entity_collider.radius,
                    entity_position.value,
                    entity_velocity.value,
                    0.0, // Against a mass of 1, meaning the tilemap has "infinite mass"
                    entity_restitution,

                    TILE_SIZE,
                    TILE_SIZE,
                    Vec2::new(x as f32, y as f32) * TILE_SIZE - Vec2::splat(TILE_SIZE / 2.0) + TILEMAP_OFFSET,
                    Vec2::ZERO,
                    1.0,
                    DEFAULT_RESTITUTION // TODO: Tilemap
                );

                entity_velocity.value.x += entity_velocity_change.x;
                entity_position.value.x += entity_position_change.x;
            }
        }

        // Do y axis
        entity_position.value.y += entity_velocity.value.y * time.delta_seconds(); // Apply y velocity
        let lower_x = (((entity_position.value.x - TILEMAP_OFFSET.x + TILE_SIZE / 2.0 - entity_collider.radius) / TILE_SIZE).floor()      ).max(0.0).min((tilemap_size.x - 1) as f32) as u32;
        let upper_x = (((entity_position.value.x - TILEMAP_OFFSET.x + TILE_SIZE / 2.0 + entity_collider.radius) / TILE_SIZE).floor()      ).max(0.0).min((tilemap_size.x - 1) as f32) as u32;
        let lower_y = (((entity_position.value.y - TILEMAP_OFFSET.y + TILE_SIZE / 2.0 - entity_collider.radius) / TILE_SIZE).floor() - 2.0).max(0.0).min((tilemap_size.y - 1) as f32) as u32;
        let upper_y = (((entity_position.value.y - TILEMAP_OFFSET.y + TILE_SIZE / 2.0 + entity_collider.radius) / TILE_SIZE).floor() + 2.0).max(0.0).min((tilemap_size.y - 1) as f32) as u32;
        for x in lower_x..=upper_x {
            for y in lower_y..=upper_y {
                // Guard clauses
                if tile_storage.get(&TilePos {x: x, y: y}).is_none() {
                    continue;
                }
                if !collision_detection::circle_aabb(
                    entity_collider.radius,
                    entity_position.value,
                    TILE_SIZE,
                    TILE_SIZE,
                    Vec2::new(x as f32, y as f32) * TILE_SIZE - Vec2::splat(TILE_SIZE / 2.0) + TILEMAP_OFFSET
                ) {
                    continue;
                }

                let ( // Tilemap velocity/position changes are always zero
                    (entity_velocity_change, _),
                    (entity_position_change, _)
                ) = collision_resolution::circle_aabb(
                    entity_collider.radius,
                    entity_position.value,
                    entity_velocity.value,
                    0.0, // Against a mass of 1, meaning the tilemap has "infinite mass"
                    entity_restitution,

                    TILE_SIZE,
                    TILE_SIZE,
                    Vec2::new(x as f32, y as f32) * TILE_SIZE - Vec2::splat(TILE_SIZE / 2.0) + TILEMAP_OFFSET,
                    Vec2::ZERO,
                    1.0,
                    DEFAULT_RESTITUTION // TODO: Tilemap
                );

                entity_velocity.value.y += entity_velocity_change.y;
                entity_position.value.y += entity_position_change.y;
            }
        }

        // Scrapping entity-level hits to a "TODO", if you're moving fast enough for that you're going to clip through the walls
        // if let Some((ref mut hits, hit_force_threshold)) = entity_hit_related_option {
        //     let force = entity_velocity_change * entity_mass;
        //     if force.length() >= hit_force_threshold.value {
        //         hits.value.push(Hit {
        //             entry_point: Vec2::ZERO, // TODO
        //             force: force,
        //             damage: 0.0, // TODO
        //             apply_force: false,
        //             blood_loss: 0.0
        //         });
        //     }
        // }
    }

    // Entity-entity collisions
    let mut combinations = collider_query.iter_combinations_mut();
    while let Some([
        (
            a_collider,
            mut a_position,
            mut a_velocity,
            a_mass_option,
            a_restitution_option,
            a_children_option,
            a_hit_related_option
        ), (
            b_collider,
            mut b_position,
            mut b_velocity,
            b_mass_option,
            b_restitution_option,
            b_children_option,
            b_hit_related_option
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

            let ((a_acceleration, b_acceleration), (a_shift, b_shift)) = collision_resolution::circle_circle(
                a_position.value, a_collider.radius, a_velocity.value, a_mass, a_restitution,
                b_position.value, b_collider.radius, b_velocity.value, b_mass, b_restitution
            );

            a_velocity.value += a_acceleration;
            if let Some((mut a_hits, a_hit_force_threshold)) = a_hit_related_option {
                let a_force = a_acceleration * a_mass;
                if a_force.length() >= a_hit_force_threshold.value {
                    a_hits.value.push(Hit {
                        entry_point: Vec2::ZERO, // TODO
                        force: a_force,
                        damage: 0.0, // TODO
                        apply_force: false,
                        blood_loss: 0.0
                    });
                }
            }
            b_velocity.value += b_acceleration;
            if let Some((mut b_hits, b_hit_force_threshold)) = b_hit_related_option {
                let b_force = b_acceleration * b_mass;
                if b_force.length() >= b_hit_force_threshold.value {
                    b_hits.value.push(Hit {
                        entry_point: Vec2::ZERO, // TODO
                        force: b_force,
                        damage: 0.0, // TODO
                        apply_force: false,
                        blood_loss: 0.0
                    });
                }
            }

            a_position.value += a_shift;
            b_position.value += b_shift;
        }
    }
}

pub fn apply_velocity(
    mut query: Query<(&mut Position, &Velocity), Without<Collider>>, // Without collider because velocity is done in collision for colliders
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
    mut query: Query<(&mut Grounded, Option<&Alive>)>,
    time: Res<Time>
) {
    for (mut grounded, alive_option) in query.iter_mut() {
        if let Some(old_timer_state) = grounded.floored_recovery_timer {
            if alive_option.is_some() {
                let new_timer_state = (old_timer_state - time.delta_seconds()).max(0.0);
                if new_timer_state > 0.0 {
                    grounded.floored_recovery_timer = Some(new_timer_state);
                } else {
                    grounded.floored_recovery_timer = None;
                    grounded.standing = true;
                }
            } else {
                // This is also done in damage::dying
                grounded.standing = false;
                grounded.floored_recovery_timer = None;
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
