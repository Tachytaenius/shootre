mod components;

use bevy::prelude::*;
use std::f32::consts::TAU;
use rand::prelude::*;
use bevy_prototype_lyon::prelude::*;
use components::*;

const DEFAULT_FLYING_RECOVERY_RATE: f32 = 1000.0;

fn proper_signum(x: f32) -> f32 {
    if x > 0.0 {
        return 1.0;
    } else if x < 0.0 {
        return -1.0;
    } else {
        return 0.0;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_dots)
        .add_system(walking)
        .add_system(turning)
        .add_system(apply_velocity)
        .add_system(apply_angular_velocity)
		.add_system(follow_player)
		.add_system(update_transforms)
        .add_system(manage_flyers)
        .add_system(tripping)
        .add_system(hollow_flying)
        .add_system(fill_grounded)
        .add_system(shooting)
        .add_system(gun_cooldown)
        .run();
}

fn spawn_camera (mut commands: Commands) {
    commands.spawn(
        Camera2dBundle {
            ..default()
        }
    );
}

fn spawn_player (
    mut commands: Commands
) {
    let shape = shapes::Circle {
        radius: 10.0,
        ..default()
    };
    let _machine_gun = Gun {
        projectile_speed: 2000.0,
        projectile_radius: 1.0,
        projectile_spread: Vec2::new(10.0, 10.0),
        projectile_count: 1,
        muzzle_distance: 12.5,
        cooldown: 0.01,
        auto: true,

        cooldown_timer: 0.0
    };
    let shotgun = Gun {
        projectile_speed: 1750.0,
        projectile_radius: 0.2,
        projectile_spread: Vec2::new(50.0, 50.0),
        projectile_count: 25,
        muzzle_distance: 11.0,
        cooldown: 1.0,
        auto: false,

        cooldown_timer: 0.0
    };
    commands.spawn((
        Position {
            value: Vec2::ZERO
        },
        Velocity {
            value: Vec2::ZERO
        },
        Gait {
            max_speed: 200.0,
            acceleration: 800.0,
            stand_threshold: 210.0,
            trip_threshold: 220.0
        },
        Angle {
            value: 0.0
        },
        AngularVelocity {
            value: 0.0
        },
        AngularGait {
            max_speed: TAU / 2.0,
            acceleration: TAU * 8.0
        },
        Player,
        Grounded,
        shotgun,
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..default()
        },
        Fill::color(Color::WHITE),
        Stroke::new(Color::WHITE, 1.0)
    ));
}

fn random_vec2_circle(rng: &mut rand::rngs::ThreadRng, radius: f32) -> Vec2 {
    let r = (rng.gen_range(0.0..1.0) as f32).powf(0.5) * radius;
    let theta = rng.gen_range(0.0..TAU);
    return Vec2::new(theta.cos() * r, theta.sin() * r);
}

fn spawn_dots (
    mut commands: Commands
) {
    let shape = shapes::Circle {
        radius: 2.0,
        ..default()
    };
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        commands.spawn((
            Position {
                value: random_vec2_circle(&mut rng, 1000.0) + Vec2::new(300.0, 0.0)
            },
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Fill::color(Color::NONE),
            Stroke::new(Color::WHITE, 1.0)
        ));
    }
}

fn locomotion_handle_axis(current: f32, target: f32, acceleration: f32, delta_seconds: f32) -> f32 {
    if acceleration > 0.0 {
        return target.min(current + acceleration * delta_seconds);
    } else {
        return target.max(current + acceleration * delta_seconds);
    }
}

fn walking (
    mut query: Query<
        (
            &mut Velocity,
            &Gait,
            Option<&Angle>
        ), (
            With<Player>,
            Or<(
                With<Grounded>,
                With<Levitates> // Levitators don't need to be grounded to move
            )>
        )
    >,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>
) {
    if let Ok((mut velocity, gait, angle_option)) = query.get_single_mut() {
        let mut relative_direction = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::A) {
            relative_direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            relative_direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::W) {
            relative_direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            relative_direction.y -= 1.0;
        }
        if relative_direction != Vec2::ZERO {
            relative_direction = relative_direction.normalize();
        }

        let target_relative_velocity = relative_direction * gait.max_speed;
        let entity_angle;
        if let Some(angle) = angle_option {
            entity_angle = angle.value;
        } else {
            entity_angle = 0.0;
        }
        let rotate_angle = entity_angle - TAU / 4.0;
        let mut relative_velocity = Vec2::from_angle(-rotate_angle).rotate(velocity.value);

        let difference = target_relative_velocity - relative_velocity;
        let direction;
        if difference == Vec2::ZERO {
            direction = Vec2::ZERO;
        } else {
            direction = difference.normalize();
        }
        let acceleration_distribution = direction * gait.acceleration; // So that you don't get to use all of acceleration on both axes

        relative_velocity.x = locomotion_handle_axis(relative_velocity.x, target_relative_velocity.x, acceleration_distribution.x, time.delta_seconds());
        relative_velocity.y = locomotion_handle_axis(relative_velocity.y, target_relative_velocity.y, acceleration_distribution.y, time.delta_seconds());

        velocity.value = Vec2::from_angle(rotate_angle).rotate(relative_velocity);
    }
}

fn turning (
    mut query: Query<
        (
            &mut AngularVelocity,
            &AngularGait
        ), (
            With<Player>,
            Or<(
                With<Grounded>,
                With<Levitates>
            )>
        )
    >,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>
) {
    if let Ok((mut angular_velocity, angular_gait)) = query.get_single_mut() {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Comma) {
            direction += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Period) {
            direction -= 1.0;
        }
        
        let target_angular_velocity = direction * angular_gait.max_speed;
        angular_velocity.value = locomotion_handle_axis(
            angular_velocity.value,
            target_angular_velocity,
            angular_gait.acceleration * proper_signum(target_angular_velocity - angular_velocity.value),
            time.delta_seconds()
        );
    }
}

fn apply_velocity (
    mut query: Query<(&mut Position, &Velocity)>,
    time: Res<Time>
) {
    for (mut position, velocity) in query.iter_mut() {
        position.value += velocity.value * time.delta_seconds();
    }
}

fn apply_angular_velocity (
    mut query: Query<(&mut Angle, &AngularVelocity)>,
    time: Res<Time>
) {
    for (mut angle, angular_velocity) in query.iter_mut() {
        angle.value += angular_velocity.value * time.delta_seconds();
    }
}

fn follow_player (
    mut camera_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<(&Position, Option<&Angle>), With<Player>>
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        if let Ok((player_position, player_angle_option)) = player_query.get_single() {
            let entity_angle;
            if let Some(angle) = player_angle_option {
                entity_angle = angle.value;
            } else {
                entity_angle = 0.0;
            }
            camera_transform.rotation = Quat::from_rotation_z(entity_angle - TAU / 4.0);
            let camera_position = player_position.value + Vec2::from_angle(entity_angle) * 250.0; // Project camera position forwards to move player to bottom of screen
            camera_transform.translation = Vec3::new(camera_position.x, camera_position.y, 0.0);
        }
    }
}

fn update_transforms (mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation = Vec3::new(position.value.x, position.value.y, 0.0);
    }
}

fn manage_flyers(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Velocity,
        Option<&FlyingRecoveryRate>,
        Option<&Levitates>,
        Option<&Gait>
    ), With<Flying>>,
    time: Res<Time>
) {
    for (
        entity,
        mut velocity,
        flying_recovery_rate_option,
        levitates_option,
        gait_option
    ) in query.iter_mut() {
        let old_speed = velocity.value.length();
        let speed_reduction;
        if let Some(flying_recovery_rate) = flying_recovery_rate_option {
            speed_reduction = flying_recovery_rate.value;
        } else {
            speed_reduction = DEFAULT_FLYING_RECOVERY_RATE;
        }
        let new_speed = (old_speed - speed_reduction * time.delta_seconds()).max(0.0);

        if let None = levitates_option {
            let stop_flying_threshold;
            if let Some(gait) = gait_option {
                stop_flying_threshold = gait.stand_threshold;
            } else {
                stop_flying_threshold = 0.0;
            }
            if new_speed <= stop_flying_threshold {
                commands.entity(entity).remove::<Flying>();
                commands.entity(entity).insert(Grounded);
            }
        }

        if old_speed > 0.0 {
            velocity.value = velocity.value.normalize() * new_speed;
        }
    }
}

fn tripping(
    mut commands: Commands,
    query: Query<(Entity, &Gait, &Velocity), With<Grounded>>
) {
    for (entity, gait, velocity) in query.iter() {
        if velocity.value.length() > gait.trip_threshold {
            commands.entity(entity).remove::<Grounded>();
            commands.entity(entity).insert(Flying);
        }
    }
}

fn hollow_flying(mut query: Query<&mut Fill, Added<Flying>>) {
    for mut fill in query.iter_mut() {
        fill.color = Color::NONE;
    }
}

fn fill_grounded(mut query: Query<(&mut Fill, &Stroke), Added<Grounded>>) {
    for (mut fill, stroke) in query.iter_mut() {
        fill.color = stroke.color;
    }
}

fn gun_cooldown(
    mut query: Query<&mut Gun>,
    time: Res<Time>
) {
    for mut gun in query.iter_mut() {
        gun.cooldown_timer = (gun.cooldown_timer - time.delta_seconds()).max(0.0);
    }
}

fn progress_time_with_cooldown_interrupt(current: &mut f32, target: f32, cooldown: &mut f32) {
    // Move current up towards target but "stop" if cooldown ticks down towards 0 before then
    assert!(*current < target); // Not <= because we shouldn't be progressing time if we've already reached the target
    let delta = (target - *current).min(*cooldown);
    *current += delta;
    *cooldown -= delta;
}

fn shooting(
    mut commands: Commands,
    mut query: Query<(&mut Gun, &Velocity, &AngularVelocity, &Angle, &Position), With<Player>>, // TODO: Make velocities options
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>
) {
    if let Ok((mut gun, velocity, angular_velocity, angle, position)) = query.get_single_mut() {
        let mut rng = rand::thread_rng();

        let mut shoot = if gun.auto {
            keyboard_input.pressed(KeyCode::Space)
        } else {
            keyboard_input.just_pressed(KeyCode::Space)
        };
        
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

                let shape = shapes::Circle {
                    radius: gun.projectile_radius,
                    ..default()
                };

                let shooter_position = position.value + velocity.value * current_time;
                let shooter_angle = angle.value + angular_velocity.value * current_time;
                let aim_direction = Vec2::from_angle(shooter_angle);

                for _ in 0..gun.projectile_count {
                    // target_time - current_time is used a couple of times because the earlier the projectile was fired, the longer it has had for its properties to advance
                    let mut projectile_velocity = velocity.value + aim_direction * gun.projectile_speed +
                        random_vec2_circle(&mut rng, 1.0) * gun.projectile_spread; // In here because of projectile-specific use of random
                    let projectile_position = shooter_position +
                        aim_direction * gun.muzzle_distance +
                        projectile_velocity * (target_time - current_time);

                    // Simulate a bit of flying recovery
                    let old_speed = projectile_velocity.length();
                    let flying_recovery_rate = 500.0;
                    let speed_reduction = flying_recovery_rate;
                    let new_speed = (old_speed - speed_reduction * (target_time - current_time)).max(0.0);
                    if old_speed > 0.0 {
                        projectile_velocity = projectile_velocity.normalize() * new_speed;
                    }

                    commands.spawn((
                        Position {
                            value: projectile_position
                        },
                        Velocity {
                            value: projectile_velocity
                        },
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&shape),
                            transform: Transform { // In order to avoid projectiles appearing at the centre of the world for one frame
                                translation: Vec3::new(position.value.x, position.value.y, 0.0),
                                ..default()
                            },
                            ..default()
                        },
                        Fill::color(Color::NONE),
                        Stroke::new(Color::CYAN, 1.0),
                        Flying,
                        FlyingRecoveryRate {
                            value: flying_recovery_rate
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
