use bevy::prelude::*;

pub struct Dropping {pub entity: Entity}

pub struct Death {pub entity: Entity}

pub struct Gibbing {
	pub entity: Entity,
	pub total_impact: f32
}
