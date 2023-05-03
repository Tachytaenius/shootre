use crate::components::*;
use std::f32::consts::TAU;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

pub fn follow_player(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<
        (&Position, Option<&Angle>),
        (With<Player>, Or<(Changed<Position>, Changed<Angle>, Changed<Parent>)>)
    >
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        if let Ok((player_position, player_angle_option)) = player_query.get_single() {
            let entity_angle;
            if let Some(angle) = player_angle_option {
                entity_angle = angle.value;
            } else {
                entity_angle = 0.0;
            }
            camera_transform.rotation = Quat::from_rotation_z(entity_angle - TAU / 4.0);
            let camera_position = player_position.value + Vec2::from_angle(entity_angle) * 250.0; // Project camera position forwards to move player to bottom of screen
            let z_height = camera_transform.translation.z;
            camera_transform.translation = Vec3::new(camera_position.x, camera_position.y, z_height);
        }
    }
}

pub fn update_transforms(
    mut main_query: Query<
        (&mut Transform, &DisplayLayer, Option<&Position>, Option<&Angle>, Option<&Parent>, Option<&HoldingInfo>, Option<&Flying>, Option<&Sprite>),
        Or<(Changed<Position>, Changed<Angle>, Changed<Parent>)>
    >,
    parent_query: Query<(&DisplayLayer, Option<&Flying>)>
) {
    for (mut transform, display_layer, position_option, angle_option, parent_option, holding_info_option, flying_option, sprite_option) in main_query.iter_mut() {
        if parent_option.is_some() {
            let holding_info = holding_info_option.unwrap();
            transform.translation = Vec3::new(holding_info.held_distance, 0.0, 0.0);
            transform.rotation = Quat::from_rotation_z(holding_info.held_angle);
        } else if let Some(position) = position_option {
            let angle;
            if let Some(angle_component) = angle_option {
                angle = angle_component.value;
            } else {
                angle = 0.0;
            }
            transform.translation = Vec3::new(position.value.x, position.value.y, 0.0);
            transform.rotation = Quat::from_rotation_z(angle);
        }

        // Layering for non-sprites, sprites are handled by extol_sprite_layer
        if sprite_option.is_none() {
            transform.translation.z = (*display_layer).index as u32 as f32;
            if flying_option.is_some() {
                // transform.translation.z += mem::variant_count::<DisplayLayer>();
                transform.translation.z += DisplayLayerIndex::LayerCount as u32 as f32;
            }
            transform.translation.z *= 1.1; // Blood pools don't appear on the background without a small boost to z
            // Calculate parent's z and compensate for it
            // This only works because parent-child hierarchies are limited such that no entity may have both a parent and children
            if let Some(parent) = parent_option {
                let (parent_display_layer, parent_flying_option) = parent_query.get(parent.get()).unwrap();
                let mut parent_transform_z = (*parent_display_layer).index as u32 as f32;
                if parent_flying_option.is_some() {
                    // parent_transform_z += mem::variant_count::<DisplayLayer>();
                    parent_transform_z += DisplayLayerIndex::LayerCount as u32 as f32;
                }
                parent_transform_z *= 1.1;
                transform.translation.z -= parent_transform_z;
            }
        }
    }
}

pub fn hollow_flying(mut query: Query<&mut Fill, Added<Flying>>) {
    for mut fill in query.iter_mut() {
        fill.color = Color::NONE;
    }
}

pub fn fill_grounded(mut query: Query<(&mut Fill, &Stroke), Added<Grounded>>) {
    for (mut fill, stroke) in query.iter_mut() {
        fill.color = stroke.color;
    }
}

const DRAW_TRACER_AS_POINT_THRESHOLD: f32 = 1.0;
const TRACER_POINT_CIRCLE_RADIUS: f32 = 0.1;

pub fn rebuild_traced_shape(
    mut commands: Commands,
    mut tracer_query: Query<(Entity, &mut Stroke, Option<&SpawnedMidTick>, &ProjectileColour, &Position, &PreviousPosition), (With<Path>, With<TracedLine>)>,
    player_query: Query<(&Position, &Angle, &PreviousPosition, &PreviousAngle), With<Player>>
) {
    // NOTE: It might be better if this was in terms of rotation and translation directly, and not using Transform objects.
    if let Ok((player_position, player_angle, player_previous_position, player_previous_angle)) = player_query.get_single() {
        for (entity, mut tracer_stroke, tracer_spawned_mid_tick_option, tracer_projectile_colour, tracer_position, tracer_previous_position) in tracer_query.iter_mut() {
            let previous_transform_lerp;
            if let Some(tracer_spawned_mid_tick) = tracer_spawned_mid_tick_option {
                previous_transform_lerp = tracer_spawned_mid_tick.when;
            } else {
                previous_transform_lerp = 0.0;
            }

            let player_previous_position = // Shadow
                player_previous_position.value * (1.0 - previous_transform_lerp)
                + player_position.value * previous_transform_lerp;

            let player_previous_camera_transform = Transform {
                translation: Vec3::new(player_previous_position.x, player_previous_position.y, 0.0),
                rotation: Quat::from_rotation_z(
                    player_previous_angle.value * (1.0 - previous_transform_lerp)
                    + player_angle.value * previous_transform_lerp
                ),
                ..default()
            };
            let player_current_camera_transform = Transform {
                translation: Vec3::new(player_position.value.x, player_position.value.y, 0.0),
                rotation: Quat::from_rotation_z(player_angle.value),
                ..default()
            };

            let tracer_previous_screen_space_position_4d = player_previous_camera_transform.compute_matrix().inverse() * // Inverting because camera
                Vec4::new(tracer_previous_position.value.x, tracer_previous_position.value.y, 0.0, 1.0);
            let tracer_current_screen_space_position_4d = player_current_camera_transform.compute_matrix().inverse() *
                Vec4::new(tracer_position.value.x, tracer_position.value.y, 0.0, 1.0);

            let path_vector = Vec2::from_angle(player_angle.value).rotate( // Transform out of screen space back into world space, but keeping the difference
                Vec2::new(tracer_previous_screen_space_position_4d.x, tracer_previous_screen_space_position_4d.y)
                - Vec2::new(tracer_current_screen_space_position_4d.x, tracer_current_screen_space_position_4d.y)
            );

            if path_vector.length() <= DRAW_TRACER_AS_POINT_THRESHOLD {
                let circle = shapes::Circle {
                    radius: TRACER_POINT_CIRCLE_RADIUS,
                    center: path_vector
                };
                commands.entity(entity).insert(GeometryBuilder::build_as(&circle));
            } else {
                let line = shapes::Line(Vec2::ZERO, path_vector);
                tracer_stroke.color.set_a(
                    tracer_projectile_colour.value.a()
                    * (1.0 / path_vector.length()).min(1.0)
                );
                commands.entity(entity).insert(GeometryBuilder::build_as(&line));
            }
        }
    }
}

pub fn rebuild_collider_shape(
    mut commands: Commands,
    query: Query<(Entity, &Collider), (Changed<Collider>, With<Path>)>
) {
    for (entity, collider) in query.iter() {
        let circle = shapes::Circle {
            radius: collider.radius,
            ..default()
        };
        let line = shapes::Line(
            Vec2::new(collider.radius, 0.0),
            Vec2::new(collider.radius + 5.0, 0.0)
        );
        commands.entity(entity).insert(GeometryBuilder::new().add(&circle).add(&line).build());
    }
}

fn area_to_radius(area: f32) -> f32 {
	(area / (TAU / 2.0)).sqrt()
}

pub fn rebuild_blood_pool(
    mut commands: Commands,
    mut query: Query<(Entity, &BloodPool, &mut Stroke, &mut Fill), (Changed<BloodPool>, With<Path>)>
) {
    for (entity, blood_pool, mut stroke, mut fill) in query.iter_mut() {
        let circle = shapes::Circle {
            radius: area_to_radius(blood_pool.area),
            ..default()
        };
        commands.entity(entity).insert(GeometryBuilder::build_as(&circle));
        stroke.color = blood_pool.colour;
        fill.color = blood_pool.colour;
    }
}
