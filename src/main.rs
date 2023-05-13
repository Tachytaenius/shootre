// This project may have TODOs in it

mod components;
mod events;
mod systems;
mod util;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use extol_sprite_layer::SpriteLayerPlugin;
use systems::*;
use components::*;
use events::*;

fn main() {
    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    enum PreUpdateSet {Main, CommandFlush}

    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    struct Wills;
    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    enum LinearAngular {Locomotion, ApplyVelocity, Friction}

    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    struct ConsistentStateChecks;

    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    enum RenderPreparationSet {CommandFlush, Main}

    let mut app = App::new();

    app
        .add_plugins(
            DefaultPlugins
            .set(ImagePlugin::default_nearest())
        )
        .add_plugin(ShapePlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(SpriteLayerPlugin::<DisplayLayer>::default())

        .insert_resource(ClearColor(Color::BLACK))

        .add_event::<Dropping>()
        .add_event::<Death>()
        .add_event::<Gibbing>()

        .add_startup_systems(( // Chained for determinism
            startup::spawn_camera,
            startup::spawn_player,
            startup::spawn_other,
            startup::spawn_dots,
            startup::spawn_tilemap
        ).chain())

        .add_systems((
            pre_update::store_previous_position,
            pre_update::store_previous_angle,
            pre_update::store_previous_trigger_depressed,
            pre_update::remove_spawned_mid_tick,
            pre_update::clear_wills,
            pre_update::remove_destroyed_but_rendered_entities,
            pre_update::remove_hits
        ).in_set(PreUpdateSet::Main).before(PreUpdateSet::CommandFlush))
        .add_system(apply_system_buffers.in_set(PreUpdateSet::CommandFlush).before(Wills))


        .add_systems(( // Parallellised
            wills::player_input,
            // wills::ai
        ).in_set(Wills))

        .add_systems(( // Not paralellised
            hierarchy::send_dropping_events,
            hierarchy::send_picking_up_events
        ).chain().after(Wills).before(LinearAngular::Locomotion))

        .add_systems(( // Parallelised
            locomotion::walking,
            locomotion::turning
        ).in_set(LinearAngular::Locomotion).before(LinearAngular::ApplyVelocity))

        .add_systems(( // Parallelised
            physics::apply_velocity,
            physics::apply_angular_velocity
        ).in_set(LinearAngular::ApplyVelocity))

        // Not parallelised
        .add_system(guns::tick_guns.after(LinearAngular::ApplyVelocity))
        .add_system(apply_system_buffers.after(guns::tick_guns).before(guns::detect_hits)) // So that detect_hits sees projectiles spawned this tick, in case they're shot inside a collider
        .add_systems((
            guns::detect_hits,
            physics::collision,
            gore::blood_loss,
            gore::manage_globules
        ).chain())
        .add_system(apply_system_buffers.after(gore::manage_globules).before(physics::manage_flyers)) // So that despawned blood globules won't be acted on (panics otherwise)
        .add_systems((
            physics::manage_flyers,
            physics::manage_flooreds
        ).chain().before(LinearAngular::Friction))

        .add_systems(( // Parallelised
            physics::floor_friction,
            physics::angular_friction
        ).in_set(LinearAngular::Friction))

        .add_systems(( // Not parallelised
            physics::tripping,
            damage::process_hits,
            damage::dying,
            hierarchy::handle_dropping,
            gore::gibbing,
            guns::despawn_stationary_projectiles
        ).chain().before(ConsistentStateChecks).after(LinearAngular::Friction).before(RenderPreparationSet::CommandFlush));


    #[cfg(debug_assertions)]
    app.add_systems((
        physics::check_consistent_grounded_flying_state,
        hierarchy::check_consistent_hierarchy_state
    ).in_set(ConsistentStateChecks).before(RenderPreparationSet::CommandFlush));

    app
        .add_system(apply_system_buffers.in_set(RenderPreparationSet::CommandFlush).before(RenderPreparationSet::Main))
        .add_systems((
            graphics::hollow_flying,
            graphics::fill_grounded,
            graphics::follow_player,
            graphics::update_transforms,
            graphics::rebuild_traced_shape,
            graphics::rebuild_collider_shape,
            graphics::rebuild_blood_pool
        ).in_set(RenderPreparationSet::Main));

    app.run();
}
