use crate::components::*;
use crate::util::*;
use std::f32::consts::TAU;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::tiles::TileTextureIndex;

pub fn spawn_camera (mut commands: Commands) {
    commands.spawn(
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: 1.0 / crate::graphics::SCALE,
                ..default()
            },
            ..default()
        }
    );
}

pub fn spawn_player(
    mut commands: Commands
) {
    let position = Vec2::ZERO;
    let angle = 0.0;
    commands.spawn((
        ( // Nested to get around bundle size limit
            Position {value: position},
            PreviousPosition {value: position},
            Velocity {value: Vec2::ZERO},
            Gait {
                standing_max_speed: 200.0,
                standing_acceleration: 800.0,
                floored_max_speed: 100.0,
                floored_acceleration: 400.0,
                floored_recovery_time: 2.0
            },
            FlyingRecoveryRate {value: 800.0},
            RegroundThreshold {value: 210.0},
            TripThreshold {value: 220.0}
        ),
        (
            Angle {value: angle},
            PreviousAngle {value: angle},
            AngularVelocity {value: 0.0},
            AngularGait {
                max_speed: TAU / 2.0,
                acceleration: TAU * 8.0
            },
        ),
        (
            Collider {
                radius: 10.0,
                solid: true
            },
            Mass {value: 100.0},
            Restitution {value: 0.2},
            FloorFriction {value: 300.0}
        ),
        (
            ShapeBundle {
                // Path is created by rebuild_collider_shape before rendering
                ..default()
            },
            Fill::color(Color::WHITE),
            Stroke::new(Color::WHITE, 1.0),
            DisplayLayer {
                index: DisplayLayerIndex::Actors,
                flying: false
            }
        ),
        Player,
        (
            Alive,
            Will {..default()},
            Grounded {
                standing: true,
                floored_recovery_timer: None
            },
            Health {
                maximum: 1.0,
                current: 1.0
            }
        ),
        (
            ContainedBlood {
                drip_time: 0.1,
                drip_time_minimum_multiplier: 0.75,
                smear_drip_time_multiplier: 0.3,
                colour: Color::RED,
                minimum_amount: 100.0,
                death_threshold: Some(500.0),

                leak_rate: 0.0,
                amount: 1000.0,
                drip_timer: 0.5,
                amount_to_drip: 0.0
            },
            Hits {value: Vec::<Hit>::new()},
            Gibbable,
            GibForceThreshold {value: 400000.0},
            HitForceThreshold {value: 40000.0}
        ),
        Holder {pick_up_range: 20.0}
    ));
}

pub fn spawn_other(
    mut commands: Commands
) {
    // Shotgun
    let position = Vec2::new(100.0, 0.0);
    commands.spawn((
        (
            Position {value: position},
            PreviousPosition {value: position},
            Velocity {value: Vec2::ZERO}
        ),
        (
            Collider {
                radius: 5.0,
                solid: false
            },
            Mass {value: 10.0},
            Restitution {value: 0.4},
            FloorFriction {value: 200.0}
        ),
        (
            ShapeBundle {
                ..default()
            },
            Fill::color(Color::GRAY),
            Stroke::new(Color::GRAY, 1.0),
            DisplayLayer {
                index: DisplayLayerIndex::Items,
                flying: false
            }
        ),
        Grounded {
            standing: false,
            floored_recovery_timer: None
        },
        Gun {
            projectile_speed: 6000.0,
            projectile_flying_recovery_rate: 500.0,
            projectile_spread: Vec2::new(0.05, 0.05),
            projectile_count: 10,
            projectile_colour: Color::YELLOW,
            projectile_mass: 0.01,
            projectile_base_damage_per_unit: 1.0 / 600.0 / (10.0 - 3.0),
            muzzle_distance: 5.0,
            cooldown: 1.0,
            auto: false,

            cooldown_timer: 0.0,
            trigger_depressed: false,
            trigger_depressed_previous_frame: false
        },
        Holdable
    ));

    // Machine gun
    let position = Vec2::new(100.0, 100.0);
    commands.spawn((
        (
            Position {value: position},
            PreviousPosition {value: position},
            Velocity {value: Vec2::ZERO}
        ),
        (
            Collider {
                radius: 7.0,
                solid: false
            },
            Mass {value: 20.0},
            Restitution {value: 0.3},
            FloorFriction {value: 300.0}
        ),
        (
            ShapeBundle {
                ..default()
            },
            Fill::color(Color::GRAY),
            Stroke::new(Color::GRAY, 1.0),
            DisplayLayer {
                index: DisplayLayerIndex::Items,
                flying: false
            }
        ),
        Grounded {
            standing: false,
            floored_recovery_timer: None
        },
        Gun {
            projectile_speed: 8000.0,
            projectile_flying_recovery_rate: 250.0,
            projectile_spread: Vec2::new(0.005, 0.005),
            projectile_count: 1,
            projectile_colour: Color::YELLOW,
            projectile_mass: 0.025,
            projectile_base_damage_per_unit: 1.0 / 1000.0,
            muzzle_distance: 7.0,
            cooldown: 0.1,
            auto: true,
    
            cooldown_timer: 0.0,
            trigger_depressed: false,
            trigger_depressed_previous_frame: false
        },
        Holdable
    ));

    // Ship cannon
    let position = Vec2::new(100.0, 200.0);
    commands.spawn((
        (
            Position {value: position},
            PreviousPosition {value: position},
            Velocity {value: Vec2::ZERO}
        ),
        (
            Collider {
                radius: 15.0,
                solid: false
            },
            Mass {value: 100.0},
            Restitution {value: 0.3},
            FloorFriction {value: 400.0}
        ),
        (
            ShapeBundle {
                ..default()
            },
            Fill::color(Color::GRAY),
            Stroke::new(Color::GRAY, 1.0),
            DisplayLayer {
                index: DisplayLayerIndex::Items,
                flying: false
            }
        ),
        Grounded {
            standing: false,
            floored_recovery_timer: None
        },
        Gun {
            projectile_speed: 20000.0,
            projectile_flying_recovery_rate: 250.0,
            projectile_spread: Vec2::new(0.0, 0.0),
            projectile_count: 1,
            projectile_colour: Color::YELLOW,
            projectile_mass: 30.0,
            projectile_base_damage_per_unit: 1.0 / 100.0,
            muzzle_distance: 15.0,
            cooldown: 2.0,
            auto: false,
    
            cooldown_timer: 0.0,
            trigger_depressed: false,
            trigger_depressed_previous_frame: false
        },
        Holdable
    ));

    // Giant mass to gib with
    // commands.spawn((
    //     Position {value: Vec2::new(-100.0, 10000.0)},
    //     Velocity {value: Vec2::new(0.0, -4000.0)},
    //     Collider {
    //         radius: 100.0,
    //         solid: true
    //     },
    //     Mass {value: 1000.0},
    //     Restitution {value: 1.0},
    //     ShapeBundle {
    //         // Path is created by rebuild_collider_shape before rendering
    //         ..default()
    //     },
    //     Fill::color(Color::WHITE),
    //     Stroke::new(Color::WHITE, 1.0),
    //     DisplayLayer {
    //         index: DisplayLayerIndex::Actors,
    //         flying: false
    //     }
    // ));

    let position = Vec2::new(-100.0, 100.0);
    let angle = 0.0;
    commands.spawn((
        ( // Nested to get around bundle size limit
            Position {value: position},
            PreviousPosition {value: position},
            Velocity {value: Vec2::ZERO},
            Gait {
                standing_max_speed: 200.0,
                standing_acceleration: 800.0,
                floored_max_speed: 100.0,
                floored_acceleration: 400.0,
                floored_recovery_time: 2.0
            },
            FlyingRecoveryRate {value: 800.0},
            RegroundThreshold {value: 210.0},
            TripThreshold {value: 220.0}
        ),
        (
            Angle {value: angle},
            PreviousAngle {value: angle},
            AngularVelocity {value: 0.0},
            AngularGait {
                max_speed: TAU / 2.0,
                acceleration: TAU * 8.0
            },
        ),
        (
            Collider {
                radius: 10.0,
                solid: true
            },
            Mass {value: 100.0},
            Restitution {value: 0.2},
            FloorFriction {value: 300.0}
        ),
        (
            ShapeBundle {
                // Path is created by rebuild_collider_shape before rendering
                ..default()
            },
            Fill::color(Color::WHITE),
            Stroke::new(Color::WHITE, 1.0),
            DisplayLayer {
                index: DisplayLayerIndex::Actors,
                flying: false
            }
        ),
        (
            Alive,
            Will {..default()},
            Health {
                maximum: 1.0,
                current: 1.0
            }
        ),
        Grounded {
            standing: true,
            floored_recovery_timer: None
        },
        (
            ContainedBlood {
                drip_time: 0.1,
                drip_time_minimum_multiplier: 0.75,
                smear_drip_time_multiplier: 0.3,
                colour: Color::RED,
                minimum_amount: 100.0,
                death_threshold: Some(500.0),

                leak_rate: 0.0,
                amount: 1000.0,
                drip_timer: 0.5,
                amount_to_drip: 0.0
            },
            Hits {value: Vec::<Hit>::new()},
            Gibbable,
            GibForceThreshold {value: 400000.0},
            HitForceThreshold {value: 4000.0}
        ),
        Holder {pick_up_range: 20.0}
    ));
}

pub fn spawn_dots(
    mut commands: Commands
) {
    let shape = shapes::Circle {
        radius: 2.0,
        ..default()
    };
    let mut rng = rand::thread_rng();
    for _ in 0..0 {
        commands.spawn((
            Position {value: random_in_shape::circle(&mut rng, 1000.0)},
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Fill::color(Color::NONE),
            Stroke::new(Color::WHITE, 1.0)
        ));
    }
}

pub fn spawn_tilemaps(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    array_texture_loader: Res<ArrayTextureLoader>
) {
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let map_size = TilemapSize {x: 20, y: 20};

    let main_tilemap_entity = commands.spawn((
        DisplayLayer {
            index: DisplayLayerIndex::TilemapFloors,
            flying: false
        },
        UpdateTransforms // So that the z updates at least once
    )).id();
    let mut main_tile_storage = TileStorage::empty(map_size);

    let wall_tilemap_entity = commands.spawn((
        DisplayLayer {
            index: DisplayLayerIndex::TilemapWalls,
            flying:false
        },
        UpdateTransforms 
    )).id();
    let mut wall_tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_position = TilePos {x, y};

            let tile = if
                x == 0 || x == map_size.x - 1 ||
                y == 0 || y == map_size.y - 1
            {2} else {1};
            let wall = tile == 2;

            let main_tile_entity = commands.spawn(TileBundle {
                position: tile_position,
                tilemap_id: TilemapId(main_tilemap_entity),
                texture_index: TileTextureIndex(tile),
                ..Default::default()
            }).id();
            main_tile_storage.set(&tile_position, main_tile_entity);

            let wall_tile_entity = commands.spawn(TileBundle {
                position: tile_position,
                tilemap_id: TilemapId(wall_tilemap_entity),
                texture_index: TileTextureIndex(if wall {tile} else {0}),
                ..Default::default()
            }).id();
            wall_tile_storage.set(&tile_position, wall_tile_entity);
        }
    }

    let tile_size = TilemapTileSize {x: 8.0, y: 8.0};
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();
    
    commands.entity(main_tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: main_tile_storage,
        texture: TilemapTexture::Single(texture_handle.clone()),
        tile_size,
        ..Default::default()
    });

    commands.entity(wall_tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: wall_tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        ..Default::default()
    });

    array_texture_loader.add(TilemapArrayTexture {
        texture: TilemapTexture::Single(asset_server.load("tiles.png")),
        tile_size,
        ..Default::default()
    });
}
