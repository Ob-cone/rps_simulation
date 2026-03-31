use bevy::{
    app::{App, Startup},
    asset::AssetServer,
    camera::visibility::{NoFrustumCulling, Visibility},
    color::{
        Color,
        palettes::css::{BLACK, RED},
    },
    ecs::{
        component::Component,
        observer::On,
        query::With,
        system::{Commands, Res, ResMut, Single},
    },
    math::{Vec2, Vec3},
    picking::{
        Pickable,
        events::{Click, Pointer, Scroll},
    },
    sprite::{Anchor, Sprite, Text2d},
    state::state::NextState,
    text::TextFont,
    transform::components::Transform,
    window::{PrimaryWindow, Window},
};
use cargo_toml::{Dependency, Manifest};

use crate::{
    CamerInfo, FONTPATH, SimState,
    main_home::MainUi,
    move_camera::MoveInfo,
    scroller::{ScrollMove, Scroller},
};

#[derive(Component)]
struct CreditParent;

pub fn credit_plugin(app: &mut App) {
    app.add_systems(Startup, set_credit);
}

pub fn set_credit(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camer_info: Res<CamerInfo>,
    asset_server: Res<AssetServer>,
) {
    let width = window.width();
    let height = window.height();

    let scale = camer_info.scale * 2.0;
    let ui_width = width / 3.0 * scale;
    let left = ui_width / 2.0 + camer_info.x * (4.0 - 2.0);

    let bytes = include_bytes!("../Cargo.toml");
    let manifest = Manifest::from_slice(bytes).unwrap();
    let package = manifest.package.unwrap();

    commands.spawn((
        Sprite {
            color: BLACK.into(),
            custom_size: Some(Vec2::new(ui_width, height * scale)),
            ..Default::default()
        },
        Transform::from_xyz(left, 0.0, -10.0),
        Anchor::CENTER_LEFT,
        NoFrustumCulling,
        MainUi,
    ));

    commands
        .spawn((
            Sprite {
                color: RED.into(),
                custom_size: Some(Vec2::new(125.0, 125.0)),
                ..Default::default()
            },
            Anchor::TOP_RIGHT,
            Transform::from_xyz(left + ui_width - 25.0, height * scale / 2.0 - 25.0, 20.0),
            NoFrustumCulling,
            Pickable::default(),
            MainUi,
        ))
        .with_children(|p| {
            p.spawn((
                Text2d::new("✕"),
                TextFont {
                    font: asset_server.load(FONTPATH),
                    font_size: 125.0,
                    ..Default::default()
                },
                Anchor::TOP_RIGHT,
                Transform::from_xyz(-15.0, 35.0, 0.0),
            ));
        })
        .observe(
            |_: On<Pointer<Click>>,
             mut move_info: ResMut<MoveInfo>,
             mut state: ResMut<NextState<SimState>>,
             camer_info: ResMut<'_, CamerInfo>| {
                let ms = 2.0;
                move_info.time = 0.0;
                move_info.next = SimState::Main;
                move_info.scale = (camer_info.scale * 2.0, camer_info.scale);
                move_info.trans = (
                    Vec3::new(camer_info.x * (4.0 - ms), 0.0, 0.0),
                    Vec3::new(camer_info.x, 0.0, 0.0),
                );
                state.set(SimState::Move);
            },
        );

    commands.spawn((
        Sprite {
            color: BLACK.into(),
            custom_size: Some(Vec2::new(ui_width * 0.8, height * scale * 0.1)),
            ..Default::default()
        },
        Anchor::TOP_CENTER,
        Transform::from_xyz(left + ui_width * 0.5, height * scale * 0.5, 10.0),
        MainUi,
        NoFrustumCulling,
        Pickable::default(),
    ));

    commands.spawn((
        Sprite {
            color: BLACK.into(),
            custom_size: Some(Vec2::new(ui_width * 0.8, height * scale * 0.1)),
            ..Default::default()
        },
        Anchor::TOP_CENTER,
        Transform::from_xyz(left + ui_width * 0.5, -height * scale * 0.4, 10.0),
        MainUi,
        NoFrustumCulling,
        Pickable::default(),
    ));

    commands
        .spawn((
            Sprite {
                color: BLACK.into(),
                custom_size: Some(Vec2::new(ui_width * 0.8, height * scale * 0.8)),
                ..Default::default()
            },
            Anchor::TOP_CENTER,
            Transform::from_xyz(left + ui_width * 0.5, height * scale * 0.4, 0.0),
            MainUi,
            NoFrustumCulling,
            Pickable::default(),
        ))
        .observe(
            |trigger: On<Pointer<Scroll>>, mut scroll: ResMut<ScrollMove>| {
                //println!("SC: {:?}",trigger.y);
                scroll.0 = 3;
                scroll.1 += trigger.y;
            },
        )
        .with_children(|p| {
            let h = 1680.0 + (manifest.dependencies.len() + 1) as f32 * 230.0;

            p.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Visibility::default(),
                Anchor::TOP_CENTER,
                Scroller {
                    id: 3,
                    height: h,
                    start: 0.0,
                    size: height * scale * 0.8,
                },
                CreditParent,
            ))
            .with_children(|p| {
                let ver = if let Ok(ver) = package.version.get() {
                    ver
                } else {
                    "?.?.?"
                };
                let title = format!("rps_sim v{}", ver);

                let sprite_color = Color::NONE;

                p.spawn((
                    Sprite {
                        color: sprite_color,
                        custom_size: Some(Vec2::new(1300.0, 150.0)),
                        ..Default::default()
                    },
                    Transform::from_xyz(0.0, -35.0, 1.0),
                    Pickable::default(),
                    Anchor::TOP_CENTER,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text2d(title),
                        TextFont {
                            font: asset_server.load(FONTPATH),
                            font_size: 120.0,
                            ..Default::default()
                        },
                        Anchor::TOP_CENTER,
                    ));
                })
                .observe(|_: On<Pointer<Click>>| {
                    let _ = open_link("https://github.com/Ob-cone/rps_simulation".to_string());
                });

                let title = (
                    TextFont {
                        font: asset_server.load(FONTPATH),
                        font_size: 100.0,
                        ..Default::default()
                    },
                    Anchor::TOP_CENTER,
                );

                let info_sprite = (
                    Sprite {
                        color: sprite_color,
                        custom_size: Some(Vec2::new(ui_width * 0.8 * 0.9, 215.0)),
                        ..Default::default()
                    },
                    Pickable::default(),
                    Anchor::TOP_LEFT,
                );

                let info_1 = (
                    TextFont {
                        font: asset_server.load(FONTPATH),
                        font_size: 80.0,
                        ..Default::default()
                    },
                    Anchor::TOP_LEFT,
                );

                let info_2 = (
                    TextFont {
                        font: asset_server.load(FONTPATH),
                        font_size: 80.0,
                        ..Default::default()
                    },
                    Anchor::TOP_RIGHT,
                );

                p.spawn((
                    Text2d::new("Team"),
                    title.clone(),
                    Transform::from_xyz(0.0, -215.0, 0.0),
                ));

                p.spawn((
                    info_sprite.clone(),
                    Transform::from_xyz(-ui_width * 0.36, -330.0, 1.0),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text2d::new("Planning"),
                        info_1.clone(),
                        Transform::from_xyz(0.0, 0.0, 1.0),
                    ));
                    p.spawn((
                        Text2d::new("Ob-cone"),
                        info_2.clone(),
                        Transform::from_xyz(ui_width * 0.8 * 0.9, -115.0, 1.0),
                    ));
                })
                .observe(|_: On<Pointer<Click>>| {
                    let _ = open_link("https://github.com/Ob-cone".to_string());
                });

                p.spawn((
                    info_sprite.clone(),
                    Transform::from_xyz(-ui_width * 0.36, -560.0, 1.0),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text2d::new("Programming"),
                        info_1.clone(),
                        Transform::from_xyz(0.0, 0.0, 1.0),
                    ));
                    p.spawn((
                        Text2d::new("Ob-cone"),
                        info_2.clone(),
                        Transform::from_xyz(ui_width * 0.8 * 0.9, -115.0, 1.0),
                    ));
                })
                .observe(|_: On<Pointer<Click>>| {
                    let _ = open_link("https://github.com/Ob-cone".to_string());
                });

                p.spawn((
                    info_sprite.clone(),
                    Transform::from_xyz(-ui_width * 0.36, -790.0, 1.0),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text2d::new("Design"),
                        info_1.clone(),
                        Transform::from_xyz(0.0, 0.0, 1.0),
                    ));
                    p.spawn((
                        Text2d::new("Ob-cone"),
                        info_2.clone(),
                        Transform::from_xyz(ui_width * 0.8 * 0.9, -115.0, 1.0),
                    ));
                })
                .observe(|_: On<Pointer<Click>>| {
                    let _ = open_link("https://github.com/Ob-cone".to_string());
                });

                p.spawn((
                    Text2d::new("Font"),
                    title.clone(),
                    Transform::from_xyz(0.0, -1120.0, 0.0),
                ));

                p.spawn((
                    info_sprite.clone(),
                    Transform::from_xyz(-ui_width * 0.36, -1350.0, 1.0),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text2d::new("PixelCode"),
                        info_1.clone(),
                        Transform::from_xyz(0.0, 0.0, 1.0),
                    ));
                    p.spawn((
                        Text2d::new("qwerasd205"),
                        info_2.clone(),
                        Transform::from_xyz(ui_width * 0.8 * 0.9, -115.0, 1.0),
                    ));
                })
                .observe(|_: On<Pointer<Click>>| {
                    let _ = open_link("https://github.com/qwerasd205/PixelCode".to_string());
                });

                p.spawn((
                    Text2d::new("Special Thanks"),
                    title.clone(),
                    Transform::from_xyz(0.0, -1680.0, 0.0),
                ));
                let mut i = 1.0;
                for (name, dep) in &manifest.dependencies {
                    let ver = match dep {
                        Dependency::Simple(info) => info.clone(),
                        Dependency::Inherited(_) => "None".to_string(),
                        Dependency::Detailed(info) => match &info.version {
                            None => "None".to_string(),
                            Some(ver) => ver.clone(),
                        },
                    };

                    let mut entity = p.spawn((
                        info_sprite.clone(),
                        Transform::from_xyz(-ui_width * 0.36, -1680.0 - i * 230.0, 1.0),
                    ));
                    entity.with_children(|p| {
                        p.spawn((
                            Text2d::new(name.clone()),
                            info_1.clone(),
                            Transform::from_xyz(0.0, 0.0, 1.0),
                        ));
                        p.spawn((
                            Text2d::new(ver.to_string()),
                            info_2.clone(),
                            Transform::from_xyz(ui_width * 0.8 * 0.9, -115.0, 1.0),
                        ));
                    });

                    if ver != "None".to_string() {
                        let url_name = name.clone();
                        entity.observe(move |_: On<Pointer<Click>>| {
                            let url = format!("https://crates.io/crates/{}", url_name);
                            let _ = open_link(url);
                        });
                    }

                    i += 1.0;
                }
            });
        });
}

#[cfg(not(target_arch = "wasm32"))]
fn open_link(url: String) {
    // PC 환경: open 라이브러리 사용
    let _ = open::that(url);
}

#[cfg(target_arch = "wasm32")]
fn open_link(url: String) {
    // WASM 환경: web-sys 사용
    let window = web_sys::window().unwrap();
    let _ = window.open_with_url(&url);
}
