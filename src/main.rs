use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::prelude::*;

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
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_dots)
        .add_system(walking)
        .add_system(turning)
        .add_system(apply_velocity)
        .add_system(apply_angular_velocity)
		.add_system(follow_player)
		.add_system(update_transforms)
        .run();
}

#[derive(Component)]
struct Position {
    value: Vec2
}

#[derive(Component)]
struct Velocity {
    value: Vec2
}

#[derive(Component)]
struct Gait {
    max_speed: f32,
    acceleration: f32,
    stand_threshold: f32,
    trip_threshold: f32
}

#[derive(Component)]
struct Angle {
    value: f32
}

#[derive(Component)]
struct AngularVelocity {
    value: f32
}

#[derive(Component)]
struct AngularGait {
    max_speed: f32,
    acceleration: f32
}

#[derive(Component)]
struct Player {}

fn spawn_camera (mut commands: Commands) {
    commands.spawn(
        Camera2dBundle {
            ..default()
        }
    );
}

fn spawn_player (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn((
        Position {
            value: Vec2::ZERO
        },
        Velocity {
            value: Vec2::ZERO
        },
        Gait {
            max_speed: 100.0,
            acceleration: 800.0,
            stand_threshold: 110.0,
            trip_threshold: 120.0
        },
        Angle {
            value: 0.0
        },
        AngularVelocity {
            value: 0.0
        },
        AngularGait {
            max_speed: std::f32::consts::TAU / 2.0,
            acceleration: std::f32::consts::TAU * 8.0
        },
        Player {},
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(10.0).into()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        }
    ));
}

fn random_vec2_circle(rng: &mut rand::rngs::ThreadRng, radius: f32) -> Vec2 {
    let r = (rng.gen_range(0.0..1.0) as f32).powf(0.5) * radius;
    let theta = rng.gen_range(0.0..std::f32::consts::TAU);
    return Vec2::new(theta.cos() * r, theta.sin() * r);
}

fn spawn_dots (
    mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>
) {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        commands.spawn((
            Position {
                value: random_vec2_circle(&mut rng, 300.0) + Vec2::new(300.0, 0.0)
            },
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(2.0).into()).into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                transform: Transform::from_translation(Vec3::ZERO),
                ..default()
            }
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
    mut query: Query<(&mut Velocity, &Gait, Option<&Angle>), With<Player>>,
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
        let rotate_angle = entity_angle - std::f32::consts::TAU / 4.0;
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
    mut query: Query<(&mut AngularVelocity, &AngularGait), With<Player>>,
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
            camera_transform.rotation = Quat::from_rotation_z(entity_angle - std::f32::consts::TAU / 4.0);
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
