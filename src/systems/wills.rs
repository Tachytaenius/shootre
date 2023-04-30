use crate::components::*;
use bevy::prelude::*;

pub fn player_input(
    mut query: Query<&mut Will, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>
) {
    if let Ok(mut will) = query.get_single_mut() {
        let mut target = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::A) {
            target.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            target.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::W) {
            target.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            target.y -= 1.0;
        }
        if target != Vec2::ZERO {
            target = target.normalize();
        }
        will.target_relative_velocity_multiplier = Some(target);

        let mut target = 0.0;
        if keyboard_input.pressed(KeyCode::Comma) {
            target += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Period) {
            target -= 1.0;
        }
        will.target_angular_velocity_multiplier = Some(target);

        will.drop = keyboard_input.just_pressed(KeyCode::Q);
        will.pick_up = keyboard_input.just_pressed(KeyCode::F);

        will.depress_trigger = keyboard_input.pressed(KeyCode::Space);
    }
}
