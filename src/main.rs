// This project may have TODOs in it

mod components;

use bevy::prelude::*;
use std::f32::consts::TAU;
use rand::prelude::*;
use bevy_prototype_lyon::prelude::*;
use components::*;

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
    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    enum PreUpdateSet {Main, CommandFlush}

    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    struct MainSet;

    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    struct ConsistentStateChecks;

    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    enum RenderPreparationSet {CommandFlush, Main}

    let mut app = App::new();

    app // TODO: Work out deterministic-but-still-parallelised system order
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))

        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_other)
        .add_startup_system(spawn_dots)

        .add_systems((
            store_previous_position,
            store_previous_angle,
            remove_spawned_mid_tick,
            clear_wills
        ).in_set(PreUpdateSet::Main).before(PreUpdateSet::CommandFlush))
        .add_system(apply_system_buffers.in_set(PreUpdateSet::CommandFlush).before(MainSet))

        .add_systems((
            player_input.before(dropping),
            // ai.before(dropping),
            dropping.before(picking_up),
            picking_up.before(turning).before(walking),
            walking.before(collision),
            turning.before(shooting),
            collision.before(shooting),
            shooting.before(apply_velocity).before(apply_angular_velocity),
            apply_velocity.before(manage_flyers).before(tripping),
            apply_angular_velocity,
            manage_flyers.before(manage_flooreds),
            manage_flooreds.before(floor_friction), // This comes before floor_friction so that friction can be skipped in case the timer starts at zero
            floor_friction.before(tripping),
            tripping,
            gun_cooldown
        ).in_set(MainSet).before(RenderPreparationSet::CommandFlush));

    #[cfg(debug_assertions)]
    app.add_systems((
        check_consistent_grounded_flying_state,
        check_consistent_hierarchy_state
    ).in_set(ConsistentStateChecks).after(MainSet).before(RenderPreparationSet::CommandFlush));

    app
        .add_system(apply_system_buffers.in_set(RenderPreparationSet::CommandFlush).before(RenderPreparationSet::Main))
        .add_systems((
            hollow_flying,
            fill_grounded,
            follow_player,
            update_transforms,
            rebuild_traced_shape,
            rebuild_collider_shape
        ).in_set(RenderPreparationSet::Main));

    app.run();
}

fn spawn_camera (mut commands: Commands) {
    commands.spawn(
        Camera2dBundle {
            ..default()
        }
    );
}

fn spawn_player(
    mut commands: Commands
) {
    let _machine_gun = Gun {
        projectile_speed: 2000.0,
        projectile_flying_recovery_rate: 250.0,
        projectile_spread: Vec2::new(0.005, 0.005),
        projectile_count: 1,
        projectile_colour: Color::CYAN,
        muzzle_distance: 12.5,
        cooldown: 0.01,
        auto: true,

        cooldown_timer: 0.0
    };
    let shotgun = Gun {
        projectile_speed: 1750.0,
        projectile_flying_recovery_rate: 500.0,
        projectile_spread: Vec2::new(0.05, 0.05),
        projectile_count: 25,
        projectile_colour: Color::CYAN,
        muzzle_distance: 11.0,
        cooldown: 1.0,
        auto: false,

        cooldown_timer: 0.0
    };
    let position = Vec2::ZERO;
    let angle = 0.0;
    let player = commands.spawn((
        ( // Nested to get around bundle size limit
            Position {value: position},
            PreviousPosition {value: position},
            Velocity {value: Vec2::ZERO},
            Gait {
                standing_max_speed: 200.0,
                standing_acceleration: 800.0,
                floored_max_speed: 100.0,
                floored_acceleration: 400.0,
                floored_recovery_time: 2.0
            },
            FlyingRecoveryRate {value: 800.0},
            RegroundThreshold {value: 210.0},
            TripThreshold {value: 220.0}
        ),
        (
            Angle {value: angle},
            PreviousAngle {value: angle},
            AngularVelocity {value: 0.0},
            AngularGait {
                max_speed: TAU / 2.0,
                acceleration: TAU * 8.0
            },
        ),
        (
            Collider {
                radius: 10.0,
                solid: true
            },
            Mass {value: 100.0},
            Restitution {value: 0.2},
            FloorFriction {value: 300.0}
        ),
        (
            ShapeBundle {
                // Path is created by rebuild_collider_shape before rendering
                ..default()
            },
            Fill::color(Color::WHITE),
            Stroke::new(Color::WHITE, 1.0)
        ),
        Player,
        Will {..default()},
        Grounded {
            standing: true,
            floored_recovery_timer: None
        },
        shotgun,
        Holder {pick_up_range: 20.0}
    ));
}

fn spawn_other(
    mut commands: Commands
) {
    let position = Vec2::new(100.0, 0.0);
    commands.spawn((
        (
            Position {value: position},
            Velocity {value: Vec2::ZERO}
        ),
        (
            Collider {
                radius: 5.0,
                solid: false
            },
            Mass {value: 10.0},
            Restitution {value: 0.4},
            FloorFriction {value: 200.0}
        ),
        (
            ShapeBundle {
                ..default()
            },
            Fill::color(Color::WHITE),
            Stroke::new(Color::WHITE, 1.0)
        ),
        Grounded {
            standing: false,
            floored_recovery_timer: None
        },
        Holdable
    ));
}

fn random_vec2_circle(rng: &mut rand::rngs::ThreadRng, radius: f32) -> Vec2 {
    let r = (rng.gen_range(0.0..1.0) as f32).powf(0.5) * radius;
    let theta = rng.gen_range(0.0..TAU);
    return Vec2::new(theta.cos() * r, theta.sin() * r);
}

fn spawn_dots(
    mut commands: Commands
) {
    let shape = shapes::Circle {
        radius: 2.0,
        ..default()
    };
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        commands.spawn((
            Position {value: random_vec2_circle(&mut rng, 1000.0)},
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

fn player_input(
    mut query: Query<&mut Will, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>
) {
    if let Ok(mut will) = query.get_single_mut() {
        let mut target = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::A) {
            target.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            target.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::W) {
            target.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            target.y -= 1.0;
        }
        if target != Vec2::ZERO {
            target = target.normalize();
        }
        will.target_relative_velocity_multiplier = Some(target);

        let mut target = 0.0;
        if keyboard_input.pressed(KeyCode::Comma) {
            target += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Period) {
            target -= 1.0;
        }
        will.target_angular_velocity_multiplier = Some(target);

        will.drop = keyboard_input.just_pressed(KeyCode::Q);
        will.pick_up = keyboard_input.just_pressed(KeyCode::F);
    }
}

fn walking(
    mut query: Query<(
        &mut Velocity,
        &Gait,
        &Will,
        Option<&Angle>,
        Option<&Grounded>,
        Option<&Levitates>
    )>,
    time: Res<Time>
) {
    for (mut velocity, gait, will, angle_option, grounded_option, levitates_option) in query.iter_mut() {
        if !(grounded_option.is_some() || levitates_option.is_some()) {
            continue; // Not grounded *or* levitating, can't walk
        }

        let max_speed;
        let acceleration;
        if let None = levitates_option {
            // Grounded is definitely some
            let grounded = grounded_option.unwrap();
            if grounded.standing {
                max_speed = gait.standing_max_speed;
                acceleration = gait.standing_acceleration;
            } else {
                max_speed = gait.floored_max_speed;
                acceleration = gait.floored_acceleration;
            }
        } else {
            max_speed = gait.standing_max_speed;
            acceleration = gait.standing_acceleration;
        }

        let target_relative_velocity = will.target_relative_velocity_multiplier.unwrap_or(Vec2::ZERO) * max_speed;
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
        let acceleration_distribution = direction * acceleration; // So that you don't get to use all of acceleration on both axes

        relative_velocity.x = locomotion_handle_axis(relative_velocity.x, target_relative_velocity.x, acceleration_distribution.x, time.delta_seconds());
        relative_velocity.y = locomotion_handle_axis(relative_velocity.y, target_relative_velocity.y, acceleration_distribution.y, time.delta_seconds());

        velocity.value = Vec2::from_angle(rotate_angle).rotate(relative_velocity);
    }
}

fn turning(
    mut query: Query<
        (
            &mut AngularVelocity,
            &AngularGait,
            &Will
        ),
        Or<(
            With<Grounded>,
            With<Levitates>
        )>
    >,
    time: Res<Time>
) {
    for (mut angular_velocity, angular_gait, will) in query.iter_mut() {
        let target_angular_velocity = will.target_angular_velocity_multiplier.unwrap_or(0.0) * angular_gait.max_speed;
        angular_velocity.value = locomotion_handle_axis(
            angular_velocity.value,
            target_angular_velocity,
            angular_gait.acceleration * proper_signum(target_angular_velocity - angular_velocity.value),
            time.delta_seconds()
        );
    }
}

fn apply_velocity(
    mut query: Query<(&mut Position, &Velocity)>,
    time: Res<Time>
) {
    for (mut position, velocity) in query.iter_mut() {
        position.value += velocity.value * time.delta_seconds();
    }
}

fn apply_angular_velocity(
    mut query: Query<(&mut Angle, &AngularVelocity)>,
    time: Res<Time>
) {
    for (mut angle, angular_velocity) in query.iter_mut() {
        angle.value += angular_velocity.value * time.delta_seconds();
    }
}

fn follow_player(
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

fn update_transforms(mut query: Query<(&mut Transform, Option<&Position>, Option<&Angle>, Option<&Parent>, Option<&ParentRelationship>)>) {
    for (mut transform, position_option, angle_option, parent_option, parent_relationship_option) in query.iter_mut() {
        if let Some(_) = parent_option {
            if let ParentRelationship::Holder {held_distance, ..} = *parent_relationship_option.unwrap() {
                transform.translation = Vec3::new(held_distance, 0.0, 0.0);
            }
        } else if let Some(position) = position_option {
            let angle;
            if let Some(angle_component) = angle_option {
                angle = angle_component.value;
            } else {
                angle = 0.0;
            }
            transform.translation = Vec3::new(position.value.x, position.value.y, 0.0);
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn manage_flyers(
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

fn manage_flooreds(
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

fn floor_friction(
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

fn tripping(
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

// TODO: Don't do duplicate gun cooldown! We need an inventory system with guns tracking depressed trigger etc themsevles
// "Pull trigger" system involving holder entity in query -> gun system that handles cooldown etc and only uses holder entity to get where it's being shot from

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
    debug_assert!(*current < target); // Not <= because we shouldn't be progressing time if we've already reached the target
    let delta = (target - *current).min(*cooldown);
    *current += delta;
    *cooldown -= delta;
}

fn shooting(
    mut commands: Commands,
    mut query: Query<(&mut Gun, Option<&Velocity>, Option<&AngularVelocity>, &Angle, &Position), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>
) {
    if let Ok((mut gun, velocity_option, angular_velocity_option, angle, position)) = query.get_single_mut() {
        let velocity_value;
        if let Some(velocity) = velocity_option {
            velocity_value = velocity.value;
        } else {
            velocity_value = Vec2::ZERO;
        }

        let angular_velocity_value;
        if let Some(angular_velocity) = angular_velocity_option {
            angular_velocity_value = angular_velocity.value;
        } else {
            angular_velocity_value = 0.0;
        }

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

                let shooter_position = position.value + velocity_value * current_time;
                let shooter_angle = angle.value + angular_velocity_value * current_time;
                let aim_direction = Vec2::from_angle(shooter_angle);
                let projectile_origin = shooter_position + aim_direction * gun.muzzle_distance;

                for _ in 0..gun.projectile_count {
                    // target_time - current_time is used a couple of times because the earlier the projectile was fired, the longer it has had for its properties to advance
                    let mut projectile_velocity = velocity_value + aim_direction * gun.projectile_speed +
                        random_vec2_circle(&mut rng, 1.0) * gun.projectile_spread * gun.projectile_speed; // In here because of projectile-specific use of random
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
                        SpawnedMidTick {when: current_time / target_time}
                    ));
                }
            } else {
                // If we're not shooting (or gun.cooldown_timer failed to reach 0 before current_time reached target_time)
                break;
            }
        }
    }
}

fn store_previous_position(mut query: Query<(&mut PreviousPosition, &Position)>) {
    for (mut previous_position, position) in query.iter_mut() {
        previous_position.value = position.value;
    }
}

fn store_previous_angle(mut query: Query<(&mut PreviousAngle, &Angle)>) {
    for (mut previous_angle, angle) in query.iter_mut() {
        previous_angle.value = angle.value;
    }
}

const DRAW_TRACER_AS_POINT_THRESHOLD: f32 = 1.0;
const TRACER_POINT_CIRCLE_RADIUS: f32 = 0.1;

fn rebuild_traced_shape(
    mut commands: Commands,
    mut tracer_query: Query<(Entity, &mut Stroke, Option<&SpawnedMidTick>, &ProjectileColour, &Position, &PreviousPosition), (With<Path>, With<TracedLine>)>,
    player_query: Query<(&Position, &Angle, &PreviousPosition, &PreviousAngle), With<Player>>
) {
    // NOTE: It might be better if this was in terms of rotation and translation directly, and not using Transform objects.
    if let Ok((player_position, player_angle, player_previous_position, player_previous_angle)) = player_query.get_single() {
        for (entity, mut tracer_stroke, tracer_spawned_mid_tick_option, tracer_projectile_colour, tracer_position, tracer_previous_position) in tracer_query.iter_mut() {
            let previous_transform_lerp;
            if let Some(tracer_spawned_mid_tick) = tracer_spawned_mid_tick_option {
                previous_transform_lerp = tracer_spawned_mid_tick.when;
            } else {
                previous_transform_lerp = 0.0;
            }

            let player_previous_position = // Shadow
                player_previous_position.value * (1.0 - previous_transform_lerp)
                + player_position.value * previous_transform_lerp;

            let player_previous_camera_transform = Transform {
                translation: Vec3::new(player_previous_position.x, player_previous_position.y, 0.0),
                rotation: Quat::from_rotation_z(
                    player_previous_angle.value * (1.0 - previous_transform_lerp)
                    + player_angle.value * previous_transform_lerp
                ),
                ..default()
            };
            let player_current_camera_transform = Transform {
                translation: Vec3::new(player_position.value.x, player_position.value.y, 0.0),
                rotation: Quat::from_rotation_z(player_angle.value),
                ..default()
            };

            let tracer_previous_screen_space_position_4d = player_previous_camera_transform.compute_matrix().inverse() * // Inverting because camera
                Vec4::new(tracer_previous_position.value.x, tracer_previous_position.value.y, 0.0, 1.0);
            let tracer_current_screen_space_position_4d = player_current_camera_transform.compute_matrix().inverse() *
                Vec4::new(tracer_position.value.x, tracer_position.value.y, 0.0, 1.0);

            let path_vector = Vec2::from_angle(player_angle.value).rotate( // Transform out of screen space back into world space, but keeping the difference
                Vec2::new(tracer_previous_screen_space_position_4d.x, tracer_previous_screen_space_position_4d.y)
                - Vec2::new(tracer_current_screen_space_position_4d.x, tracer_current_screen_space_position_4d.y)
            );

            if path_vector.length() <= DRAW_TRACER_AS_POINT_THRESHOLD {
                let circle = shapes::Circle {
                    radius: TRACER_POINT_CIRCLE_RADIUS,
                    center: path_vector
                };
                commands.entity(entity).insert(GeometryBuilder::build_as(&circle));
            } else {
                let line = shapes::Line(Vec2::ZERO, path_vector);
                tracer_stroke.color.set_a(
                    tracer_projectile_colour.value.a()
                    * (1.0 / path_vector.length()).min(1.0)
                );
                commands.entity(entity).insert(GeometryBuilder::build_as(&line));
            }
        }
    }
}

fn remove_spawned_mid_tick(
    mut commands: Commands,
    query: Query<Entity, With<SpawnedMidTick>>
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<SpawnedMidTick>();
    }
}

fn circle_circle_collision_detection(a_radius: f32, a_position: Vec2, b_radius: f32, b_position: Vec2) -> bool {
    return a_position.distance(b_position) <= a_radius + b_radius;
}

fn circle_circle_collision_resolution(
    a_position: Vec2, a_velocity: Vec2, a_mass: f32, a_restitution: f32,
    b_position: Vec2, b_velocity: Vec2, b_mass: f32, b_restitution: f32
) -> (Vec2, Vec2) { // Returns new velocities
    let restitution = a_restitution.min(b_restitution);
    let direction = (a_position - b_position).normalize();
    let velocity_difference = b_velocity - a_velocity;
    let impact_speed = velocity_difference.dot(direction);
    if impact_speed > 0.0 {
        let speed_1 = ((restitution + 1.0) * b_mass * impact_speed) / (a_mass + b_mass);
        let speed_2 = ((restitution + 1.0) * a_mass * impact_speed) / (a_mass + b_mass);
        return (
            a_velocity + direction * speed_1,
            b_velocity - direction * speed_2
        );
    }
    return (a_velocity, b_velocity);
}

fn _circle_aabb_collision_detection(a_radius: f32, a_position: Vec2, b_width: f32, b_height: f32, b_position: Vec2) -> bool { // b_position is top left corner
    let mut test = a_position;

    if a_position.x < b_position.x {
        test.x = b_position.x;
    } else if a_position.x > b_position.x + b_width {
        test.x = b_position.x + b_width;
    }

    if a_position.y < b_position.y {
        test.y = b_position.y;
    } else if a_position.y > b_position.y + b_height {
        test.y = b_position.y + b_height;
    }

    return a_position.distance(test) <= a_radius;
}

fn collision(
    mut query: Query<(&Collider, &Position, &mut Velocity, Option<&Mass>, Option<&Restitution>)>
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([
        (a_collider, a_position, mut a_velocity, a_mass_option, a_restitution_option),
        (b_collider, b_position, mut b_velocity, b_mass_option, b_restitution_option)
    ]) = combinations.fetch_next() {
        if !(a_collider.solid && b_collider.solid) {
            continue;
        }
        if circle_circle_collision_detection(a_collider.radius, a_position.value, b_collider.radius, b_position.value) {
            let a_mass;
            if let Some(a_mass_component) = a_mass_option {
                a_mass = a_mass_component.value;
            } else {
                a_mass = 1.0;
            }
            let b_mass;
            if let Some(b_mass_component) = b_mass_option {
                b_mass = b_mass_component.value;
            } else {
                b_mass = 1.0;
            }

            let a_restitution;
            if let Some(a_restitution_component) = a_restitution_option {
                a_restitution = a_restitution_component.value;
            } else {
                a_restitution = 1.0;
            }
            let b_restitution;
            if let Some(b_restitution_component) = b_restitution_option {
                b_restitution = b_restitution_component.value;
            } else {
                b_restitution = 1.0;
            }

            (a_velocity.value, b_velocity.value) = circle_circle_collision_resolution(
                 a_position.value, a_velocity.value, a_mass, a_restitution,
                b_position.value, b_velocity.value, b_mass, b_restitution
            );
        }
    }
}

fn _monitor_conservation(query: Query<(&Velocity, &Mass)>) {
    let mut energy = 0.0;
    let mut momentum = Vec2::ZERO;
    for (velocity, mass) in query.iter() {
        energy += mass.value / 2.0 * velocity.value.length_squared();
        momentum += velocity.value * mass.value;
    }
    println!("Energy: {}, Momentum: {}", energy, momentum);
}

fn check_consistent_grounded_flying_state(
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

fn check_consistent_hierarchy_state(
    child_query: Query<(Entity, &Parent)>,
    holder_query: Query<&Holder>,
    holdable_query: Query<&Holdable>,
    child_type_query: Query<(Entity, &ParentRelationship)>,
    position_query: Query<&Position>,
    velocity_query: Query<&Velocity>,
    angle_query: Query<&Angle>,
    angular_velocity_query: Query<&AngularVelocity>,
    grounded_query: Query<&Grounded>,
    flying_query: Query<&Flying>,
    parents_with_children_query: Query<(With<Parent>, With<Children>)>
) {
    // Check that the set of all entities with Parent and the set of all entities with ParentRelationship is the same
    // Check that no children have spatial information components
    // Check that all held entities have Holdable and that their holders have Holder
    // Check that there are no entities with a parent and children (proper chains of entities would be a huge challenge)

    for (child_entity, parent) in child_query.iter() {
        assert!(child_type_query.contains(child_entity));

        assert!(!position_query.contains(child_entity));
        assert!(!velocity_query.contains(child_entity));
        assert!(!angle_query.contains(child_entity));
        assert!(!angular_velocity_query.contains(child_entity));
        assert!(!grounded_query.contains(child_entity));
        assert!(!flying_query.contains(child_entity));

        let (_, parent_relationship_type) = child_type_query.get(child_entity).unwrap();
        match parent_relationship_type {
            ParentRelationship::Holder {..} => {
                assert!(holdable_query.contains(child_entity));
                assert!(holder_query.contains(parent.get()));
            },
            _ => {}
        }
    }

    for (child_type_entity, _) in child_type_query.iter() {
        assert!(child_query.contains(child_type_entity));
    }

    if !parents_with_children_query.is_empty() {
        panic!();
    }
}

fn clear_wills(
    mut commands: Commands,
    query: Query<Entity, With<Will>>
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<Will>();
        commands.entity(entity).insert(Will {..default()});
    }
}

fn rebuild_collider_shape(
    mut commands: Commands,
    query: Query<(Entity, &Collider), With<Path>>
) {
    for (entity, collider) in query.iter() {
        let shape = shapes::Circle {
            radius: collider.radius,
            ..default()
        };
        commands.entity(entity).insert(GeometryBuilder::build_as(&shape));
    }
}

fn dropping(
    mut commands: Commands,
    holder_query: Query<(Entity, &Will, &Children), With<Holder>>,
    position_query: Query<&Position>,
    velocity_query: Query<&Velocity>,
    angle_query: Query<&Angle>,
    angular_velocity_query: Query<&AngularVelocity>,
    drop_as_grounded_query: Query<&RegroundThreshold, Without<Levitates>>,
    gait_query: Query<&Gait>,
    child_query: Query<&ParentRelationship, With<Parent>>
) {
    for (parent_entity, will, children) in holder_query.iter() {
        if !will.drop {
            continue;
        }

        for child_entity in children.iter() {
            let parent_relationship_type = child_query.get(*child_entity).unwrap();
            match parent_relationship_type {
                ParentRelationship::Holder {held_distance, held_angle} => {
                    let mut child_commands = commands.entity(*child_entity);
                    child_commands.remove_parent();
                    child_commands.remove::<ParentRelationship>();
                    
                    if let Ok(position) = position_query.get(parent_entity) {
                        let angle;
                        if let Ok(angle_component) = angle_query.get(parent_entity) {
                            angle = angle_component.value;
                        } else {
                            angle = 0.0;
                        }
                        child_commands.insert(Position {value: position.value + Vec2::from_angle(angle).rotate(Vec2::new(*held_distance, 0.0))});
                    }
                    if let Ok(velocity) = velocity_query.get(parent_entity) {
                        child_commands.insert(Velocity {value: velocity.value});

                        let reground_threshold;
                        if let Ok(reground_threshold_component) = drop_as_grounded_query.get(*child_entity) {
                            reground_threshold = reground_threshold_component.value;
                        } else {
                            reground_threshold = DEFAULT_REGROUND_THRESHOLD;
                        }

                        if velocity.value.length() <= reground_threshold {
                            child_commands.insert(Grounded {
                                standing: gait_query.contains(*child_entity),
                                floored_recovery_timer: None
                            });
                        } else {
                            child_commands.insert(Flying);
                        }
                    }
                    if let Ok(angle) = angle_query.get(parent_entity) {
                        child_commands.insert(Angle {value: angle.value});
                    }
                    if let Ok(angular_velocity) = angular_velocity_query.get(parent_entity) {
                        child_commands.insert(AngularVelocity {value: angular_velocity.value});
                    }
                },
                _ => {
                    continue;
                }
            }
        }
    }
}

fn picking_up(
    mut commands: Commands,
    holder_query: Query<(Entity, &Will, Option<&Children>, &Position, &Holder)>,
    pick_up_able_query: Query<(Entity, &Position), (With<Holdable>, Without<Parent>)>
) {
    for (holder_entity, will, children_option, position, holder) in holder_query.iter() {
        if !will.pick_up {
            continue;
        }
        if let Some(children) = children_option {
            if children.len() > 0 {
                continue;
            }
        }

        for (potential_child_entity, potential_child_position) in pick_up_able_query.iter() {
            if position.value.distance(potential_child_position.value) <= holder.pick_up_range {
                // Shouldn't matter if two entities pick up the same entity on the same tick (TODO: test)
                commands.entity(holder_entity).push_children(&[potential_child_entity]);
                let mut child_commands = commands.entity(potential_child_entity);
                child_commands.insert(ParentRelationship::Holder {
                    held_distance: 12.0,
                    held_angle: 0.0
                });
                child_commands.remove::<Position>();
                child_commands.remove::<Velocity>();
                child_commands.remove::<Angle>();
                child_commands.remove::<AngularVelocity>();
                child_commands.remove::<Grounded>();
                child_commands.remove::<Flying>();
                break;
            }
        }
    }
}
