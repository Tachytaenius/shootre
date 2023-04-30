use crate::components::*;
use bevy::prelude::*;

pub fn dropping(
    mut commands: Commands,
    holder_query: Query<(Entity, &Will, &Children), With<Holder>>,
    position_query: Query<&Position>,
    velocity_query: Query<&Velocity>,
    angle_query: Query<&Angle>,
    angular_velocity_query: Query<&AngularVelocity>,
    drop_as_grounded_query: Query<&RegroundThreshold, Without<Levitates>>,
    gait_query: Query<&Gait>,
    child_query: Query<&ParentRelationship, With<Parent>>
) {
    for (parent_entity, will, children) in holder_query.iter() {
        if !will.drop {
            continue;
        }

        for child_entity in children.iter() {
            let parent_relationship_type = child_query.get(*child_entity).unwrap();
            match parent_relationship_type {
                ParentRelationship::Holder {held_distance, held_angle} => {
                    let mut child_commands = commands.entity(*child_entity);
                    child_commands.remove_parent();
                    child_commands.remove::<ParentRelationship>();
                    
                    if let Ok(position) = position_query.get(parent_entity) {
                        let angle;
                        if let Ok(angle_component) = angle_query.get(parent_entity) {
                            angle = angle_component.value;
                        } else {
                            angle = 0.0;
                        }
                        child_commands.insert(Position {value: position.value + Vec2::from_angle(angle).rotate(Vec2::new(*held_distance, 0.0))});
                    }
                    if let Ok(velocity) = velocity_query.get(parent_entity) {
                        child_commands.insert(Velocity {value: velocity.value});

                        let reground_threshold;
                        if let Ok(reground_threshold_component) = drop_as_grounded_query.get(*child_entity) {
                            reground_threshold = reground_threshold_component.value;
                        } else {
                            reground_threshold = DEFAULT_REGROUND_THRESHOLD;
                        }

                        if velocity.value.length() <= reground_threshold {
                            child_commands.insert(Grounded {
                                standing: gait_query.contains(*child_entity),
                                floored_recovery_timer: None
                            });
                        } else {
                            child_commands.insert(Flying);
                        }
                    }
                    if let Ok(angle) = angle_query.get(parent_entity) {
                        child_commands.insert(Angle {value: angle.value + held_angle}); // TODO: Test that it is indeed parent_angle + held_angle
                    }
                    if let Ok(angular_velocity) = angular_velocity_query.get(parent_entity) {
                        child_commands.insert(AngularVelocity {value: angular_velocity.value});
                    }
                },
                _ => {
                    continue;
                }
            }
        }
    }
}

pub fn picking_up(
    mut commands: Commands,
    holder_query: Query<(Entity, &Will, Option<&Children>, &Position, &Holder, Option<&Collider>)>,
    pick_up_able_query: Query<(Entity, &Position), (With<Holdable>, Without<Parent>)>
) {
    for (holder_entity, will, children_option, position, holder, collider_option) in holder_query.iter() {
        if !will.pick_up {
            continue;
        }
        if let Some(children) = children_option {
            if children.len() > 0 {
                continue;
            }
        }

        for (potential_child_entity, potential_child_position) in pick_up_able_query.iter() {
            if position.value.distance(potential_child_position.value) <= holder.pick_up_range {
                // Shouldn't matter if two entities pick up the same entity on the same tick (TODO: test)
                commands.entity(holder_entity).push_children(&[potential_child_entity]);
                let mut child_commands = commands.entity(potential_child_entity);
                child_commands.insert(ParentRelationship::Holder {
                    held_distance: match collider_option {
                        Some(collider_component) => {collider_component.radius},
                        _ => 0.0
                    },
                    held_angle: 0.0
                });
                child_commands.remove::<Position>();
                child_commands.remove::<Velocity>();
                child_commands.remove::<Angle>();
                child_commands.remove::<AngularVelocity>();
                child_commands.remove::<Grounded>();
                child_commands.remove::<Flying>();
                break;
            }
        }
    }
}

#[cfg(debug_assertions)]
pub fn check_consistent_hierarchy_state(
    child_query: Query<(Entity, &Parent)>,
    holder_query: Query<&Holder>,
    holdable_query: Query<&Holdable>,
    child_type_query: Query<(Entity, &ParentRelationship)>,
    position_query: Query<&Position>,
    velocity_query: Query<&Velocity>,
    angle_query: Query<&Angle>,
    angular_velocity_query: Query<&AngularVelocity>,
    grounded_query: Query<&Grounded>,
    flying_query: Query<&Flying>,
    parents_with_children_query: Query<(With<Parent>, With<Children>)>
) {
    // Check that the set of all entities with Parent and the set of all entities with ParentRelationship is the same
    // Check that no children have spatial information components
    // Check that all held entities have Holdable and that their holders have Holder
    // Check that there are no entities with a parent and children (proper chains of entities would be a huge challenge)

    for (child_entity, parent) in child_query.iter() {
        assert!(child_type_query.contains(child_entity));

        assert!(!position_query.contains(child_entity));
        assert!(!velocity_query.contains(child_entity));
        assert!(!angle_query.contains(child_entity));
        assert!(!angular_velocity_query.contains(child_entity));
        assert!(!grounded_query.contains(child_entity));
        assert!(!flying_query.contains(child_entity));

        let (_, parent_relationship_type) = child_type_query.get(child_entity).unwrap();
        match parent_relationship_type {
            ParentRelationship::Holder {..} => {
                assert!(holdable_query.contains(child_entity));
                assert!(holder_query.contains(parent.get()));
            },
            _ => {}
        }
    }

    for (child_type_entity, _) in child_type_query.iter() {
        assert!(child_query.contains(child_type_entity));
    }

    if !parents_with_children_query.is_empty() {
        panic!();
    }
}
