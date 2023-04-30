use crate::components::*;
use bevy::prelude::*;
use std::f32::consts::TAU;

fn proper_signum(x: f32) -> f32 {
    if x > 0.0 {
        return 1.0;
    } else if x < 0.0 {
        return -1.0;
    } else {
        return 0.0;
    }
}

fn locomotion_handle_axis(current: f32, target: f32, acceleration: f32, delta_seconds: f32) -> f32 {
    if acceleration > 0.0 {
        return target.min(current + acceleration * delta_seconds);
    } else {
        return target.max(current + acceleration * delta_seconds);
    }
}

pub fn walking(
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

pub fn turning(
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
