use avian2d::{PhysicsPlugins, prelude::Gravity};
use bevy::{
    DefaultPlugins,
    app::App,
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

use crate::{
    custom::custom_plugin, main_home::main_home_plugin, move_camera::move_plugin,
    respawn::respawn_plugin, scroller::scroller_plugin, simulation::sim_plugin,
};

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
        ))
        .run();
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
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
}

#[derive(Debug, Resource)]
struct CamerInfo {
    x: f32,
    scale: f32,
}
