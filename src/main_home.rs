use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{
    app::{App, Startup},
    asset::AssetServer,
    camera::{Camera2d, OrthographicProjection, Projection, visibility::NoFrustumCulling},
    color::palettes::css::{BLACK, RED, WHEAT, WHITE},
    ecs::{
        component::Component,
        observer::On,
        query::With,
        system::{Commands, Res, ResMut, Single},
    },
    math::{Vec2, Vec3},
    picking::{
        Pickable,
        events::{Click, Pointer},
    },
    sprite::{Anchor, Sprite, Text2d},
    state::state::{NextState, State},
    text::{TextColor, TextFont, Underline},
    time::Time,
    transform::components::Transform,
    window::{PrimaryWindow, Window},
};

use crate::{CamerInfo, FONTPATH, SimState, move_camera::MoveInfo};

pub fn main_home_plugin(app: &mut App) {
    app.add_systems(Startup, main_ui_setup);
}

#[derive(Debug, Component)]
pub struct MainUi;

//ui spawn
pub fn main_ui_setup(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut time: ResMut<Time<Physics>>,
    mut camer_info: ResMut<CamerInfo>,
    asset_server: Res<AssetServer>,
    state: Res<MoveInfo>,
) {
    let width = window.width();
    let height = window.height();

    let scale = 2.0;
    camer_info.x = width / 6.0 * scale;
    camer_info.scale = scale;

    //println!("Last: {:?}", state.0);
    let (num, c_num) = if state.next == SimState::Custom {
        (-1.0, 1.0)
    } else if state.next == SimState::Credit {
        (4.0 - 2.0, 2.0)
    } else {
        (1.0, 1.0)
    };
    commands.spawn((
        Camera2d,
        Transform::from_xyz(width / 6.0 * scale * num, 0.0, 0.0),
        Projection::Orthographic(OrthographicProjection {
            scale: scale * c_num,
            ..OrthographicProjection::default_2d()
        }),
        MainUi,
    ));
    time.set_relative_speed(0.0);

    let ui_width = width / 3.0 * scale;

    commands.spawn((
        Sprite {
            color: WHEAT.into(),
            custom_size: Some(Vec2::new(ui_width, height * scale)),
            ..Default::default()
        },
        Transform::from_xyz(width / 2.0 * scale, 0.0, -10.0),
        MainUi,
    ));
    let block_width = ui_width * 0.2;
    let block_height = 150.0;
    commands
        .spawn((
            Sprite {
                color: WHITE.into(),
                custom_size: Some(Vec2::new(block_width * 3.0, block_height)),
                ..Default::default()
            },
            Text2d("Start".to_string()),
            TextFont {
                font: asset_server.load(FONTPATH),
                font_size: 100.0,
                ..Default::default()
            },
            TextColor(BLACK.into()),
            Transform::from_xyz(
                width / 2.0 * scale - 0.5 * block_width - 5.0,
                block_height / 2.0 + 5.0,
                10.0,
            ),
            NoFrustumCulling,
            Pickable::default(),
            MainUi,
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut state: ResMut<NextState<SimState>>,
             mut move_info: ResMut<MoveInfo>,
             camera_info: Res<CamerInfo>,
             n_state: Res<State<SimState>>| {
                if n_state.get() != &SimState::Main {
                    return;
                }
                state.set(SimState::Move);
                *move_info = MoveInfo {
                    time: 0.0,
                    trans: (Vec3::new(camera_info.x, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)),
                    scale: (camera_info.scale, 1.0),
                    next: SimState::Sim,
                };
                println!("Change!");
            },
        );

    commands
        .spawn((
            Sprite {
                color: RED.into(),
                custom_size: Some(Vec2::new(block_width, block_height)),
                ..Default::default()
            },
            Text2d("R".to_string()),
            TextFont {
                font: asset_server.load(FONTPATH),
                font_size: 100.0,
                ..Default::default()
            },
            TextColor(BLACK.into()),
            Transform::from_xyz(
                width / 2.0 * scale + 1.5 * block_width + 5.0,
                block_height / 2.0 + 5.0,
                10.0,
            ),
            NoFrustumCulling,
            Pickable::default(),
            MainUi,
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut state: ResMut<NextState<SimState>>,
             n_state: Res<State<SimState>>| {
                if n_state.get() != &SimState::Main {
                    return;
                }
                state.set(SimState::ReSpawnPlayer);
            },
        );
    commands
        .spawn((
            Sprite {
                color: WHITE.into(),
                custom_size: Some(Vec2::new(4.0 * block_width + 10.0, block_height)),
                ..Default::default()
            },
            Text2d("Custom".to_string()),
            TextFont {
                font: asset_server.load(FONTPATH),
                font_size: 100.0,
                ..Default::default()
            },
            TextColor(BLACK.into()),
            Transform::from_xyz(width / 2.0 * scale, -block_height / 2.0 - 5.0, 10.0),
            NoFrustumCulling,
            Pickable::default(),
            MainUi,
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut state: ResMut<NextState<SimState>>,
             mut move_info: ResMut<MoveInfo>,
             camera_info: Res<CamerInfo>,
             n_state: Res<State<SimState>>| {
                if n_state.get() != &SimState::Main {
                    return;
                }
                state.set(SimState::Move);
                *move_info = MoveInfo {
                    time: 0.0,
                    trans: (
                        Vec3::new(camera_info.x, 0.0, 0.0),
                        Vec3::new(-camera_info.x, 0.0, 0.0),
                    ),
                    scale: (camera_info.scale, camera_info.scale),
                    next: SimState::Custom,
                };
            },
        );

    commands
        .spawn((
            Sprite {
                color: WHITE.into(),
                custom_size: Some(Vec2::new(300.0, 40.0)),
                ..Default::default()
            },
            Transform::from_xyz(
                width / 2.0 * scale + ui_width / 2.0 - 10.0,
                -height * scale / 2.0 + 10.0,
                10.0,
            ),
            Anchor::BOTTOM_RIGHT,
            NoFrustumCulling,
            Pickable::default(),
            MainUi,
        ))
        .with_children(|p| {
            p.spawn((
                Text2d("Made by Ob-cone".to_string()),
                TextFont {
                    font: asset_server.load(FONTPATH),
                    font_size: 30.0,
                    ..Default::default()
                },
                Underline,
                TextColor(BLACK.into()),
                NoFrustumCulling,
                Pickable::default(),
                MainUi,
                Anchor::BOTTOM_RIGHT,
            ));
        })
        .observe(
            |_: On<Pointer<Click>>,
             mut move_info: ResMut<MoveInfo>,
             mut state: ResMut<NextState<SimState>>,
             camer_info: ResMut<'_, CamerInfo>| {
                let ms = 2.0;
                println!("B: {:?}", camer_info.x * (4.0 - ms));
                move_info.time = 0.0;
                move_info.next = SimState::Credit;
                move_info.scale = (camer_info.scale, camer_info.scale * 2.0);
                move_info.trans = (
                    Vec3::new(camer_info.x, 0.0, 0.0),
                    Vec3::new(camer_info.x * (4.0 - ms), 0.0, 0.0),
                );
                state.set(SimState::Move);
            },
        );
}
