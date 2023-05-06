use bevy::prelude::*;

pub fn circle_circle(
    a_position: Vec2, a_velocity: Vec2, a_mass: f32, a_restitution: f32,
    b_position: Vec2, b_velocity: Vec2, b_mass: f32, b_restitution: f32
) -> (Vec2, Vec2) { // Returns accelerations
    let restitution = a_restitution.min(b_restitution);
    let direction = (a_position - b_position).normalize();
    let velocity_difference = b_velocity - a_velocity;
    let impact_speed = velocity_difference.dot(direction);
    if impact_speed > 0.0 {
        let speed_1 = ((restitution + 1.0) * b_mass * impact_speed) / (a_mass + b_mass);
        let speed_2 = ((restitution + 1.0) * a_mass * impact_speed) / (a_mass + b_mass);
        return (
            direction * speed_1,
            -direction * speed_2
        );
    }
    return (Vec2::ZERO, Vec2::ZERO);
}
