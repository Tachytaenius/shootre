use bevy::prelude::*;

pub fn circle_circle(
    a_position: Vec2, a_radius: f32, a_velocity: Vec2, a_mass: f32, a_restitution: f32,
    b_position: Vec2, b_radius: f32, b_velocity: Vec2, b_mass: f32, b_restitution: f32
) -> ((Vec2, Vec2), (Vec2, Vec2)) { // Returns tuple of velocity changes and tuple of position changes
    // Something in here is probably wrong even though it works.
    // Things that should have one sign get flipped to another and the code feels inconsistent.

    let difference = a_position - b_position;
    let direction = difference.normalize();
    let distance_to_separate = difference.length() - a_radius - b_radius;
    let a_distance_movement_share = b_mass / (a_mass + b_mass);
    let b_distance_movement_share = a_mass / (a_mass + b_mass);
    // a_distance_movement_share + b_distance_movement_share is (ignoring float imprecision) equal to 1
    let position_changes = (
        -direction * distance_to_separate * a_distance_movement_share,
        direction * distance_to_separate * b_distance_movement_share
    );

    let velocity_difference = b_velocity - a_velocity;
    let impact_speed = velocity_difference.dot(direction);
    if impact_speed > 0.0 {
        let restitution = a_restitution.min(b_restitution);
        let speed_1 = ((restitution + 1.0) * b_mass * impact_speed) / (a_mass + b_mass);
        let speed_2 = ((restitution + 1.0) * a_mass * impact_speed) / (a_mass + b_mass);
        let velocity_changes = (
            direction * speed_1,
            -direction * speed_2
        );
        return (velocity_changes, position_changes);
    }
    return ((Vec2::ZERO, Vec2::ZERO), position_changes);
}

pub fn circle_aabb(
    a_radius: f32, a_position: Vec2, a_velocity: Vec2, a_mass: f32, a_restitution: f32,
    b_width: f32, b_height: f32, b_position: Vec2, b_velocity: Vec2, b_mass: f32, b_restitution: f32 // b_position is top left corner
) -> ((Vec2, Vec2), (Vec2, Vec2)) { // Returns tuple of velocity changes and tuple of position changes
    // Ported and modified from tinyc2

    // Something in here is probably wrong even though it works.
    // Things that should have one sign get flipped to another and the code feels inconsistent.

    let l = a_position.clamp(b_position, b_position + Vec2::new(b_width, b_height));
    let ab = l - a_position;
    let d2 = ab.dot(ab);
    // let r2 = a_radius * a_radius;
    // debug_assert!(d2 <= r2);

    if d2 != 0.0 { // shallow (centre of circle not inside AABB)
        let distance = d2.sqrt();
        let direction = ab.normalize();

        let distance_to_separate = a_radius - distance;
        let a_distance_movement_share = b_mass / (a_mass + b_mass);
        let b_distance_movement_share = a_mass / (a_mass + b_mass);
        let position_changes = (
            -direction * distance_to_separate * a_distance_movement_share,
            direction * distance_to_separate * b_distance_movement_share
        );

        let restitution = a_restitution.min(b_restitution);
        let velocity_difference = b_velocity - a_velocity;
        let impact_speed = velocity_difference.dot(direction);
        let impact_speed = -impact_speed; // HACK
        if impact_speed > 0.0 {
            let speed_1 = ((restitution + 1.0) * b_mass * impact_speed) / (a_mass + b_mass);
            let speed_2 = ((restitution + 1.0) * a_mass * impact_speed) / (a_mass + b_mass);
            let velocity_changes = (
                -direction * speed_1,
                direction * speed_2
            );
            return (velocity_changes, position_changes);
        }
        return ((Vec2::ZERO, Vec2::ZERO), position_changes);
    } else { // deep (centre of circle inside AABB)
        let mid = (b_position + b_position + Vec2::new(b_width, b_height)) * 0.5;
        let e = Vec2::new(b_width, b_height) * 0.5;
        let d = a_position - mid;
        let abs_d = d.abs();

        let x_overlap = e.x - abs_d.x;
        let y_overlap = e.y - abs_d.y;

        let depth;
        let direction;

        if x_overlap < y_overlap {
            depth = x_overlap;
            direction = Vec2::new(if d.x < 0.0 {1.0} else {0.0}, 0.0);
        } else {
            depth = y_overlap;
            direction = Vec2::new(0.0, if d.y < 0.0 {1.0} else {-1.0});
        }

        let distance_to_separate = a_radius + depth;
        let a_distance_movement_share = b_mass / (a_mass + b_mass);
        let b_distance_movement_share = a_mass / (a_mass + b_mass);
        let position_changes = (
            -direction * distance_to_separate * a_distance_movement_share,
            direction * distance_to_separate * b_distance_movement_share
        );

        let restitution = a_restitution.min(b_restitution);
        let velocity_difference = b_velocity - a_velocity;
        let impact_speed = velocity_difference.dot(direction);
        let impact_speed = -impact_speed; // HACK
        if impact_speed > 0.0 {
            let speed_1 = ((restitution + 1.0) * b_mass * impact_speed) / (a_mass + b_mass);
            let speed_2 = ((restitution + 1.0) * a_mass * impact_speed) / (a_mass + b_mass);
            let velocity_changes = (
                -direction * speed_1,
                direction * speed_2
            );
            return (velocity_changes, position_changes);
        }
        return ((Vec2::ZERO, Vec2::ZERO), position_changes);
    }
}
