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
pub struct Levitates;

#[derive(Component)]
pub struct FlyingRecoveryRate {
    pub value: f32
}
