use bevy::{
    app::{App, Update},
    camera::{Camera2d, Projection},
    ecs::{
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Res, ResMut, Single},
    },
    math::VectorSpace,
    state::{condition::in_state, state::NextState},
    time::Time,
    transform::components::Transform,
};

use crate::{CamerInfo, SimState};

pub fn move_plugin(app: &mut App) {
    app.insert_resource(MoveInfo(0.0))
        .add_systems(Update, update_to_sim.run_if(in_state(SimState::MoveToSim)))
        .add_systems(
            Update,
            update_to_main.run_if(in_state(SimState::MoveToMain)),
        );
}

#[derive(Debug, Resource)]
pub struct MoveInfo(pub f32);

fn update_to_sim(
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    time: Res<Time>,
    mut move_info: ResMut<MoveInfo>,
    camera_info: Res<CamerInfo>,
    mut state: ResMut<NextState<SimState>>,
) {
    move_info.0 += time.delta_secs();
    let t = (move_info.0 / 1.5).clamp(0.0, 1.0);
    let t_eased = t * t * (3.0 - 2.0 * t);
    let (mut trans, mut project) = camera.into_inner();

    trans.translation.x = camera_info.x.lerp(0.0, t_eased);
    if let Projection::Orthographic(ref mut orth) = *project {
        orth.scale = camera_info.scale.lerp(1.0, t_eased);

        if trans.translation.x < 1.0 {
            state.set(SimState::Sim);
            trans.translation.x = 0.0;
            orth.scale = 1.0;
            move_info.0 = 0.0;
            println!("Move End!");
        }
    }
}

fn update_to_main(
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    time: Res<Time>,
    mut move_info: ResMut<MoveInfo>,
    camera_info: Res<CamerInfo>,
    mut state: ResMut<NextState<SimState>>,
) {
    move_info.0 += time.delta_secs();
    let t = (move_info.0 / 1.5).clamp(0.0, 1.0);
    let t_eased = t * t * (3.0 - 2.0 * t);
    let (mut trans, mut project) = camera.into_inner();

    trans.translation.x = 0.0.lerp(camera_info.x, t_eased);
    if let Projection::Orthographic(ref mut orth) = *project {
        orth.scale = 1.0.lerp(camera_info.scale, t_eased);

        if (camera_info.x - trans.translation.x).abs() < 1.0 {
            state.set(SimState::Main);
            trans.translation.x = camera_info.x;
            orth.scale = camera_info.scale;
            move_info.0 = 0.0;
            println!("Move End!");
        }
    }
}
