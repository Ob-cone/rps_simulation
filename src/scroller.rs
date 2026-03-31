use bevy::{
    app::{App, Update},
    ecs::{
        component::Component,
        resource::Resource,
        system::{Query, Res, ResMut},
    },
    state::state::State,
    transform::components::Transform,
};

use crate::SimState;

#[derive(Debug, Resource)]
pub struct ScrollMove(pub i32, pub f32);

#[derive(Debug, Component)]
pub struct Scroller {
    pub id: i32,
    pub height: f32,
    pub start: f32,
    pub size: f32,
}

pub fn scroller_plugin(app: &mut App) {
    app.insert_resource(ScrollMove(-1, 0.0))
        .add_systems(Update, move_scroller);
}

fn move_scroller(
    mut scroll_move: ResMut<ScrollMove>,
    mut q_scroller: Query<(&mut Transform, &Scroller)>,
    state: Res<State<SimState>>,
) {
    for (mut trans, scroller) in q_scroller.iter_mut() {
        if scroll_move.0 != scroller.id {
            continue;
        }
        #[cfg(not(target_arch = "wasm32"))]
        let mut mul = 50.0;

        #[cfg(target_arch = "wasm32")]
        let mut mul = 0.5;

        if state.get() == &SimState::Credit {
            mul *= 1.5;
        }
        let over = scroller.height - scroller.size;
        let mut new_y = trans.translation.y;
        new_y = (new_y - scroll_move.1 * mul).clamp(scroller.start, scroller.start + over.max(0.0));

        trans.translation.y = new_y;

        scroll_move.1 = 0.0;
    }
}
