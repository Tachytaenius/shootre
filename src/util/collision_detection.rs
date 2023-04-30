use bevy::prelude::*;

pub fn circle_circle(a_radius: f32, a_position: Vec2, b_radius: f32, b_position: Vec2) -> bool {
    return a_position.distance(b_position) <= a_radius + b_radius;
}

pub fn _circle_aabb(a_radius: f32, a_position: Vec2, b_width: f32, b_height: f32, b_position: Vec2) -> bool { // b_position is top left corner
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
