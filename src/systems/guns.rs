use crate::components::*;
use crate::util::*;
use crate::util::collision_detection;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

fn progress_time_with_cooldown_interrupt(current: &mut f32, target: f32, cooldown: &mut f32) {
    // Move current up towards target but "stop" if cooldown ticks down towards 0 before then
    debug_assert!(*current < target); // Not <= because we shouldn't be progressing time if we've already reached the target
    let delta = (target - *current).min(*cooldown);
    *current += delta;
    *cooldown -= delta;
}

pub fn tick_guns(
    mut commands: Commands,
    // We use previous position/angle here because it fixes a bug where moving while shooting causes bullets to appear in front of you to the side.
    // This is due to ordering in main.rs. Hopefully I won't come along and break this later. This could be better fixed with a more comprehensive
    // physics engine model, unless this is the right solution and I don't have the understanding to verify it as such.
    mut gun_query: Query<(
        &mut Gun,
        Option<&Parent>,
        Option<&HoldingInfo>,
        Option<&PreviousPosition>,
        Option<&Velocity>,
        Option<&PreviousAngle>,
        Option<&AngularVelocity>
    )>,
    holder_query: Query<(
        Option<&Will>,
        &PreviousPosition,
        Option<&Velocity>,
        Option<&PreviousAngle>,
        Option<&AngularVelocity>
    ), With<Children>>,
    time: Res<Time>
) {
    for (
        mut gun,
        parent_option,
        holding_info_option,
        previous_position_option,
        velocity_option,
        previous_angle_option,
        angular_velocity_option
    ) in gun_query.iter_mut() {
        // If no willed parent, trigger is not depressd, else trigger is depressed depending on will
        gun.trigger_depressed = false;
        if let Some(parent) = parent_option {
            let parent_result = holder_query.get(parent.get());
            if let Ok((will_option, _, _, _, _)) = parent_result {
                if let Some(will) = will_option {
                    gun.trigger_depressed = will.depress_trigger;
                }
            }
        }

        // Get spatial information from self or parent
        let position; // Based on previous position
        let velocity;
        let angle; // Based on previous angle
        let angular_velocity;
        // PreviousPosition is expected, since there is no reasonable default. Panic if not present
        if let Some(parent) = parent_option {
            let parent_result = holder_query.get(parent.get());
            if let Ok((
                _,
                parent_previous_position,
                parent_velocity_option,
                parent_previous_angle_option,
                parent_angular_velocity_option
            )) = parent_result {
                let holding_info = holding_info_option.unwrap();
                let held_distance = holding_info.held_distance;
                let held_angle = holding_info.held_angle;
                
                let parent_previous_position = parent_previous_position.value;
                let parent_previous_angle;
                if let Some(parent_previous_angle_component) = parent_previous_angle_option {
                    parent_previous_angle = parent_previous_angle_component.value;
                } else {
                    parent_previous_angle = 0.0;
                }
                position = parent_previous_position + Vec2::from_angle(parent_previous_angle).rotate(Vec2::new(held_distance, 0.0));
                angle = parent_previous_angle + held_angle;
                if let Some(parent_velocity_component) = parent_velocity_option {
                    velocity = parent_velocity_component.value;
                } else {
                    velocity = Vec2::ZERO;
                }
                if let Some(parent_angular_velocity_component) = parent_angular_velocity_option {
                    angular_velocity = parent_angular_velocity_component.value;
                } else {
                    angular_velocity = 0.0;
                }
            } else {
                panic!(); // Parent does not have previous position
            }
        } else {
            position = previous_position_option.unwrap().value; // Previous position expected to be on the gun itself if there's no parent
            if let Some(previous_angle_component) = previous_angle_option {
                angle = previous_angle_component.value;
            } else {
                angle = 0.0;
            }
            if let Some(velocity_component) = velocity_option {
                velocity = velocity_component.value;
            } else {
                velocity = Vec2::ZERO;
            }
            if let Some(angular_velocity_component) = angular_velocity_option {
                angular_velocity = angular_velocity_component.value;
            } else {
                angular_velocity = 0.0;
            }
        }

        let mut shoot = if gun.auto {
            gun.trigger_depressed
        } else {
            gun.trigger_depressed && !gun.trigger_depressed_previous_frame
        };
        let mut rng = rand::thread_rng();
        // The key point here is that for rapid-fire guns, gun.cooldown (and
        // by extension gun.cooldown_timer) may fit in target_time multiple times
        let mut current_time = 0.0;
        let target_time = time.delta_seconds();
        while current_time < target_time {
            progress_time_with_cooldown_interrupt(&mut current_time, target_time, &mut gun.cooldown_timer);
            if shoot && gun.cooldown_timer == 0.0 {
                gun.cooldown_timer = gun.cooldown;
                if !gun.auto { // Only once
                    shoot = false;
                }

                let gun_position = position + velocity * current_time;
                let gun_angle = angle + angular_velocity * current_time;
                let aim_direction = Vec2::from_angle(gun_angle);
                let projectile_origin = gun_position + aim_direction * gun.muzzle_distance;

                for _ in 0..gun.projectile_count {
                    // target_time - current_time is used a couple of times because the earlier the projectile was fired, the longer it has had for its properties to advance
                    let mut projectile_velocity = velocity + aim_direction * gun.projectile_speed +
                        Vec2::from_angle(gun_angle).rotate(random_in_shape::circle(&mut rng, 1.0) * gun.projectile_spread * gun.projectile_speed); // In here because of projectile-specific use of random
                    let projectile_position = projectile_origin + projectile_velocity * (target_time - current_time); // TODO: collision detection for the distance travelled

                    // Simulate a bit of speed reduction
                    let old_speed = projectile_velocity.length();
                    let flying_recovery_rate = gun.projectile_flying_recovery_rate;
                    let new_speed = (old_speed - flying_recovery_rate * (target_time - current_time)).max(0.0);
                    if old_speed > 0.0 && new_speed != old_speed {
                        projectile_velocity = projectile_velocity.normalize() * new_speed;
                    }

                    // No need to simulate collision detection here as it is done immediately using PreviousPosition

                    commands.spawn((
                        Position {value: projectile_position},
                        PreviousPosition {value: projectile_origin},
                        Velocity {value: projectile_velocity},
                        Mass {value: gun.projectile_mass},
                        ShapeBundle {..default()},
                        Stroke::new(gun.projectile_colour, 1.0), // Gets immediately overwritten by a version with calculated alpha by rebuild_traced_shape
                        ProjectileColour {value: gun.projectile_colour},
                        Flying,
                        FlyingRecoveryRate {value: flying_recovery_rate},
                        TracedLine,
                        GunProjectile,
                        SpawnedMidTick {when: current_time / target_time},
                        DisplayLayer {
                            index: DisplayLayerIndex::Projectiles,
                            flying: false
                        }
                    ));
                }
            } else {
                // If we're not shooting (or gun.cooldown_timer failed to reach 0 before current_time reached target_time)
                break;
            }
        }
    }
}

const PROJECTILE_BLOOD_LOSS_MULTIPLIER: f32 = 0.5;

pub fn detect_hits( // TODO: Tilemap hits
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Position, &PreviousPosition, &Velocity, &Mass), (With<GunProjectile>, Without<DestroyedButRender>)>,
    mut target_query: Query<(&Position, &Collider, &mut Hits), Without<GunProjectile>>
) {
    // Starts from previous position and goes to current position
    for (
        projectile_entity,
        mut projectile_position,
        projectile_previous_position,
        projectile_velocity,
        projectile_mass
    ) in projectile_query.iter_mut() {
        for (
            target_position,
            target_collider,
            mut target_hits
        ) in target_query.iter_mut() {
            if !target_collider.solid {
                continue;
            }

            let intersections_option = collision_detection::line_circle_intersection(
                projectile_previous_position.value,
                projectile_position.value,
                target_collider.radius,
                target_position.value
            );
            if intersections_option.is_none() {
                continue;
            }
            let (intersection_in, _) = intersections_option.unwrap();

            let entry_wound = if collision_detection::circle_point(target_collider.radius, target_position.value, projectile_previous_position.value) {
                Some(projectile_previous_position.value)
            } else if 0.0 <= intersection_in && intersection_in <= 1.0 {
                Some(projectile_previous_position.value.lerp(projectile_position.value, intersection_in))
            } else {
                None
            };
            // entry_wound is an option because I was working with an exit_wound option too but removed it. This works fine, so
            if entry_wound.is_some() {
                projectile_position.value = entry_wound.unwrap();
                commands.entity(projectile_entity).insert(DestroyedButRender);
                target_hits.value.push(Hit {
                    entry_point: entry_wound.unwrap(),
                    force: projectile_velocity.value * projectile_mass.value, // Could take code from circle-circle collision resolution for this in a future project if it's more correct
                    damage: 0.0,
                    apply_force: true,
                    blood_loss: projectile_velocity.value.length() * projectile_mass.value * PROJECTILE_BLOOD_LOSS_MULTIPLIER
                });
                break;
            }
        }
    }
}

pub fn despawn_stationary_projectiles(
    mut commands: Commands,
    query: Query<(Entity, &Velocity), With<GunProjectile>>
) {
    for (entity, velocity) in query.iter() {
        if velocity.value.length() == 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
