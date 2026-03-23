use std::f32::consts::TAU;

use avian2d::prelude::{
    Collider, CollisionEventsEnabled, CollisionLayers, CollisionStart, LayerMask, LinearVelocity,
    Physics, PhysicsTime, Restitution, RigidBody,
};
use bevy::{
    app::{App, Startup, Update},
    asset::AssetServer,
    color::{
        Color,
        palettes::{
            css::{BLACK, WHITE},
            tailwind::GRAY_500,
        },
    },
    ecs::{
        component::Component,
        entity::ContainsEntity,
        hierarchy::Children,
        message::MessageReader,
        observer::On,
        query::With,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res, ResMut, Single},
    },
    math::{Vec2, Vec3},
    picking::events::{Click, Pointer},
    sprite::Sprite,
    state::{
        condition::in_state,
        state::{NextState, OnEnter, OnExit},
    },
    text::{FontWeight, TextColor, TextFont},
    time::Time,
    transform::components::Transform,
    ui::{
        AlignItems, BackgroundColor, BorderRadius, JustifyContent, Node, PositionType, UiRect, Val,
        widget::Text,
    },
    window::{PrimaryWindow, Window},
};
use rand::RngExt;

use crate::{
    CamerInfo, FONTPATH, LIST, Layer, SimState, custom::CustomInfo, despawn_screen,
    main_home::MainUi, move_camera::MoveInfo,
};

#[derive(Debug, Clone, Component)]
pub struct Player;
#[derive(Component)]
struct SimUi;

pub fn sim_plugin(app: &mut App) {
    app.add_systems(OnEnter(SimState::Sim), setup)
        .add_systems(
            Update,
            (collision_event, enforce_speed).run_if(in_state(SimState::Sim)),
        )
        .add_systems(Startup, (set_wall, spawn_player))
        .add_systems(OnExit(SimState::Sim), despawn_screen::<SimUi>);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Sim");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                padding: UiRect::all(Val::Px(7.5)),
                column_gap: Val::Px(7.5),
                ..Default::default()
            },
            BackgroundColor(GRAY_500.into()),
            SimUi,
        ))
        .with_children(|p| {
            let button_node = (
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(Val::Percent(15.0)),
                    ..Default::default()
                },
                BackgroundColor(WHITE.into()),
            );

            let text_basic = (
                TextFont {
                    font: asset_server.load(FONTPATH),
                    weight: FontWeight::BOLD,
                    font_size: 25.0,
                    ..Default::default()
                },
                TextColor(BLACK.into()),
            );

            p.spawn(button_node.clone())
                .with_child((
                    text_basic.clone(),
                    Text("▶".to_string()),
                    Node {
                        top: Val::Px(2.0),
                        left: Val::Px(1.0),
                        ..Default::default()
                    },
                ))
                .observe(
                    |trigger: On<Pointer<Click>>,
                     mut time: ResMut<Time<Physics>>,
                     q_children: Query<&Children>,
                     mut q_text: Query<&mut Text>| {
                        if let Ok(children) = q_children.get(trigger.entity) {
                            for child in children.iter() {
                                if let Ok(mut text) = q_text.get_mut(child.entity()) {
                                    if time.relative_speed() == 0.0 {
                                        text.0 = "❙❙".to_string();
                                        time.set_relative_speed(1.0);
                                    } else {
                                        text.0 = "▶".to_string();
                                        time.set_relative_speed(0.0);
                                    }
                                }
                            }
                        }
                    },
                );
            p.spawn(button_node.clone())
                .with_child((
                    text_basic.clone(),
                    Text("✕".to_string()),
                    Node {
                        top: Val::Px(2.0),
                        ..Default::default()
                    },
                ))
                .observe(
                    |_: On<Pointer<Click>>,
                     mut state: ResMut<NextState<SimState>>,
                     mut move_info: ResMut<MoveInfo>,
                     camera_info: Res<CamerInfo>,
                     mut time: ResMut<Time<Physics>>| {
                        state.set(SimState::Move);
                        *move_info = MoveInfo {
                            time: 0.0,
                            trans: (Vec3::new(0.0, 0.0, 0.0), Vec3::new(camera_info.x, 0.0, 0.0)),
                            scale: (1.0, camera_info.scale),
                            next: SimState::Main,
                        };
                        time.set_relative_speed(0.0);
                    },
                );
        });
}
pub fn set_wall(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
    let width = window.width();
    let height = window.height();

    let min_size = width.min(height);

    commands.spawn((
        Sprite {
            color: BLACK.into(),
            custom_size: Some(Vec2::new(min_size, min_size)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
        MainUi,
    ));

    let wall_color = Color::NONE;
    let wall_basic = (
        RigidBody::Static,
        Restitution::new(1.0),
        CollisionLayers::new(Layer::Wall, [Layer::Type1, Layer::Type2, Layer::Type3]),
    );
    commands.spawn((
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(16.0, min_size)),
            ..Default::default()
        },
        Transform::from_xyz(min_size / 2.0, 0.0, 0.0),
        Collider::rectangle(16.0, min_size),
        wall_basic.clone(),
        MainUi,
    ));
    commands.spawn((
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(16.0, min_size)),
            ..Default::default()
        },
        Transform::from_xyz(-min_size / 2.0, 0.0, 0.0),
        Collider::rectangle(16.0, min_size),
        wall_basic,
        MainUi,
    ));
    commands.spawn((
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(min_size, 16.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, min_size / 2.0, 0.0),
        Collider::rectangle(min_size, 16.0),
        wall_basic,
        MainUi,
    ));
    commands.spawn((
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(min_size, 16.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, -min_size / 2.0, 0.0),
        Collider::rectangle(min_size, 16.0),
        wall_basic,
        MainUi,
    ));
}

pub fn spawn_player(
    mut commands: Commands,
    custom_info: Res<CustomInfo>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::rng();
    let num = 50;
    let player_basic = (
        RigidBody::Dynamic,
        Collider::rectangle(30.0, 30.0),
        Restitution::new(1.0),
        CollisionEventsEnabled,
        Player,
    );
    let speed = 100.0;
    let layer_list = vec![Layer::Type1, Layer::Type2, Layer::Type3];
    for i in 1..(custom_info.nums.len() as i32 + 1) {
        let Some(handle) = custom_info.image_hash.get(&i) else {
            println!("Nope!");
            continue;
        };
        let Some(num) = custom_info.nums.get(&i) else {
            println!("Nope!");
            continue;
        };
        let mut all = LayerMask::ALL;
        all.remove(layer_list[(i - 1) as usize]);
        let player = (
            Sprite {
                image: handle.clone(),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..Default::default()
            },
            CollisionLayers::new(layer_list[(i - 1) as usize], all),
        );

        for _ in 0..num.abs() {
            let x = rng.random_range(-300.0..300.0);
            let y = rng.random_range(-300.0..300.0);

            let angle = rng.random_range(0.0..TAU);

            commands.spawn((
                LinearVelocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
                Transform::from_xyz(x, y, 0.0),
                player_basic.clone(),
                player.clone(),
            ));
        }
    }

    // let Some(handle) = custom_info.image_hash.get(&1) else {
    //     println!("Nope!");
    //     return;
    // };

    // let player1 = (
    //     Sprite {
    //         image: handle.clone(),
    //         custom_size: Some(Vec2::new(32.0, 32.0)),
    //         ..Default::default()
    //     },
    //     CollisionLayers::new(Layer::Type1, [Layer::Type2, Layer::Type3, Layer::Wall]),
    // );
    // for _ in 0..num {
    //     let x = rng.random_range(-300.0..300.0);
    //     let y = rng.random_range(-300.0..300.0);

    //     let angle = rng.random_range(0.0..TAU);

    //     commands.spawn((
    //         LinearVelocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
    //         Transform::from_xyz(x, y, 0.0),
    //         player_basic.clone(),
    //         player1.clone(),
    //     ));
    // }
    // let player2 = (
    //     Sprite {
    //         image: asset_server.load(LIST[1]),
    //         custom_size: Some(Vec2::new(32.0, 32.0)),
    //         ..Default::default()
    //     },
    //     CollisionLayers::new(Layer::Type2, [Layer::Type1, Layer::Type3, Layer::Wall]),
    // );
    // for _ in 0..num {
    //     let x = rng.random_range(-300.0..300.0);
    //     let y = rng.random_range(-300.0..300.0);

    //     let angle = rng.random_range(0.0..TAU);

    //     commands.spawn((
    //         LinearVelocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
    //         Transform::from_xyz(x, y, 0.0),
    //         player_basic.clone(),
    //         player2.clone(),
    //     ));
    // }

    // let player3 = (
    //     Sprite {
    //         image: asset_server.load(LIST[2]),
    //         custom_size: Some(Vec2::new(32.0, 32.0)),
    //         ..Default::default()
    //     },
    //     CollisionLayers::new(Layer::Type3, [Layer::Type1, Layer::Type2, Layer::Wall]),
    // );
    // for _ in 0..num {
    //     let x = rng.random_range(-300.0..300.0);
    //     let y = rng.random_range(-300.0..300.0);

    //     let angle = rng.random_range(0.0..TAU);

    //     commands.spawn((
    //         LinearVelocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
    //         Transform::from_xyz(x, y, 0.0),
    //         player_basic.clone(),
    //         player3.clone(),
    //     ));
    // }
}

fn collision_event(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collison: MessageReader<CollisionStart>,
    mut q_sprite: Query<&mut Sprite>,
    custom_info: Res<CustomInfo>,
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
                sprite.image =
                    if let Some(handle) = custom_info.image_hash.get(&(e1_layer as i32 + 1)) {
                        handle.clone()
                    } else {
                        asset_server.load(LIST[e1_layer as usize])
                    };
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
                sprite.image =
                    if let Some(handle) = custom_info.image_hash.get(&(e2_layer as i32 + 1)) {
                        handle.clone()
                    } else {
                        asset_server.load(LIST[e2_layer as usize])
                    };
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
