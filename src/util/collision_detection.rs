use bevy::prelude::*;

pub fn circle_circle(a_radius: f32, a_position: Vec2, b_radius: f32, b_position: Vec2) -> bool {
    // Both shapes are filled, not hollow
    return a_position.distance(b_position) <= a_radius + b_radius;
}

pub fn _circle_aabb(a_radius: f32, a_position: Vec2, b_width: f32, b_height: f32, b_position: Vec2) -> bool { // b_position is top left corner
    // Both shapes are filled, not hollow
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

pub fn circle_point(a_radius: f32, a_position: Vec2, b: Vec2) -> bool {
    // The circle is filled, not hollow
    return a_position.distance(b) <= a_radius;
}

pub fn line_circle_intersection(line_start: Vec2, line_end: Vec2, circle_radius: f32, circle_position: Vec2) -> Option<(f32, f32)> {
    // Includes intersection times outside the interval [0, 1] (line_start and line_end represent a line segment that lies along the line).
    // The out intersection will be (negative and) closer to the line segment than the (negative) in intersection when the circle is behind the line segment.

    if line_start == line_end {
        return None; // I don't know if this refers to zero lines or infinitely many lines, but it isn't well-defined
    }
    let start_to_end = line_end - line_start;
    let circle_to_start = line_start - circle_position;

    let a = start_to_end.dot(start_to_end);
    let b = 2.0 * circle_to_start.dot(start_to_end);
    let c = circle_to_start.dot(circle_to_start) - circle_radius * circle_radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let discriminant_sqrt = discriminant.sqrt();
    return Some((
        (-discriminant_sqrt - b) / (2.0 * a),
        (discriminant_sqrt - b) / (2.0 * a)
    ));
}
