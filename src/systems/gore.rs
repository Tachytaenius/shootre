use crate::components::*;
use crate::util::*;
use crate::events::*;

use rand::prelude::*;
use std::f32::consts::TAU;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

fn radius_to_area(radius: f32) -> f32 {
	TAU / 2.0 * radius.powf(2.0)
}

fn area_to_radius(area: f32) -> f32 {
	(area / (TAU / 2.0)).sqrt()
}

const GIB_VELOCITY_VARIATION_MULTIPLIER: f32 = 1.05;
const GIBS_PER_GIB_FORCE_THRESHOLD_IN_GIB_TOTAL_IMPACT: f32 = 400.0;
const MAX_GIBS_PER_GIBBING: u32 = 100;
const GIB_LEAK_RATE_MULTIPLIER: f32 = 0.01; // Multiplied with blood amount, not leak rate

pub fn gibbing(
	mut commands: Commands,
	mut gib_events: EventReader<Gibbing>,
	query: Query<(
		&GibForceThreshold,
		&Collider,
		Option<&ContainedBlood>,
		&Position,
		&Velocity,
		Option<&Mass>,
		Option<&Restitution>
	)>
) {
	for event in gib_events.iter() {
		let (
			gib_force_threshold,
			collider,
			contained_blood_option,
			position,
			velocity,
			mass_option,
			restitution_option
		) = query.get(event.entity).unwrap();
		let gib_count = (( // Nasty calculation
			(event.total_impact / gib_force_threshold.value - 1.0) * GIBS_PER_GIB_FORCE_THRESHOLD_IN_GIB_TOTAL_IMPACT
		) as u32).min(MAX_GIBS_PER_GIBBING) + 2; // Without + 2 it could be 0 or 1
		let (blood_amount, blood_colour) = if let Some(contained_blood) = contained_blood_option {
			(contained_blood.amount, contained_blood.colour)
		} else {
			(0.0, Color::NONE)
		};
		gib(
			&mut commands,
			event.entity,
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

pub fn gib( // Not a system
	commands: &mut Commands,
	entity_to_gib: Entity,
	gib_count: u32,
	gib_velocity_variation: f32,
	radius: f32,
	blood_amount: f32,
	blood_colour: Color,
	position: Vec2,
	velocity: Vec2,
	mass_option: Option<f32>,
	restitution_option: Option<f32>
) {
	// TODO: Factor in angular velocity
	let mut rng = rand::thread_rng();
	commands.entity(entity_to_gib).despawn();
	for _ in 0..gib_count {
		let gib_velocity = velocity + random_in_shape::circle(&mut rng, gib_velocity_variation);
		// Based on reground threshold, flying or floored is then added
		let drip_time = 0.05;
		let gib = commands.spawn((
			DisplayLayer {
                index: DisplayLayerIndex::Gibs,
                flying: false
            },
			Position {value: position},
			PreviousPosition {value: position},
			Velocity {value: gib_velocity},
			Angle {value: rng.gen_range(0.0..TAU)}, // TODO
			AngularVelocity {value: 0.0}, // TODO
			Collider {
				radius: area_to_radius(radius_to_area(radius) / gib_count as f32),
				solid: false
			},
			ContainedBlood {
				drip_time: drip_time,
				drip_time_minimum_multiplier: 0.0, // At 0 so that massive gore explosions have more continuous blood drips near the origin
				smear_drip_time_multiplier: 0.3,
				colour: blood_colour,
				minimum_amount: 0.0,

				leak_rate: blood_amount * GIB_LEAK_RATE_MULTIPLIER,
				amount: blood_amount / gib_count as f32,
				drip_timer: 0.0,
				amount_to_drip: drip_time // For the initial drip, act like the drip time was multiplied by 1, not something lower
			},
			Gib,
			ShapeBundle {
                ..default()
            },
            Fill::color(Color::WHITE),
            Stroke::new(Color::WHITE, 1.0),
			FlyingRecoveryRate {value: 750.0}
		)).id();
		if gib_velocity.length() <= DEFAULT_REGROUND_THRESHOLD {
			commands.entity(gib).insert(Grounded {
				standing: false,
				floored_recovery_timer: None
			});
		} else {
			commands.entity(gib).insert(Flying);
		}
		if let Some(mass) = mass_option {
			commands.entity(gib).insert(Mass {value: mass / gib_count as f32});
		}
		if let Some(restitution) = restitution_option {
			commands.entity(gib).insert(Mass {value: restitution});
		}
	}
}

const GLOBULE_REGROUND_THRESHOLD: f32 = 0.0;
const GLOBULE_TRIP_THRESHOLD: f32 = 1.0;
const GLOBULE_LEAK_RATE: f32 = 0.5;

pub fn spawn_blood_globules( // Not a system
	commands: &mut Commands,
	globule_count: u32,
	globule_velocity_variation: f32,
	blood_amount: f32,
	blood_colour: Color,
	position: Vec2,
	velocity: Vec2
) {
	let mut rng = rand::thread_rng();
	for _ in 0..globule_count {
		let globule_velocity = velocity + random_in_shape::circle(&mut rng, globule_velocity_variation);
		let drip_time = 0.01;
		let mut globule_commands = commands.spawn((
			DisplayLayer {
				index: DisplayLayerIndex::BloodGlobules,
				flying: false
			},
			Position {value: position},
			PreviousPosition {value: position},
			Velocity {value: globule_velocity},
			Collider {
				radius: area_to_radius(blood_amount / globule_count as f32),
				solid: false
			},
			ContainedBlood {
				drip_time: drip_time,
				drip_time_minimum_multiplier: 0.0,
				smear_drip_time_multiplier: 0.1,
				colour: blood_colour,
				minimum_amount: 0.0,

				leak_rate: GLOBULE_LEAK_RATE,
				amount: blood_amount / globule_count as f32,
				drip_timer: 0.0,
				amount_to_drip: drip_time
			},
			BloodGlobule,
			ShapeBundle {
                ..default()
            },
            Fill::color(blood_colour),
            Stroke::new(blood_colour, 1.0),
			FlyingRecoveryRate {value: 750.0},
			RegroundThreshold {value: GLOBULE_REGROUND_THRESHOLD},
			TripThreshold {value: GLOBULE_TRIP_THRESHOLD}
		));
		if globule_velocity.length() <= GLOBULE_REGROUND_THRESHOLD {
			globule_commands.insert(Grounded {
				standing: false,
				floored_recovery_timer: None
			});
		} else {
			globule_commands.insert(Flying);
		}
	}
}

const STATIONARY_BLOOD_POOL_CLOSENESS_THRESHOLD: f32 = 0.01; // How close a stationary gib must be to a blood pool to claim it

pub fn get_blood_transfer(blood_amount: f32, minimum_blood_amount: f32, unfiltered_transfer_amount: f32) -> f32 { // Not a system
	return unfiltered_transfer_amount.min(blood_amount).min(blood_amount - minimum_blood_amount);
}

pub fn spawn_blood_pool( // Not a system
	commands: &mut Commands,
	area: f32,
	position: Vec2,
	colour: Color
) {
	debug_assert!(area > 0.0);
	commands.spawn((
		Position {value: position},
		BloodPool {
			colour: colour,
			area: area
		},
		ShapeBundle {
			// Path is set by rebuild_blood_pool before rendering
			..default()
		},
		Fill::color(Color::NONE), // Ditto
		Stroke::new(Color::NONE, 1.0), // Ditto
		DisplayLayer {
			index: DisplayLayerIndex::BloodPools,
			flying: false
		}
	));
}

pub fn blood_loss(
	mut commands: Commands,
	mut bleeder_query: Query<(&mut ContainedBlood, &Position, Option<&PreviousPosition>, Option<&Velocity>, Option<&Grounded>)>,
	mut blood_pool_query: Query<(&mut BloodPool, &Position)>,
	time: Res<Time>
) {
	let mut rng = rand::thread_rng();
	for (mut contained_blood, position, previous_position_option, velocity_option, grounded_option) in bleeder_query.iter_mut() {
		if contained_blood.amount == 0.0 || contained_blood.leak_rate == 0.0 || contained_blood.amount <= contained_blood.minimum_amount {
			continue;
		}

		let previous_position = if let Some(previous_position_component) = previous_position_option {
			previous_position_component.value
		} else {
			position.value
		};

		let velocity = if let Some(velocity_component) = velocity_option {
			velocity_component.value
		} else {
			Vec2::ZERO
		};

		let smearing = grounded_option.is_some() && !grounded_option.unwrap().standing; // Rapid "drips"
		let pooling = velocity.length() == 0.0; // Instead of dripping, just pool on the ground

		// The timer operates regardless as to whether we're pooling or dripping
		contained_blood.drip_timer -= time.delta_seconds();
		if contained_blood.drip_timer <= 0.0 {
			// If dripping, actually do something in the world with the drip timer reaching 0
			if !pooling {
				let blood_transfer = get_blood_transfer(
					contained_blood.amount,
					contained_blood.minimum_amount,
					contained_blood.leak_rate * time.delta_seconds()
				);
				contained_blood.amount -= blood_transfer;
				spawn_blood_pool(
					&mut commands,
					blood_transfer,
					previous_position.lerp(position.value, rng.gen_range(0.0..1.0)), // Lerped so that you don't see collected circles of blood drips in extreme hit-by-a-train gibbing scenarios
					contained_blood.colour
				);
			}
			// Reset timer. This comes after dripping because amount_to_drip from previous timer reset must be used before being overwritten
			contained_blood.drip_timer = contained_blood.drip_time * rng.gen_range(contained_blood.drip_time_minimum_multiplier..=1.0); // Multiplied by random to stagger the drips
			if smearing {
				contained_blood.drip_timer *= contained_blood.smear_drip_time_multiplier;
			}
			// Set blood to drip an amount consistent with leak_time (which is in units per second) for when the timer next reaches 0
			// I believe this better approximates a perfect adherence to leak_amount with arbitrary switching between pooling and dripping the smaller the length of a tick is
			// So it's a good enough solution for this project
			contained_blood.amount_to_drip = contained_blood.leak_rate * contained_blood.drip_timer;
		}
		if pooling {
			// Look for a near-enough blood pool to leak into or create one if not present
			let blood_transfer = get_blood_transfer(
				contained_blood.amount,
				contained_blood.minimum_amount,
				contained_blood.leak_rate * time.delta_seconds()
			);
			contained_blood.amount -= blood_transfer;
			let mut found = false;
			for (mut blood_pool, blood_pool_position) in blood_pool_query.iter_mut() {
				if position.value.distance(blood_pool_position.value) <= STATIONARY_BLOOD_POOL_CLOSENESS_THRESHOLD
					&& blood_pool.colour == contained_blood.colour
				{
					found = true;
					blood_pool.area += blood_transfer;
					break;
				}
			}
			if !found {
				spawn_blood_pool(&mut commands, blood_transfer, position.value, contained_blood.colour);
			}
		}
	}
}

pub fn manage_globules(
	mut commands: Commands,
	mut query: Query<(Entity, &mut Collider, &ContainedBlood, &Position, Option<&Grounded>), With<BloodGlobule>>
) {
	for (entity, mut collider, contained_blood, position, grounded_option) in query.iter_mut() {
		if contained_blood.amount <= 0.0 {
			commands.entity(entity).despawn();
			continue;
		}
		if grounded_option.is_some() {
			commands.entity(entity).despawn();
			spawn_blood_pool(&mut commands, contained_blood.amount, position.value, contained_blood.colour);
			continue;
		}
		collider.radius = area_to_radius(contained_blood.amount);
	}
}
