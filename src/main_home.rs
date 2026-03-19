use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{
    app::{App, Startup},
    camera::{Camera2d, OrthographicProjection, Projection},
    color::palettes::css::{BLACK, RED, WHEAT, WHITE},
    ecs::{
        component::Component,
        observer::On,
        query::With,
        system::{Commands, ResMut, Single},
    },
    math::Vec2,
    picking::{
        Pickable,
        events::{Click, Pointer},
    },
    sprite::{Sprite, Text2d},
    state::state::NextState,
    text::{TextColor, TextFont},
    time::Time,
    transform::components::Transform,
    window::{PrimaryWindow, Window},
};

use crate::{CamerInfo, SimState};

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
) {
    let width = window.width();
    let height = window.height();

    let scale = 2.0;
    camer_info.x = width / 6.0 * scale;
    camer_info.scale = scale;
    commands.spawn((
        Camera2d,
        Transform::from_xyz(width / 6.0 * scale, 0.0, 0.0),
        Projection::Orthographic(OrthographicProjection {
            scale: scale,
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
    commands
        .spawn((
            Sprite {
                color: WHITE.into(),
                custom_size: Some(Vec2::new(block_width * 3.0, 150.0)),
                ..Default::default()
            },
            Text2d("Start".to_string()),
            TextFont {
                font_size: 100.0,
                ..Default::default()
            },
            TextColor(BLACK.into()),
            Transform::from_xyz(width / 2.0 * scale - 0.5 * block_width - 5.0, 0.0, 10.0),
            Pickable::default(),
            MainUi,
        ))
        .observe(
            |_: On<Pointer<Click>>, mut state: ResMut<NextState<SimState>>| {
                state.set(SimState::MoveToSim);
                println!("Change!");
            },
        );

    commands
        .spawn((
            Sprite {
                color: RED.into(),
                custom_size: Some(Vec2::new(block_width, 150.0)),
                ..Default::default()
            },
            Text2d("R".to_string()),
            TextFont {
                font_size: 100.0,
                ..Default::default()
            },
            TextColor(BLACK.into()),
            Transform::from_xyz(width / 2.0 * scale + 1.5 * block_width + 5.0, 0.0, 10.0),
            Pickable::default(),
            MainUi,
        ))
        .observe(
            |_: On<Pointer<Click>>, mut state: ResMut<NextState<SimState>>| {
                state.set(SimState::ReSpawnPlayer);
            },
        );
}
