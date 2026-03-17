use std::f32::consts::TAU;

use avian2d::{
    PhysicsPlugins,
    prelude::{
        Collider, CollisionEventsEnabled, CollisionLayers, CollisionStart, Gravity, LayerMask,
        LinearVelocity, PhysicsLayer, Restitution, RigidBody,
    },
};
use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::AssetServer,
    camera::Camera2d,
    color::palettes::css::{BLACK, WHITE},
    ecs::{
        message::MessageReader,
        query::With,
        system::{Commands, Query, Res, Single},
    },
    math::Vec2,
    sprite::Sprite,
    transform::components::Transform,
    window::{PrimaryWindow, Window},
};
use rand::RngExt;

const LIST: [&str; 3] = ["rock.png", "paper.png", "scissors.png"];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity::ZERO)
        .add_systems(Startup, setup)
        .add_systems(Startup, set_wall)
        .add_systems(Update, (collision_event, enforce_speed))
        .run();
}

#[derive(Default, Debug, PartialEq, Clone, Copy, PhysicsLayer)]
enum Layer {
    #[default]
    Wall = 4,
    Type1 = 0, //rock
    Type2 = 1, //paper
    Type3 = 2, //scissor
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let mut rng = rand::rng();

    let num = 50;
    for _ in 1..num {
        let x = rng.random_range(-300.0..300.0);
        let y = rng.random_range(-300.0..300.0);

        let angle = rng.random_range(0.0..TAU);
        let speed = 500.0;
        commands.spawn((
            Sprite {
                image: asset_server.load(LIST[0]),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Collider::rectangle(30.0, 30.0),
            LinearVelocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
            Restitution::new(1.0),
            Transform::from_xyz(x, y, 0.0),
            CollisionLayers::new(Layer::Type1, [Layer::Type2, Layer::Type3, Layer::Wall]),
            CollisionEventsEnabled,
        ));
    }

    for _ in 1..num {
        let x = rng.random_range(-300.0..300.0);
        let y = rng.random_range(-300.0..300.0);

        let angle = rng.random_range(0.0..TAU);
        let speed = 500.0;
        commands.spawn((
            Sprite {
                image: asset_server.load(LIST[1]),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Collider::rectangle(30.0, 30.0),
            LinearVelocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
            Restitution::new(1.0),
            Transform::from_xyz(x, y, 0.0),
            CollisionLayers::new(Layer::Type2, [Layer::Type1, Layer::Type3, Layer::Wall]),
            CollisionEventsEnabled,
        ));
    }

    for _ in 1..num {
        let x = rng.random_range(-300.0..300.0);
        let y = rng.random_range(-300.0..300.0);

        let angle = rng.random_range(0.0..TAU);
        let speed = 500.0;
        commands.spawn((
            Sprite {
                image: asset_server.load(LIST[2]),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..Default::default()
            },
            RigidBody::Dynamic,
            Collider::rectangle(30.0, 30.0),
            LinearVelocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
            Restitution::new(1.0),
            Transform::from_xyz(x, y, 0.0),
            CollisionLayers::new(Layer::Type3, [Layer::Type1, Layer::Type2, Layer::Wall]),
            CollisionEventsEnabled,
        ));
    }
}

fn set_wall(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
    let width = window.width();
    let height = window.height();

    let min_size = width.min(height);

    commands.spawn(Sprite {
        color: BLACK.into(),
        custom_size: Some(Vec2::new(min_size, min_size)),
        ..Default::default()
    });

    commands.spawn((
        Sprite {
            color: WHITE.into(),
            custom_size: Some(Vec2::new(16.0, min_size)),
            ..Default::default()
        },
        Transform::from_xyz(min_size / 2.0, 0.0, 0.0),
        Collider::rectangle(16.0, min_size),
        RigidBody::Static,
        Restitution::new(1.0),
        CollisionLayers::new(Layer::Wall, [Layer::Type1, Layer::Type2, Layer::Type3]),
    ));
    commands.spawn((
        Sprite {
            color: WHITE.into(),
            custom_size: Some(Vec2::new(16.0, min_size)),
            ..Default::default()
        },
        Transform::from_xyz(-min_size / 2.0, 0.0, 0.0),
        Collider::rectangle(16.0, min_size),
        RigidBody::Static,
        Restitution::new(1.0),
        CollisionLayers::new(Layer::Wall, [Layer::Type1, Layer::Type2, Layer::Type3]),
    ));
    commands.spawn((
        Sprite {
            color: WHITE.into(),
            custom_size: Some(Vec2::new(min_size, 16.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, min_size / 2.0, 0.0),
        Collider::rectangle(min_size, 16.0),
        RigidBody::Static,
        Restitution::new(1.0),
        CollisionLayers::new(Layer::Wall, [Layer::Type1, Layer::Type2, Layer::Type3]),
    ));
    commands.spawn((
        Sprite {
            color: WHITE.into(),
            custom_size: Some(Vec2::new(min_size, 16.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, -min_size / 2.0, 0.0),
        Collider::rectangle(min_size, 16.0),
        RigidBody::Static,
        Restitution::new(1.0),
        CollisionLayers::new(Layer::Wall, [Layer::Type1, Layer::Type2, Layer::Type3]),
    ));
}

fn collision_event(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collison: MessageReader<CollisionStart>,
    mut q_sprite: Query<&mut Sprite>,
    q_layer: Query<&CollisionLayers>,
) {
    for event in collison.read() {
        let e1_layer_mask = q_layer.get(event.collider1).unwrap().memberships;
        let e2_layer_mask = q_layer.get(event.collider2).unwrap().memberships;

        let e1_layer = get_layer(e1_layer_mask);
        let e2_layer = get_layer(e2_layer_mask);
        //println!("{:?} {:?}", e1_layer, e2_layer);

        if e1_layer == Layer::Wall || e2_layer == Layer::Wall {
            continue;
        } else if (e1_layer == Layer::Type1 && e2_layer == Layer::Type3)
            || (e1_layer == Layer::Type2 && e2_layer == Layer::Type1)
            || (e1_layer == Layer::Type3 && e2_layer == Layer::Type2)
        {
            if let Ok(mut sprite) = q_sprite.get_mut(event.collider2) {
                sprite.image = asset_server.load(LIST[e1_layer as usize]);
            }
            if let Ok(mut c) = commands.get_entity(event.collider2) {
                let masks = match e1_layer.clone() {
                    Layer::Type1 => [Layer::Type2, Layer::Type3, Layer::Wall],
                    Layer::Type2 => [Layer::Type1, Layer::Type3, Layer::Wall],
                    Layer::Type3 => [Layer::Type1, Layer::Type2, Layer::Wall],
                    Layer::Wall => [Layer::Type1, Layer::Type2, Layer::Type3],
                };

                c.insert(CollisionLayers::new(e1_layer, masks));
            }
        } else {
            if let Ok(mut sprite) = q_sprite.get_mut(event.collider1) {
                sprite.image = asset_server.load(LIST[e2_layer as usize]);
            }

            if let Ok(mut c) = commands.get_entity(event.collider1) {
                let masks = match e2_layer.clone() {
                    Layer::Type1 => [Layer::Type2, Layer::Type3, Layer::Wall],
                    Layer::Type2 => [Layer::Type1, Layer::Type3, Layer::Wall],
                    Layer::Type3 => [Layer::Type1, Layer::Type2, Layer::Wall],
                    Layer::Wall => [Layer::Type1, Layer::Type2, Layer::Type3],
                };

                c.insert(CollisionLayers::new(e2_layer, masks));
            }
        }
    }
}

fn get_layer(layer_mask: LayerMask) -> Layer {
    if (layer_mask & Layer::Type1) != 0 {
        Layer::Type1
    } else if (layer_mask & Layer::Type2) != 0 {
        Layer::Type2
    } else if (layer_mask & Layer::Type3) != 0 {
        Layer::Type3
    } else {
        Layer::Wall
    }
}

fn enforce_speed(mut query: Query<&mut LinearVelocity>) {
    let target_speed = 200.0;
    for mut velocity in query.iter_mut() {
        if velocity.length() > 0.0 {
            velocity.0 = velocity.normalize() * target_speed;
        }
    }
}
