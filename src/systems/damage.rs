use bevy::prelude::*;
use crate::components::*;
use crate::systems::*;

use super::gore::get_blood_transfer;
use super::gore::spawn_blood_globules;

const GIB_VELOCITY_VARIATION_MULTIPLIER: f32 = 0.25;
const GIBS_PER_GIB_FORCE_THRESHOLD_IN_GIB_TOTAL_IMPACT: f32 = 400.0;
const MAX_GIBS_PER_GIBBING: u32 = 100;
const GLOBULE_VELOCITY_VARIATION_MULTIPLIER: f32 = 0.2;
const GLOBULE_SPEED: f32 = 100.0;

pub fn process_hits (
	mut commands: Commands,
	mut query: Query<(
		Entity,
		&Hits,
		&Position,
		&mut Velocity,
		Option<&Mass>,
		Option<&GibForceThreshold>,
		&Collider,
		Option<&mut ContainedBlood>,
		Option<&Restitution>
	)>
) {
	for (
		entity,
		hits,
		position,
		mut velocity,
		mass_option,
		gib_force_threshold_option,
		collider,
		mut contained_blood_option,
		restitution_option
	) in query.iter_mut() {
		let mut to_gib = false; // If any force is enough to cause gibbing, gib, but do it using the sum of all forces
		let mut gib_total_impact = 0.0; // Add lengths of every force to be used in getting how many gibs to create
		for hit in hits.value.iter() { //  The vector gets cleared at the beginnning of each frame
			if let Some(mass) = mass_option {
				if hit.apply_force {
					velocity.value += hit.force / mass.value;
				}
				gib_total_impact += hit.force.length();
			}
			if let Some(gib_force_threshold_component) = gib_force_threshold_option {
				if hit.force.length() >= gib_force_threshold_component.value {
					to_gib = true;
				}
			}
			// TODO: do stuff with entry_wound and damage
			if hit.blood_loss > 0.0 && contained_blood_option.is_some() {
				let contained_blood = contained_blood_option.as_mut().unwrap();
				let blood_transfer = get_blood_transfer(
					contained_blood.amount,
					contained_blood.minimum_amount,
					hit.blood_loss
				);
				contained_blood.amount -= blood_transfer;
				let globule_velocity = velocity.value - hit.force.normalize_or_zero() * GLOBULE_SPEED;
				spawn_blood_globules(
					&mut commands,
					10,
					globule_velocity.length() * GLOBULE_VELOCITY_VARIATION_MULTIPLIER,
					blood_transfer,
					contained_blood.colour,
					hit.entry_point,
					globule_velocity
				)
			}
		}
		let (blood_amount, blood_colour) = if let Some(contained_blood) = contained_blood_option {
			(contained_blood.amount, contained_blood.colour)
		} else {
			(0.0, Color::NONE)
		};
		if to_gib {
			let gib_count = (( // Nasty calculation
				(gib_total_impact / gib_force_threshold_option.unwrap().value - 1.0) * GIBS_PER_GIB_FORCE_THRESHOLD_IN_GIB_TOTAL_IMPACT
			) as u32).min(MAX_GIBS_PER_GIBBING) + 2; // Without + 2 it could be 0 or 1
			gore::gib(
				&mut commands,
				entity,
				gib_count,
				velocity.value.length() * GIB_VELOCITY_VARIATION_MULTIPLIER,
				collider.radius,
				blood_amount,
				blood_colour,
				position.value,
				velocity.value,
				match mass_option {
					Some(mass_component) => {Some(mass_component.value)},
					_ => {None}
				},
				match restitution_option {
					Some(restitution_component) => {Some(restitution_component.value)},
					_ => {None}
				},
			);
		}
	}
}
