use std::f32::consts::TAU;
use bevy::prelude::*;
use extol_sprite_layer::LayerIndex;

#[derive(Component)]
pub struct Position {pub value: Vec2}

#[derive(Component)]
pub struct PreviousPosition {pub value: Vec2}

#[derive(Component)]
pub struct Velocity {pub value: Vec2}

#[derive(Component)]
pub struct Gait {
    pub standing_max_speed: f32, // Levitators always use standing speed
    pub standing_acceleration: f32,
    pub floored_max_speed: f32,
    pub floored_acceleration: f32,
    pub floored_recovery_time: f32 // How long in seconds from being floored (just recovered from flying, etc) to standing again
}

pub const DEFAULT_REGROUND_THRESHOLD: f32 = 110.0;
#[derive(Component)]
pub struct RegroundThreshold {pub value: f32} // Flying to grounded under or at this speed

pub const DEFAULT_TRIP_THRESHOLD: f32 = 120.0;
#[derive(Component)]
pub struct TripThreshold {pub value: f32} // Grounded to flying over this speed

#[derive(Component)]
pub struct PreviousAngle {pub value: f32}

#[derive(Component)]
pub struct Angle {pub value: f32}

#[derive(Component)]
pub struct AngularVelocity {pub value: f32}

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

pub const DEFAULT_FLOORED_ANGULAR_FRICTION: f32 = TAU * 2.0;
#[derive(Component)]
pub struct FlooredAngularFriction {pub value: f32}

pub const DEFAULT_UNFLOORED_ANGULAR_FRICTION: f32 = TAU;
#[derive(Component)]
pub struct UnflooredAngularFriction {pub value: f32}

pub const DEFAULT_FLOOR_FRICTION: f32 = 200.0;
#[derive(Component)]
pub struct FloorFriction {pub value: f32} // How much speed to remove per second when floored

#[derive(Component)]
pub struct Flying; // Not grounded

#[derive(Component)]
pub struct Levitates;

pub const DEFAULT_FLYING_RECOVERY_RATE: f32 = 1000.0;
#[derive(Component)]
pub struct FlyingRecoveryRate {pub value: f32}

#[derive(Component)]
pub struct Gun {
    pub projectile_speed: f32,
    pub projectile_flying_recovery_rate: f32,
    pub projectile_spread: Vec2,
    pub projectile_count: u32,
    pub muzzle_distance: f32,
    pub projectile_colour: Color,
    pub projectile_mass: f32,
    pub projectile_base_damage_per_unit: f32,
    pub cooldown: f32,
    pub auto: bool,

    pub cooldown_timer: f32,
    pub trigger_depressed: bool,
    pub trigger_depressed_previous_frame: bool
}

#[derive(Component)]
pub struct TracedLine; // For rendering

#[derive(Component)]
pub struct GunProjectile; // For collision detection

#[derive(Component)]
pub struct ProjectileColour {pub value: Color}

#[derive(Component)]
pub struct SpawnedMidTick {pub when: f32} // From 0 to 1

#[derive(Component)]
pub struct Collider {
    pub radius: f32,
    pub solid: bool
}

#[derive(Component)]
pub struct Mass {pub value: f32}

pub const DEFAULT_RESTITUTION: f32 = 1.0;
#[derive(Component)]
pub struct Restitution {pub value: f32}

#[derive(Component, Default)]
pub struct Will {
    pub target_relative_velocity_multiplier: Option<Vec2>, // Should not lie outside the unit cirlce
    pub target_angular_velocity_multiplier: Option<f32>, // Should be -1 to 1
    pub drop: bool,
    pub pick_up: bool,
    pub depress_trigger: bool
}

#[derive(Component)]
pub struct Holder {
    pub pick_up_range: f32
}

#[derive(Component)]
pub struct Holdable;

#[derive(Component)]
pub struct HoldingInfo {
    pub held_distance: f32,
    pub held_angle: f32
}

#[derive(Component)]
pub struct ContainedBlood {
    pub drip_time: f32, // How long between drips of blood when flying. Is multiplied by a random number in the range drip_time_minimum_multiplier..=1.0
    pub drip_time_minimum_multiplier: f32, // 1.0 for evenly-spaced-in-time drip events (though the drips will not be evenly-spaced in space as they are created somewhere between previous and last position)
    pub smear_drip_time_multiplier: f32, // When floored but not stationary, how much to multiply drip time (down) by when resetting the timer
    pub colour: Color,
    pub minimum_amount: f32, // How much to not leak out, so that an entity that has bled out completely and is then gibbed doesn't have bloodless gibs

    pub leak_rate: f32,
    pub amount: f32, // Area of blood pool when fully depleted in pixels
    pub drip_timer: f32,
    pub amount_to_drip: f32 // Changes each time the drip timer is reset so that leak_rate can be in terms of units per second when dripping
}

#[derive(Component)]
pub struct BloodPool {
    pub colour: Color,
    pub area: f32
}

#[derive(Component)]
pub struct BloodGlobule;

#[derive(Component)]
pub struct Gib;

#[derive(Component)]
pub struct Gibbable;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DisplayLayer {
    pub index: DisplayLayerIndex,
    pub flying: bool
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DisplayLayerIndex {
    Background,
    BloodPools,
    BloodGlobules,
    Gibs,
    Items,
    Projectiles,
    Actors,

    LayerCount // Keep this last and remove it if mem::variant_count leaves nightly
}

impl LayerIndex for DisplayLayer {
    fn as_z_coordinate(&self) -> f32 {
        let mut z = (*self).index as u32 as f32;
        if self.flying {
            z += DisplayLayerIndex::LayerCount as u32 as f32;
        }
        z *= 1.1; // To match what happens to non-sprites
        return z;
    }
}

pub struct Hit {
    pub entry_point: Vec2, // Relative to Position
    pub force: Vec2, // Applied directly, entry_point's position has already made a difference to the force
    pub damage: f32,
    pub apply_force: bool, // False if it has already been applied by collision resolution (kinda spaghetti, other projects may have more proper systems)
    pub blood_loss: f32
}

#[derive(Component)]
pub struct Hits {
    pub value: Vec<Hit>
}

#[derive(Component)]
pub struct DestroyedButRender;

#[derive(Component)]
pub struct HitForceThreshold {pub value: f32}

#[derive(Component)]
pub struct GibForceThreshold {pub value: f32}

#[derive(Component)]
pub struct Alive;

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct Health {
    pub maximum: f32,
    pub current: f32 // Can go below zero
}

#[derive(Component)]
pub struct BaseDamagePerSpeed {pub value: f32}
