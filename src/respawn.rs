use bevy::{
    app::{App, Update},
    ecs::{
        message::MessageReader, resource::Resource, schedule::IntoScheduleConfigs, system::ResMut,
    },
    state::{
        condition::in_state,
        state::{NextState, OnEnter},
    },
    window::WindowResized,
};

use crate::{
    SimState, despawn_screen,
    main_home::{MainUi, main_ui_setup},
    simulation::{Player, set_wall, spawn_player},
};

pub fn respawn_plugin(app: &mut App) {
    app.insert_resource(IsResize(false))
        .add_systems(
            OnEnter(SimState::ReSpawnPlayer),
            (despawn_screen::<Player>, spawn_player, respawn_exit).chain(),
        )
        .add_systems(
            OnEnter(SimState::ReSpawnUi),
            (
                despawn_screen::<MainUi>,
                main_ui_setup,
                set_wall,
                respawn_exit,
            )
                .chain(),
        )
        .add_systems(Update, check_window)
        .add_systems(Update, resize_change.run_if(in_state(SimState::Main)));
}

fn respawn_exit(mut state: ResMut<NextState<SimState>>) {
    state.set(SimState::Main);
}

#[derive(Debug, Resource)]
pub struct IsResize(pub bool);

fn check_window(mut resize: MessageReader<WindowResized>, mut is_resize: ResMut<IsResize>) {
    if !is_resize.0 {
        for _ in resize.read() {
            is_resize.0 = true;
        }
    }
}

fn resize_change(mut is_resize: ResMut<IsResize>, mut state: ResMut<NextState<SimState>>) {
    if is_resize.0 {
        state.set(SimState::ReSpawnUi);
        is_resize.0 = false;
    }
}
