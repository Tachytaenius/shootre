use bevy::prelude::*;

#[derive(Component)]
pub struct Position {
    pub value: Vec2
}

#[derive(Component)]
pub struct PreviousPosition {
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
    pub floored_recovery_time: f32 // How long in seconds from being floored (just recovered from flying, etc) to standing again
}

pub const DEFAULT_REGROUND_THRESHOLD: f32 = 110.0; // For when the component is not present
pub const DEFAULT_TRIP_THRESHOLD: f32 = 120.0;
#[derive(Component)]
pub struct FlyingThresholds {
    pub reground_threshold: f32, // Flying->Grounded under or at this speed
    pub trip_threshold: f32, // Grounded->Flying over this speed
}

#[derive(Component)]
pub struct PreviousAngle {
    pub value: f32
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
pub struct Grounded { // Not flying
    pub standing: bool, // Exempt from friction?
    pub floored_recovery_timer: Option<f32> // When this reaches 0, stand
}

pub const DEFAULT_FLOOR_FRICTION: f32 = 500.0; // For when the component is not present
#[derive(Component)]
pub struct FloorFriction {
    pub value: f32 // How much speed to remove per second when floored
}

#[derive(Component)]
pub struct Flying; // Not grounded

#[derive(Component)]
pub struct Levitates;

pub const DEFAULT_FLYING_RECOVERY_RATE: f32 = 1000.0; // For when the component is not present
#[derive(Component)]
pub struct FlyingRecoveryRate {
    pub value: f32
}

#[derive(Component)]
pub struct Gun {
    pub projectile_speed: f32,
    pub projectile_flying_recovery_rate: f32,
    pub projectile_spread: Vec2,
    pub projectile_count: u32,
    pub muzzle_distance: f32,
    pub projectile_colour: Color,
    pub cooldown: f32,
    pub auto: bool,

    pub cooldown_timer: f32
}

#[derive(Component)]
pub struct TracedLine;

#[derive(Component)]
pub struct ProjectileColour {
    pub value: Color
}

#[derive(Component)]
pub struct SpawnedMidTick {
    pub when: f32 // From 0 to 1
}

#[derive(Component)]
pub struct Collider {
    pub radius: f32
}

#[derive(Component)]
pub struct Mass {
    pub value: f32
}

#[derive(Component)]
pub struct Restitution {
    pub value: f32
}
