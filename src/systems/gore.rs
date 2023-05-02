use crate::components::*;
use crate::util::*;
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

fn gib(
	commands: &mut Commands,
	entity_to_gib: Entity,
	gib_count: u32,
	gibbing_velocity: Vec2,
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
		let gib_velocity = velocity + gibbing_velocity + random_in_shape::circle(&mut rng, gib_velocity_variation);
		// Based on reground threshold, flying or floored is then added
		let gib = commands.spawn((
			DisplayLayer::Gibs,
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
				leak_amount: blood_amount / 40.0,
				drip_time: 0.05,
				drip_time_minimum_multiplier: 0.75,
				floor_smear_drip_timer_speed_multiplier: 3.0,
				drip_amount_multiplier: 0.005,
				colour: blood_colour,
				amount: blood_amount / gib_count as f32,
				
				drip_timer: 0.0
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

pub fn gibbing(
	keyboard_input: Res<Input<KeyCode>>,
	mut commands: Commands,
	gibbable_query: Query<(
		Entity,
		&Position,
		Option<&Velocity>,
		&Collider,
		Option<&ContainedBlood>,
		Option<&Mass>,
		Option<&Restitution>
	), (With<Gibbable>, Without<Gib>, Without<Player>)> // Without player is temporary
) {
	if !keyboard_input.just_pressed(KeyCode::G) { // TEMP
		return;
	}
	for (
		entity,
		position,
		velocity_option,
		collider,
		contained_blood_option,
		mass_option,
		restitution_option
	) in gibbable_query.iter() {
		let velocity;
		if let Some(velocity_component) = velocity_option {
			velocity = velocity_component.value;
		} else {
			velocity = Vec2::ZERO;
		}
		let blood_amount;
		let blood_colour;
		if let Some(contained_blood) = contained_blood_option {
			blood_amount = contained_blood.amount;
			blood_colour = contained_blood.colour;
		} else {
			blood_amount = 0.0;
			blood_colour = Color::NONE;
		}
		gib(
			&mut commands,
			entity,
			20,
			Vec2::new(0.0, 0.0),
			400.0,
			collider.radius,
			blood_amount,
			blood_colour,
			position.value,
			velocity,
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

const STATIONARY_BLOOD_POOL_CLOSENESS_THRESHOLD: f32 = 0.01; // How close a stationary gib must be to a blood pool to claim it

pub fn blood_loss(
	mut commands: Commands,
	mut bleeder_query: Query<(&mut ContainedBlood, &Position, Option<&PreviousPosition>, Option<&Velocity>, Option<&Grounded>)>,
	mut blood_pool_query: Query<(&mut BloodPool, &Position)>,
	time: Res<Time>
) {
	let mut rng = rand::thread_rng();
	for (mut contained_blood, position, previous_position_option, velocity_option, grounded_option) in bleeder_query.iter_mut() {
		if contained_blood.amount == 0.0 || contained_blood.leak_amount == 0.0 {
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

		if smearing {
			contained_blood.drip_timer -= time.delta_seconds() * contained_blood.floor_smear_drip_timer_speed_multiplier;
		} else {
			contained_blood.drip_timer -= time.delta_seconds();
		}
		if contained_blood.drip_timer <= 0.0 {
			contained_blood.drip_timer = contained_blood.drip_time * rng.gen_range(contained_blood.drip_time_minimum_multiplier..=1.0); // Multiplied like this to stagger the drips
			if !pooling { // Actually do something with the drip timer going down
				let blood_transfer = (contained_blood.drip_amount_multiplier * contained_blood.leak_amount).min(contained_blood.amount);
				contained_blood.amount -= blood_transfer;
				commands.spawn((
					Position {value: previous_position.lerp(position.value, rng.gen_range(0.0..1.0))}, // Lerped so that you don't see collected circles of blood drips in extreme hit-by-a-train gibbing scenarios
					BloodPool {
						colour: contained_blood.colour,
						area: blood_transfer
					},
					ShapeBundle {
						// Path is set by rebuild_blood_pool before rendering
						..default()
					},
					Fill::color(Color::NONE), // Ditto
					Stroke::new(Color::NONE, 1.0), // Ditto
					DisplayLayer::BloodPools
				));
			}
		}
		if pooling {
			let blood_transfer = (contained_blood.leak_amount * time.delta_seconds()).min(contained_blood.amount);
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
				commands.spawn((
					Position {value: position.value},
					BloodPool {
						colour: contained_blood.colour,
						area: blood_transfer
					},
					ShapeBundle {
						// Path is set by rebuild_blood_pool before rendering
						..default()
					},
					Fill::color(Color::NONE), // Ditto
					Stroke::new(Color::NONE, 1.0), // Ditto
					DisplayLayer::BloodPools
				)); 
			}
		}
	}
}
