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

pub struct GridRaycast {
    tile_x: i32,
    tile_y: i32,
    delta_tile_x: i32,
    delta_tile_y: i32,
    delta_t_x: f32,
    delta_t_y: f32,
    delta_delta_t_x: f32,
    delta_delta_t_y: f32,
    t: f32,
    last_t: Option<f32>,

    ray_start_x: f32, // Relative to the frame where the grid offset is 0
    ray_start_y: f32,
    ray_end_x: f32,
    ray_end_y: f32
}

pub struct GridRaycastResult {
    pub tile_x: i32,
    pub tile_y: i32,
    pub intersection_t: f32
}

impl Iterator for GridRaycast {
    type Item = GridRaycastResult;

    fn next(&mut self) -> Option<GridRaycastResult> {
        if
            self.ray_start_x == self.ray_end_x &&
            self.ray_start_y == self.ray_end_y
        { // Is there a good way to not do this check every iteration?
            if self.last_t.is_some() {
                return None;
            } else {
                debug_assert!(self.t == 0.0); // Should be undefined, really
                self.last_t = Some(self.t);
                return Some(GridRaycastResult {
                    // For rays with direction (mathematical definitions break down in programming) that start on a tile boundary, the initial tile coords depend on the direction.
                    // This result is just arbitrarily choosing a direction that returns these tile coords in those cases.
                    tile_x: self.tile_x,
                    tile_y: self.tile_y,
                    intersection_t: self.t
                });
            }
        }

        // This loop is only here for skipping iterations that have the same t as the previous one
        loop {
            if self.t > 1.0 {
                return None;
            }

            let result = GridRaycastResult {
                tile_x: self.tile_x,
                tile_y: self.tile_y,
                intersection_t: self.t
            };
            self.last_t = Some(self.t);

            if self.delta_t_x < self.delta_t_y {
                self.tile_x += self.delta_tile_x;
                let delta_t = self.delta_t_x;
                self.t += delta_t;
                self.delta_t_x += self.delta_delta_t_x - delta_t;
                self.delta_t_y -= delta_t;
            } else {
                self.tile_y += self.delta_tile_y;
                let delta_t = self.delta_t_y;
                self.t += delta_t;
                self.delta_t_x -= delta_t;
                self.delta_t_y += self.delta_delta_t_y - delta_t;
            }

            if self.t == self.last_t.unwrap() {
                // Unlike what the code was intended to do, this actually ends up skipping all the but the *last* of a run of identical intersections.
                // This is actually the right way to do it, though, because it returns intersection tile coordinates for the correct tiles when doing
                // diagonals that go through corners and/or starting on a tile boundary (4-way or 2).
                continue;
            }

            return Some(result);
        }
    }
}

pub fn new_grid_raycast(ray_start: Vec2, ray_end: Vec2, tile_width: f32, tile_height: f32, tilemap_position: Vec2) -> GridRaycast {
    // (It's a directed line segment, not a ray)

    fn get_helpers(tile_size: f32, pos: f32, dir: f32) -> (i32, i32, f32, f32) {
        let tile = (pos / tile_size).floor() as i32; // floor makes a difference for negatives

        let (delta_tile, delta_t);
        if dir > 0.0 {
            delta_tile = 1;
            delta_t = ((tile as f32 + 1.0) * tile_size - pos) / dir;
        } else if dir == 0.0 {
            delta_tile = 0;
            delta_t = ((tile as f32 + 1.0) * tile_size - pos) / dir;
        } else {
            delta_tile = -1;
            delta_t = ((tile as f32 + 0.0) * tile_size - pos) / dir;
        }

        let delta_delta_t = delta_tile as f32 * tile_size / dir;

        return (tile, delta_tile, delta_t, delta_delta_t);
    }

    let (tile_x, delta_tile_x, delta_t_x, delta_delta_t_x) = get_helpers(tile_width, ray_start.x - tilemap_position.x, (ray_end - ray_start).x);
    let (tile_y, delta_tile_y, delta_t_y, delta_delta_t_y) = get_helpers(tile_height, ray_start.y - tilemap_position.y, (ray_end - ray_start).y);

    return GridRaycast {
        tile_x: tile_x,
        tile_y: tile_y,
        delta_tile_x: delta_tile_x,
        delta_tile_y: delta_tile_y,
        delta_t_x: delta_t_x,
        delta_t_y: delta_t_y,
        delta_delta_t_x: delta_delta_t_x,
        delta_delta_t_y: delta_delta_t_y,
        t: 0.0,
        last_t: None,

        ray_start_x: ray_start.x,
        ray_start_y: ray_start.y,
        ray_end_x: ray_end.x,
        ray_end_y: ray_end.y
    }
}
