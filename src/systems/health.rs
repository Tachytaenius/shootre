use bevy::prelude::*;
use crate::components::*;

pub fn process_hits (
	mut _commands: Commands,
	mut query: Query<(Entity, &Hits, &mut Velocity, Option<&Mass>)>
) {
	for (_entity, hits, mut velocity, mass_option) in query.iter_mut() {
		for hit in hits.value.iter() { //  The vector gets cleared at the beginnning of each frame
			if let Some(mass) = mass_option {
				velocity.value += hit.force / mass.value;
			}
			// TODO: do stuff with entry_wound and damage
		}
	}
}
