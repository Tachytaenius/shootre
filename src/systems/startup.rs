use crate::components::*;
use crate::util::*;
use std::f32::consts::TAU;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub fn spawn_camera (mut commands: Commands) {
    commands.spawn(
        Camera2dBundle {
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
        Will {..default()},
        Grounded {
            standing: true,
            floored_recovery_timer: None
        },
        ContainedBlood {
            leak_amount: 0.0,
            drip_time: 0.1,
            drip_time_minimum_multiplier: 0.75,
            smear_drip_time_multiplier: 0.3,
            amount: 100.0,
            colour: Color::RED,

            drip_timer: 0.5,
            amount_to_drip: 0.0
        },
        Hits {value: Vec::<Hit>::new()},
        Gibbable,
        Holder {pick_up_range: 20.0}
    ));
}

pub fn spawn_other(
    mut commands: Commands
) {
    // Shotgun
    commands.spawn((
        (
            Position {value: Vec2::new(100.0, 0.0)},
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
            projectile_speed: 2000.0,
            projectile_flying_recovery_rate: 500.0,
            projectile_spread: Vec2::new(0.05, 0.05),
            projectile_count: 25,
            projectile_colour: Color::YELLOW,
            projectile_mass: 0.001,
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
    commands.spawn((
        (
            Position {value: Vec2::new(100.0, 100.0)},
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
            projectile_speed: 2500.0,
            projectile_flying_recovery_rate: 250.0,
            projectile_spread: Vec2::new(0.005, 0.005),
            projectile_count: 1,
            projectile_colour: Color::YELLOW,
            projectile_mass: 0.025,
            muzzle_distance: 7.0,
            cooldown: 0.1,
            auto: true,
    
            cooldown_timer: 0.0,
            trigger_depressed: false,
            trigger_depressed_previous_frame: false
        },
        Holdable
    ));

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
        Will {..default()},
        Grounded {
            standing: true,
            floored_recovery_timer: None
        },
        ContainedBlood {
            leak_amount: 0.0,
            drip_time: 0.1,
            drip_time_minimum_multiplier: 0.75,
            smear_drip_time_multiplier: 0.3,
            amount: 1000.0,
            colour: Color::RED,

            drip_timer: 0.5,
            amount_to_drip: 0.0
        },
        Hits {value: Vec::<Hit>::new()},
        Gibbable,
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

pub fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    array_texture_loader: Res<ArrayTextureLoader>
) {
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let map_size = TilemapSize {x: 20, y: 20};
    let tilemap_entity = commands.spawn(
        DisplayLayer {
            index: DisplayLayerIndex::Background,
            flying: false
        }
    ).id();
    let mut tile_storage = TileStorage::empty(map_size);
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_position = TilePos {x, y};
            let tile_entity = commands.spawn(TileBundle {
                position: tile_position,
                tilemap_id: TilemapId(tilemap_entity),
                ..Default::default()
            }).id();
            tile_storage.set(&tile_position, tile_entity);
        }
    }
    let tile_size = TilemapTileSize {x: 8.0, y: 8.0};
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
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
