use bevy::{
    app::{App, Update},
    ecs::{
        component::Component,
        entity::{ContainsEntity, Entity},
        message::MessageReader,
        query::With,
        resource::Resource,
        schedule::{IntoScheduleConfigs, SystemCondition},
        system::{Commands, Res, ResMut, Single},
    },
    state::{
        condition::in_state,
        state::{NextState, OnEnter},
    },
    window::WindowResized,
};

use crate::{
    SimState,
    custom::{CustomUi, TypeParent, set_custom_ui, spawn_type_children, spawn_type_ui},
    despawn_screen,
    main_home::{MainUi, main_ui_setup},
    move_camera::MoveInfo,
    simulation::{Player, set_wall, spawn_player},
};

pub fn respawn_plugin(app: &mut App) {
    app.insert_resource(IsResize(false))
        .add_systems(
            OnEnter(SimState::ReSpawnPlayer),
            (despawn_screen::<Player>, spawn_player, respawn_exit).chain(),
        )
        .add_systems(
            OnEnter(SimState::ReSpawnChildren),
            (
                despawn_children::<TypeParent>,
                spawn_type_children,
                respawn_exit,
            )
                .chain(),
        )
        .add_systems(
            OnEnter(SimState::ReSpawnUi),
            (
                despawn_screen::<MainUi>,
                despawn_screen::<CustomUi>,
                main_ui_setup,
                set_custom_ui,
                set_wall,
                spawn_type_ui,
                spawn_type_children,
                respawn_exit,
            )
                .chain(),
        )
        .add_systems(Update, check_window)
        .add_systems(
            Update,
            resize_change.run_if(in_state(SimState::Main).or(in_state(SimState::Custom))),
        );
}

fn respawn_exit(mut state: ResMut<NextState<SimState>>, prev: Res<MoveInfo>) {
    println!("Respawn End! {:?}", prev.next);
    state.set(prev.next);
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

fn despawn_children<T: Component>(mut commands: Commands, q_parent: Single<Entity, With<T>>) {
    if let Ok(mut p) = commands.get_entity(q_parent.entity()) {
        p.despawn_children();
    }
}
