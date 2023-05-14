use bevy::prelude::*;
use crate::components::*;
use crate::events::*;

use super::gore::get_blood_transfer;
use super::gore::spawn_blood_globules;

const GLOBULE_VELOCITY_VARIATION_MULTIPLIER: f32 = 0.2;
const GLOBULE_SPEED: f32 = 100.0;
const WOUND_BLOOD_LOSS_MAXIMUM: f32 = 50.0;

pub fn process_hits (
	mut commands: Commands,
	mut query: Query<(
		Entity,
		&Hits,
		&mut Velocity,
		Option<&Mass>,
		Option<&GibForceThreshold>,
		Option<&mut ContainedBlood>,
		Option<&mut Health>
	)>,
	mut die_event_writer: EventWriter<Death>,
	mut gib_event_writer: EventWriter<Gibbing>
) {
	for (
		entity,
		hits,
		mut velocity,
		mass_option,
		gib_force_threshold_option,
		mut contained_blood_option,
		mut health_option
	) in query.iter_mut() {
		let mut to_die = false;
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

			// Take damage from hit
			if let Some(mut health_component) = health_option.as_deref_mut() {
				health_component.current -= hit.damage;
			}

			// Lose blood from hit
			if hit.blood_loss > 0.0 && contained_blood_option.is_some() {
				let contained_blood = contained_blood_option.as_deref_mut().unwrap();
				let blood_transfer = get_blood_transfer(
					contained_blood.amount,
					contained_blood.minimum_amount,
					hit.blood_loss
				).min(WOUND_BLOOD_LOSS_MAXIMUM);
				contained_blood.amount -= blood_transfer;
				let globule_velocity = velocity.value - hit.force.normalize_or_zero() * GLOBULE_SPEED;
				spawn_blood_globules(
					&mut commands,
					3,
					globule_velocity.length() * GLOBULE_VELOCITY_VARIATION_MULTIPLIER,
					blood_transfer,
					contained_blood.colour,
					hit.entry_point,
					globule_velocity
				)
			}
		}

		if to_gib {
			to_die = true;
			gib_event_writer.send(Gibbing {
				entity: entity,
				total_impact: gib_total_impact
			});
		}
		if to_die {
			die_event_writer.send(Death {entity: entity});
		}
	}
}

pub fn check_health(
	mut die_event_writer: EventWriter<Death>,
	query: Query<(Entity, &Health), Without<Dead>>
) {
	for (entity, health) in query.iter() {
		if health.current <= 0.0 {
			die_event_writer.send(Death {entity: entity});
		}
	}
}

pub fn dying(
	mut commands: Commands,
	mut die_events: EventReader<Death>,
	mut drop_event_writer: EventWriter<Dropping>,
	mut grounded_query: Query<&mut Grounded>,
	children_query: Query<&Children>,
	child_query: Query<&HoldingInfo>
) {
	for event in die_events.iter() {
		let mut entity_commands = commands.entity(event.entity);
		entity_commands.remove::<Alive>();
		entity_commands.insert(Dead);
		if let Ok(mut grounded) = grounded_query.get_mut(event.entity) {
			// This is also done in physics::manage_flooreds
			grounded.standing = false;
			grounded.floored_recovery_timer = None;
		}
		if let Ok(children) = children_query.get(event.entity) {
			for child_entity in children {
				if let Ok(_) = child_query.get(*child_entity) {
					drop_event_writer.send(Dropping {entity: *child_entity});
				}
			}
		}
	}
}
