use std::{collections::HashMap, f32::consts::TAU};

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
        resource::Resource,
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
use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::{
    CamerInfo, FONTPATH, LIST, SimState, custom::CustomInfo, despawn_screen, main_home::MainUi,
    move_camera::MoveInfo,
};

#[derive(Debug, Clone, Component)]
pub struct Player;
#[derive(Component)]
struct SimUi;
#[derive(Component)]
pub struct MapSprite;
#[derive(Component)]
pub struct RankUi;
#[derive(Debug, Resource)]
pub struct SimInfo {
    pub seed: Option<u64>,
    pub collider_size: f32,
    pub icon_size: f32,
    pub view_rank: bool,
    pub map_size: f32,
    pub map_color: Color,
}

pub fn sim_plugin(app: &mut App) {
    app.add_systems(OnEnter(SimState::Sim), setup)
        .add_systems(
            Update,
            (collision_event, enforce_speed, rank_view).run_if(in_state(SimState::Sim)),
        )
        .insert_resource(SimInfo {
            seed: None,
            icon_size: 2.0,
            collider_size: 2.0,
            view_rank: false,
            map_size: 0.0,
            map_color: BLACK.into(),
        })
        .add_systems(Startup, (set_wall, spawn_player).chain())
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
pub fn set_wall(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut sim_info: ResMut<SimInfo>,
) {
    let width = window.width();
    let height = window.height();

    let min_size = width.min(height);
    sim_info.map_size = min_size;

    commands.spawn((
        Sprite {
            color: sim_info.map_color,
            custom_size: Some(Vec2::new(min_size, min_size)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, -100.0),
        MainUi,
        MapSprite,
    ));

    let wall_color = Color::NONE;
    let mut mask = LayerMask::ALL;
    mask.remove(LayerMask(1));
    let wall_basic = (
        RigidBody::Static,
        Restitution::new(1.0),
        CollisionLayers::new(LayerMask(1), mask),
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

pub fn spawn_player(mut commands: Commands, custom_info: Res<CustomInfo>, sim_info: Res<SimInfo>) {
    let mut rng = if let Some(seed) = sim_info.seed {
        StdRng::seed_from_u64(seed)
    } else {
        rand::make_rng()
    };

    let collider_size = sim_info.collider_size * sim_info.map_size / (2.4 * 10.0 * 2.0);
    let icon_size = sim_info.icon_size * sim_info.map_size / (2.25 * 10.0 * 2.0);
    let half_range = (sim_info.map_size - collider_size) * 0.45;

    let player_basic = (
        RigidBody::Dynamic,
        Collider::rectangle(collider_size, collider_size),
        Restitution::new(1.0),
        CollisionEventsEnabled,
        Player,
    );
    let speed = 100.0;

    let type_num = custom_info.len;
    for i in 1..=type_num {
        let Some(handle) = custom_info.image_hash.get(&i) else {
            println!("Nope!");
            continue;
        };
        let Some(num) = custom_info.nums.get(&i) else {
            println!("Nope!");
            continue;
        };
        let mut all = LayerMask::ALL;
        let my_layer = LayerMask(1 << i);
        all.remove(my_layer);
        let player = (
            Sprite {
                image: handle.clone(),
                custom_size: Some(Vec2::new(icon_size, icon_size)),
                ..Default::default()
            },
            CollisionLayers::new(my_layer, all),
        );

        for _ in 0..num.abs() {
            let x = rng.random_range(-half_range..half_range);
            let y = rng.random_range(-half_range..half_range);

            let angle = rng.random_range(0.0..TAU);

            commands.spawn((
                LinearVelocity(Vec2::new(angle.cos() * speed, angle.sin() * speed)),
                Transform::from_xyz(x, y, -55.0),
                player_basic.clone(),
                player.clone(),
            ));
        }
    }
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
        let e1_mask = q_layer.get(event.collider1).unwrap().memberships;
        let e2_mask = q_layer.get(event.collider2).unwrap().memberships;

        let len = custom_info.len;

        let e1_layer = get_layer(e1_mask, len);
        let e2_layer = get_layer(e2_mask, len);
        println!("len: {:?}, {:?} {:?}", len, e1_layer, e2_layer);

        if e1_layer == 0 || e2_layer == 0 || e1_layer == e2_layer {
            continue;
        } else if ((e1_layer + 1) == e2_layer) || (e1_layer == len && e2_layer == 1) {
            if let Ok(mut sprite) = q_sprite.get_mut(event.collider1) {
                sprite.image = if let Some(handle) = custom_info.image_hash.get(&e2_layer) {
                    handle.clone()
                } else {
                    asset_server.load(LIST[e2_layer as usize % 3])
                };
            }
            if let Ok(mut c) = commands.get_entity(event.collider1) {
                let mut mask = LayerMask::ALL;
                mask.remove(e2_mask);
                c.insert(CollisionLayers::new(e2_mask, mask));
            }
        } else if ((e2_layer + 1) == e1_layer) || (e2_layer == len && e1_layer == 1) {
            if let Ok(mut sprite) = q_sprite.get_mut(event.collider2) {
                sprite.image = if let Some(handle) = custom_info.image_hash.get(&e1_layer) {
                    handle.clone()
                } else {
                    asset_server.load(LIST[e1_layer as usize % 3])
                };
            }
            if let Ok(mut c) = commands.get_entity(event.collider2) {
                let mut mask = LayerMask::ALL;
                mask.remove(e1_mask);
                c.insert(CollisionLayers::new(e1_mask, mask));
            }
        }
    }
}

fn get_layer(layer_mask: LayerMask, len: i32) -> i32 {
    for i in 1..=len {
        if layer_mask & (1 << i) != 0 {
            return i;
        }
    }
    0
}

fn enforce_speed(mut query: Query<&mut LinearVelocity>) {
    let target_speed = 200.0;
    for mut velocity in query.iter_mut() {
        if velocity.length() > 0.0 {
            velocity.0 = velocity.normalize() * target_speed;
        }
    }
}

fn set_rank(mut commands: Commands, custom_info: Res<CustomInfo>, sim_info: Res<SimInfo>) {
    if sim_info.view_rank == false {
        return;
    }

    for i in 1..=10.min(custom_info.len) {
        //이미지및 text 생성
    }
}

fn rank_view(
    q_player: Query<&CollisionLayers, With<Player>>,
    custom_info: Res<CustomInfo>,
    sim_info: Res<SimInfo>,
) {
    if sim_info.view_rank == false {
        return;
    }
    let mut key: Vec<i32> = (1..=custom_info.len).collect();
    let mut value = vec![0; custom_info.len as usize];

    for layer in q_player.iter() {
        let num = get_layer(layer.memberships, custom_info.len) as usize;
        value[num - 1] += 1;
    }

    key.sort_by(|a, b| value[(b - 1) as usize].cmp(&value[(a - 1) as usize]));

    println!("Rank: {:?}", key);
}
