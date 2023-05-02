use crate::components::*;
use crate::util::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

fn progress_time_with_cooldown_interrupt(current: &mut f32, target: f32, cooldown: &mut f32) {
    // Move current up towards target but "stop" if cooldown ticks down towards 0 before then
    debug_assert!(*current < target); // Not <= because we shouldn't be progressing time if we've already reached the target
    let delta = (target - *current).min(*cooldown);
    *current += delta;
    *cooldown -= delta;
}

pub fn guns(
    mut commands: Commands,
    mut gun_query: Query<(
        &mut Gun,
        Option<&Parent>,
        Option<&HoldingInfo>,
        Option<&Position>,
        Option<&Velocity>,
        Option<&Angle>,
        Option<&AngularVelocity>
    )>,
    holder_query: Query<(
        Option<&Will>,
        &Position,
        Option<&Velocity>,
        Option<&Angle>,
        Option<&AngularVelocity>
    ), With<Children>>,
    time: Res<Time>
) {
    for (
        mut gun,
        parent_option,
        holding_info_option,
        position_option,
        velocity_option,
        angle_option,
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
        let position;
        let velocity;
        let angle;
        let angular_velocity;
        // Position is expected, since there is no reasonable default. Panic if not present
        if let Some(parent) = parent_option {
            let parent_result = holder_query.get(parent.get());
            if let Ok((
                _,
                parent_position,
                parent_velocity_option,
                parent_angle_option,
                parent_angular_velocity_option
            )) = parent_result {
                let holding_info = holding_info_option.unwrap();
                let held_distance = holding_info.held_distance;
                let held_angle = holding_info.held_angle;
                
                let parent_position = parent_position.value;
                let parent_angle;
                if let Some(parent_angle_component) = parent_angle_option {
                    parent_angle = parent_angle_component.value;
                } else {
                    parent_angle = 0.0;
                }
                position = parent_position + Vec2::from_angle(parent_angle).rotate(Vec2::new(held_distance, 0.0));
                angle = parent_angle + held_angle;
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
                panic!(); // Parent does not have position
            }
        } else {
            position = position_option.unwrap().value; // Position expected to be on the gun itself if there's no parent
            if let Some(angle_component) = angle_option {
                angle = angle_component.value;
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

                    commands.spawn((
                        Position {value: projectile_position},
                        PreviousPosition {value: projectile_origin},
                        Velocity {value: projectile_velocity},
                        ShapeBundle {..default()},
                        Stroke::new(gun.projectile_colour, 1.0), // Gets immediately overwritten by a version with calculated alpha by rebuild_traced_shape
                        ProjectileColour {value: gun.projectile_colour},
                        Flying,
                        FlyingRecoveryRate {value: flying_recovery_rate},
                        TracedLine,
                        SpawnedMidTick {when: current_time / target_time},
                        DisplayLayer::Projectiles
                    ));
                }
            } else {
                // If we're not shooting (or gun.cooldown_timer failed to reach 0 before current_time reached target_time)
                break;
            }
        }
    }
}
