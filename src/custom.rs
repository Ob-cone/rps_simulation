use std::{
    collections::HashMap,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

use bevy::{
    app::{App, PreStartup, Startup, Update},
    asset::{AssetServer, Assets, Handle, RenderAssetUsages},
    camera::visibility::NoFrustumCulling,
    color::palettes::css::{BLACK, RED, WHEAT, WHITE},
    ecs::{
        component::Component,
        observer::On,
        query::With,
        resource::Resource,
        system::{Commands, Res, ResMut, Single},
    },
    image::Image,
    math::{Vec2, Vec3},
    picking::{
        Pickable,
        events::{Click, Pointer},
    },
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::{Anchor, Sprite, Text2d},
    state::state::NextState,
    text::{TextColor, TextFont},
    transform::components::Transform,
    window::{PrimaryWindow, Window},
};

use crate::{CamerInfo, FONTPATH, SimState, move_camera::MoveInfo, respawn::IsResize};

#[derive(Debug, Resource)]
pub struct CustomInfo {
    pub size: f32,
    pub image_hash: HashMap<i32, Handle<Image>>,
    pub nums: HashMap<i32, i32>,
}

impl Default for CustomInfo {
    fn default() -> Self {
        Self {
            size: 1.0,
            image_hash: HashMap::new(),
            nums: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ReadImage(pub i32, pub Vec<u8>);
#[derive(Debug, Resource)]
pub struct ImageChannel(pub Sender<ReadImage>, pub Mutex<Receiver<ReadImage>>);
#[derive(Debug, Resource)]
pub struct CustomSelect(pub bool);

pub fn custom_plugin(app: &mut App) {
    let (tx, rx) = channel::<ReadImage>();

    app.insert_resource(CustomInfo::default())
        .insert_resource(ImageChannel(tx, Mutex::new(rx)))
        .insert_resource(CustomSelect(false))
        .add_systems(PreStartup, custom_info_reset)
        .add_systems(Startup, set_custom_ui)
        .add_systems(Update, add_image);
}

fn custom_info_reset(mut custom_info: ResMut<CustomInfo>, asset_server: Res<AssetServer>) {
    custom_info.size = 2.0;
    custom_info.image_hash.clear();
    custom_info
        .image_hash
        .insert(1, asset_server.load("rock.png"));
    custom_info.nums.insert(1, 35);
    custom_info
        .image_hash
        .insert(2, asset_server.load("paper.png"));
    custom_info.nums.insert(2, 35);
    custom_info
        .image_hash
        .insert(3, asset_server.load("scissors.png"));
    custom_info.nums.insert(2, 35);
}

fn add_image(
    mut custom_info: ResMut<CustomInfo>,
    mut images: ResMut<Assets<Image>>,
    image_channel: Res<ImageChannel>,
) {
    if let Ok(read_image) = image_channel.1.lock().unwrap().try_recv() {
        let Ok(dyn_img) = image::load_from_memory(&read_image.1) else {
            return;
        };
        let rgba = dyn_img.to_rgba8();
        let (w, h) = rgba.dimensions();

        let handle = images.add(Image::new(
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba.into_raw(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        ));

        custom_info.image_hash.insert(read_image.0, handle);
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct CustomUi;

pub fn set_custom_ui(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_info: Res<CamerInfo>,
    asset_server: Res<AssetServer>,
    select: Res<CustomSelect>,
) {
    let width = window.width();
    let height = window.height();

    let scale = camera_info.scale;
    let ui_width = width / 3.0 * scale;

    commands.spawn((
        Sprite {
            color: WHEAT.into(),
            custom_size: Some(Vec2::new(ui_width, height * scale)),
            ..Default::default()
        },
        Transform::from_xyz(-width / 2.0 * scale, 0.0, -10.0),
        CustomUi,
    ));

    let block_width = ui_width * 0.2;
    let block_height = 150.0;
    let y = 0.4 * height * scale;
    let mut bundle = (
        TextFont {
            font: asset_server.load(FONTPATH),
            font_size: 100.0,
            ..Default::default()
        },
        TextColor(BLACK.into()),
        Pickable::default(),
        CustomUi,
    );
    commands
        .spawn((
            Sprite {
                color: WHITE.into(),
                custom_size: Some(Vec2::new(block_width, block_height)),
                ..Default::default()
            },
            Text2d("T".to_string()),
            Transform::from_xyz(
                -width / 2.0 * scale - 1.5 * block_width - 15.0,
                y - 10.0 * (!select.0 as i32 as f32),
                10.0,
            ),
            bundle.clone(),
            NoFrustumCulling,
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut select: ResMut<CustomSelect>,
             mut resize: ResMut<IsResize>| {
                select.0 = false;
                resize.0 = true;
            },
        );

    commands
        .spawn((
            Sprite {
                color: WHITE.into(),
                custom_size: Some(Vec2::new(block_width, block_height)),
                ..Default::default()
            },
            Text2d("E".to_string()),
            Transform::from_xyz(
                -width / 2.0 * scale - 0.5 * block_width - 5.0,
                y - 10.0 * (select.0 as i32 as f32),
                10.0,
            ),
            bundle.clone(),
            NoFrustumCulling,
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut select: ResMut<CustomSelect>,
             mut resize: ResMut<IsResize>| {
                select.0 = true;
                resize.0 = true;
            },
        );

    commands
        .spawn((
            Sprite {
                color: RED.into(),
                custom_size: Some(Vec2::new(block_width, block_height)),
                ..Default::default()
            },
            Text2d("R".to_string()),
            Transform::from_xyz(-width / 2.0 * scale + 0.5 * block_width + 5.0, y, 10.0),
            bundle.clone(),
            NoFrustumCulling,
        ))
        .observe(
            |_: On<Pointer<Click>>, mut state: ResMut<NextState<SimState>>| {
                state.set(SimState::ReSpawnPlayer);
            },
        );

    bundle.0.font_size = 150.0;

    commands
        .spawn((
            Sprite {
                color: RED.into(),
                custom_size: Some(Vec2::new(block_width, block_height)),
                ..Default::default()
            },
            Text2d("➡".to_string()),
            Transform::from_xyz(-width / 2.0 * scale + 1.5 * block_width + 15.0, y, 10.0),
            bundle.clone(),
            NoFrustumCulling,
        ))
        .observe(
            |_: On<Pointer<Click>>,
             mut state: ResMut<NextState<SimState>>,
             mut move_info: ResMut<MoveInfo>,
             camera_info: Res<CamerInfo>| {
                state.set(SimState::Move);
                *move_info = MoveInfo {
                    time: 0.0,
                    trans: (
                        Vec3::new(-camera_info.x, 0.0, 0.0),
                        Vec3::new(camera_info.x, 0.0, 0.0),
                    ),
                    scale: (camera_info.scale, camera_info.scale),
                    next: SimState::Main,
                };
                println!("Change!");
            },
        );

    commands.spawn((
        Sprite {
            color: WHITE.into(),
            custom_size: Some(Vec2::new(4.0 * block_width + 30.0, 2.0 * y - 10.0)),
            ..Default::default()
        },
        Transform::from_xyz(-width / 2.0 * scale, -5.0 - block_height / 2.0, 0.0),
        CustomUi,
        NoFrustumCulling,
    ));

    commands.spawn((
        Sprite {
            color: WHEAT.into(),
            custom_size: Some(Vec2::new(
                4.0 * block_width + 30.0 - 40.0,
                2.0 * y - 10.0 - 40.0,
            )),
            ..Default::default()
        },
        Anchor::TOP_CENTER,
        Transform::from_xyz(-width / 2.0 * scale, y - block_height / 2.0 - 30.0, 0.0),
        CustomUi,
        NoFrustumCulling,
    ));
}
