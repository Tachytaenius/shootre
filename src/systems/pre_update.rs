use crate::components::*;
use bevy::prelude::*;

pub fn store_previous_position(mut query: Query<(&mut PreviousPosition, &Position)>) {
    for (mut previous_position, position) in query.iter_mut() {
        previous_position.value = position.value;
    }
}

pub fn store_previous_angle(mut query: Query<(&mut PreviousAngle, &Angle)>) {
    for (mut previous_angle, angle) in query.iter_mut() {
        previous_angle.value = angle.value;
    }
}

pub fn store_previous_trigger_depressed(mut query: Query<&mut Gun>) {
    for mut gun in query.iter_mut() {
        gun.trigger_depressed_previous_frame = gun.trigger_depressed;
    }
}

pub fn remove_spawned_mid_tick(
    mut commands: Commands,
    query: Query<Entity, With<SpawnedMidTick>>
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<SpawnedMidTick>();
    }
}

pub fn clear_wills(
    mut commands: Commands,
    query: Query<Entity, With<Will>>
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<Will>();
        commands.entity(entity).insert(Will {..default()});
    }
}
