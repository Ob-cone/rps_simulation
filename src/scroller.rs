use bevy::{
    app::{App, Update},
    ecs::{
        component::Component,
        resource::Resource,
        system::{Query, ResMut},
    },
    transform::components::Transform,
};

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
) {
    for (mut trans, scroller) in q_scroller.iter_mut() {
        if scroll_move.0 != scroller.id {
            continue;
        }
        let mul = 50.0;
        let over = scroller.height - scroller.size;
        let mut new_y = trans.translation.y;
        new_y = (new_y - scroll_move.1 * mul).clamp(scroller.start, scroller.start + over.max(0.0));

        trans.translation.y = new_y;

        scroll_move.1 = 0.0;
    }
}
