use bevy::prelude::*;
use rand::*;
use std::f32::consts::TAU;

pub fn circle(rng: &mut rand::rngs::ThreadRng, radius: f32) -> Vec2 {
    let r = (rng.gen_range(0.0..1.0) as f32).powf(0.5) * radius;
    let theta = rng.gen_range(0.0..TAU);
    return Vec2::new(theta.cos() * r, theta.sin() * r);
}
