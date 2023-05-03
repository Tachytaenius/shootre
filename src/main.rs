// This project may have TODOs in it

mod components;
mod systems;
mod util;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use extol_sprite_layer::SpriteLayerPlugin;
use systems::*;
use components::*;

fn main() {
    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    enum PreUpdateSet {Main, CommandFlush}

    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    struct MainSet;

    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    struct ConsistentStateChecks;

    #[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
    enum RenderPreparationSet {CommandFlush, Main}

    let mut app = App::new();

    app // TODO: Work out deterministic-but-still-parallelised system order
        .add_plugins(
            DefaultPlugins
            .set(ImagePlugin::default_nearest())
        )
        .add_plugin(ShapePlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(SpriteLayerPlugin::<DisplayLayer>::default())
        .insert_resource(ClearColor(Color::BLACK))

        .add_startup_system(startup::spawn_camera)
        .add_startup_system(startup::spawn_player)
        .add_startup_system(startup::spawn_other)
        .add_startup_system(startup::spawn_dots)
        .add_startup_system(startup::spawn_tilemap)

        .add_systems((
            pre_update::store_previous_position,
            pre_update::store_previous_angle,
            pre_update::store_previous_trigger_depressed,
            pre_update::remove_spawned_mid_tick,
            pre_update::clear_wills
        ).in_set(PreUpdateSet::Main).before(PreUpdateSet::CommandFlush))
        .add_system(apply_system_buffers.in_set(PreUpdateSet::CommandFlush).before(MainSet))

        .add_systems((
            wills::player_input.before(hierarchy::dropping),
            // wills:ai.before(dropping),
            hierarchy::dropping.before(hierarchy::picking_up),
            hierarchy::picking_up.before(locomotion::turning).before(locomotion::walking),
            locomotion::walking.before(guns::guns),
            locomotion::turning.before(guns::guns),
            guns::guns.before(physics::collision)
        ).in_set(MainSet)) // Set tuple size limit...
        .add_systems((
            physics::collision.before(physics::apply_velocity).before(physics::apply_angular_velocity),
            physics::apply_velocity.before(physics::manage_flyers).before(physics::tripping),
            physics::apply_angular_velocity.before(gore::gibbing),
            gore::gibbing.before(gore::blood_loss),
            gore::blood_loss.before(physics::manage_flyers),
            physics::manage_flyers.before(physics::manage_flooreds),
            physics::manage_flooreds.before(physics::floor_friction).before(physics::angular_friction), // This comes before floor_friction so that friction can be skipped in case the timer starts at zero
            physics::angular_friction,
            physics::floor_friction.before(physics::tripping),
            physics::tripping
        ).in_set(MainSet).before(RenderPreparationSet::CommandFlush));

    #[cfg(debug_assertions)]
    app.add_systems((
        physics::check_consistent_grounded_flying_state,
        hierarchy::check_consistent_hierarchy_state
    ).in_set(ConsistentStateChecks).after(MainSet).before(RenderPreparationSet::CommandFlush));

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
