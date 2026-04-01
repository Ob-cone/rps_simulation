//#![windows_subsystem = "windows"]
use crate::{
    credit::credit_plugin, custom::custom_plugin, main_home::main_home_plugin,
    move_camera::move_plugin, respawn::respawn_plugin, scroller::scroller_plugin,
    simulation::sim_plugin,
};
use avian2d::{PhysicsPlugins, prelude::Gravity};
use bevy::prelude::NonSend;

use bevy::window::WindowResolution;
use bevy::winit::WinitWindows;
use bevy::{
    DefaultPlugins,
    app::{App, Startup},
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        resource::Resource,
        system::{Commands, Query},
    },
    prelude::PluginGroup,
    state::{app::AppExtStates, state::States},
    window::{Window, WindowPlugin},
};
use bevy_bc_ime_text_field::ImeTextFieldPlugin;

mod credit;
mod custom;
mod main_home;
mod move_camera;
mod respawn;
mod scroller;
mod simulation;

const FONTPATH: &str = "font/PixelCode-Bold.otf";
const LIST: [&str; 3] = ["rock.png", "paper.png", "scissors.png"];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "rps_sim".into(),
                resolution: WindowResolution::new(1280, 720),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(ImeTextFieldPlugin)
        .insert_resource(Gravity::ZERO)
        .insert_resource(CamerInfo { x: 0.0, scale: 0.0 })
        .init_state::<SimState>()
        .add_plugins((
            main_home_plugin,
            sim_plugin,
            move_plugin,
            custom_plugin,
            respawn_plugin,
            scroller_plugin,
            credit_plugin,
        ))
        .add_systems(Startup, set_window_icon)
        .run();
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

fn set_window_icon(_windows: Option<NonSend<WinitWindows>>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use bevy::winit::WINIT_WINDOWS;

        WINIT_WINDOWS.with_borrow_mut(|window| {
            use winit::window::Icon;

            let image = image::open("assets/rps_icon.png")
                .expect("아이콘 파일 없음")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();

            let icon = Icon::from_rgba(rgba, width, height).unwrap();

            if window.windows.is_empty() {
                return;
            }
            for window in window.windows.values() {
                window.set_window_icon(Some(icon.clone()));
            }
        });
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum SimState {
    #[default]
    Main,
    Move,
    ReSpawnPlayer,
    ReSpawnUi,
    ReSpawnChildren,
    Sim,
    Custom,
    Credit,
}

#[derive(Debug, Resource)]
struct CamerInfo {
    x: f32,
    scale: f32,
}
