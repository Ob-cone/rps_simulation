use bevy::{
    app::{App, Update},
    camera::{Camera2d, Projection},
    ecs::{
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Res, ResMut, Single},
    },
    math::{Vec3, VectorSpace},
    state::{condition::in_state, state::NextState},
    time::Time,
    transform::components::Transform,
};

use crate::SimState;

const MOVETIME: f32 = 1.5;

pub fn move_plugin(app: &mut App) {
    app.insert_resource(MoveInfo::default())
        .add_systems(Update, update.run_if(in_state(SimState::Move)));
}

#[derive(Debug, Default, Resource)]
pub struct MoveInfo {
    pub time: f32,
    pub trans: (Vec3, Vec3),
    pub scale: (f32, f32),
    pub next: SimState,
}

fn update(
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    time: Res<Time>,
    mut move_info: ResMut<MoveInfo>,
    mut state: ResMut<NextState<SimState>>,
) {
    move_info.time += time.delta_secs();
    let t = (move_info.time / MOVETIME).clamp(0.0, 1.0);
    let t_eased = t * t * (3.0 - 2.0 * t);
    let (mut trans, mut project) = camera.into_inner();

    trans.translation = move_info.trans.0.lerp(move_info.trans.1, t_eased);
    if let Projection::Orthographic(ref mut orth) = *project {
        orth.scale = move_info.scale.0.lerp(move_info.scale.1, t_eased);

        if move_info.time > MOVETIME {
            state.set(move_info.next);
            trans.translation = move_info.trans.1;
            orth.scale = move_info.scale.1;
            println!("Move End!");
        }
    }
}
