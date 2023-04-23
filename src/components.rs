use bevy::prelude::*;

#[derive(Component)]
pub struct Position {
    pub value: Vec2
}

#[derive(Component)]
pub struct Velocity {
    pub value: Vec2
}

#[derive(Component)]
pub struct Gait {
    pub max_speed: f32,
    pub acceleration: f32,
    pub stand_threshold: f32,
    pub trip_threshold: f32
}

#[derive(Component)]
pub struct Angle {
    pub value: f32
}

#[derive(Component)]
pub struct AngularVelocity {
    pub value: f32
}

#[derive(Component)]
pub struct AngularGait {
    pub max_speed: f32,
    pub acceleration: f32
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Grounded;

#[derive(Component)]
pub struct Flying;

#[derive(Component)]
pub struct Levitates;

#[derive(Component)]
pub struct FlyingRecoveryRate {
    pub value: f32
}

#[derive(Component)]
pub struct Gun {
    pub projectile_speed: f32,
    pub projectile_radius: f32,
    pub projectile_spread: Vec2,
    pub projectile_count: u32,
    pub muzzle_distance: f32,
    pub cooldown: f32,
    pub auto: bool,

    pub cooldown_timer: f32
}
