#[cfg(not(target_arch = "wasm32"))]
use std::thread::spawn;
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
    camera::visibility::{NoFrustumCulling, Visibility},
    color::{
        Color,
        palettes::css::{BLACK, BLUE, GRAY, RED, WHEAT, WHITE},
    },
    ecs::{
        component::Component,
        entity::{ContainsEntity, Entity},
        message::{Message, MessageReader, MessageWriter},
        observer::On,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut, Single},
    },
    image::Image,
    math::{Vec2, Vec3},
    picking::{
        Pickable,
        events::{Click, Pointer, Scroll},
    },
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::{Anchor, Sprite, Text2d},
    state::state::NextState,
    text::{TextColor, TextFont},
    transform::components::Transform,
    window::{PrimaryWindow, Window},
};
use bevy_bc_ime_text_field::{
    event::EnterEvent,
    text_field::{TextField, TextFieldInfo},
    text_field_style::TextFieldStyle,
};
use rand::seq::SliceRandom;

use crate::{
    CamerInfo, FONTPATH, SimState,
    move_camera::MoveInfo,
    respawn::IsResize,
    scroller::{ScrollMove, Scroller},
};

#[derive(Debug, Resource)]
pub struct CustomInfo {
    pub size: f32,
    pub image_hash: HashMap<i32, Handle<Image>>,
    pub nums: HashMap<i32, i32>,
    pub len: i32,
}

impl Default for CustomInfo {
    fn default() -> Self {
        Self {
            size: 1.0,
            image_hash: HashMap::new(),
            nums: HashMap::new(),
            len: 0,
        }
    }
}

#[derive(Debug)]
pub struct ReadImage(pub i32, pub Vec<u8>);
#[derive(Debug, Resource)]
pub struct ImageChannel(pub Sender<ReadImage>, pub Mutex<Receiver<ReadImage>>);
#[derive(Debug, Resource)]
pub struct CustomSelect(pub bool);
#[derive(Component)]
pub struct TypeParent;
#[derive(Debug, Component, Clone, Copy)]
pub struct CustomUi;
#[derive(Debug, Component)]
pub struct CustomIcon(i32);
#[derive(Debug, Component)]
pub struct CustomNum(i32);

#[derive(Debug, Message)]
pub struct RemoveType(i32);
#[derive(Debug, Component)]
pub struct RemoveIcon;

pub fn custom_plugin(app: &mut App) {
    let (tx, rx) = channel::<ReadImage>();

    app.insert_resource(CustomInfo::default())
        .add_message::<RemoveType>()
        .insert_resource(ImageChannel(tx, Mutex::new(rx)))
        .insert_resource(CustomSelect(false))
        .add_systems(PreStartup, custom_info_reset)
        .add_systems(Startup, set_custom_ui)
        .add_systems(Update, (add_image, remove_type))
        .add_systems(Startup, (spawn_type_ui, spawn_type_children));
}

fn custom_info_reset(mut custom_info: ResMut<CustomInfo>, asset_server: Res<AssetServer>) {
    custom_info.size = 2.0;
    custom_info.image_hash.clear();
    custom_info.nums.clear();
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
    custom_info.nums.insert(3, 35);

    custom_info.len = 3;
}

fn add_image(
    mut custom_info: ResMut<CustomInfo>,
    mut images: ResMut<Assets<Image>>,
    image_channel: Res<ImageChannel>,
    mut q_icon: Query<(&CustomIcon, &mut Sprite)>,
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

        custom_info.image_hash.insert(read_image.0, handle.clone());

        for (num, mut splite) in q_icon.iter_mut() {
            if num.0 == read_image.0 {
                splite.image = handle.clone();
            }
        }
    }
}

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
            color: WHITE.into(),
            custom_size: Some(Vec2::new(4.0 * block_width + 30.0, 20.0)),
            ..Default::default()
        },
        Transform::from_xyz(-width / 2.0 * scale, y - 0.5 * block_height - 20.0, 5.0),
        CustomUi,
        NoFrustumCulling,
    ));

    commands.spawn((
        Sprite {
            color: WHEAT.into(),
            custom_size: Some(Vec2::new(
                4.0 * block_width + 30.0,
                (height * scale / 2.0) - (y - 0.5 * block_height - 10.0),
            )),
            ..Default::default()
        },
        Transform::from_xyz(-width / 2.0 * scale, y - 0.5 * block_height - 10.0, 5.0),
        CustomUi,
        Anchor::BOTTOM_CENTER,
        NoFrustumCulling,
    ));

    commands.spawn((
        Sprite {
            color: WHITE.into(),
            custom_size: Some(Vec2::new(4.0 * block_width + 30.0, 20.0)),
            ..Default::default()
        },
        Transform::from_xyz(-width / 2.0 * scale, -y - block_width / 2.0 + 20.0, 15.0),
        CustomUi,
        NoFrustumCulling,
    ));
    commands.spawn((
        Sprite {
            color: WHEAT.into(),
            custom_size: Some(Vec2::new(
                4.0 * block_width + 30.0,
                (height * scale / 2.0) - y - block_width / 2.0 + 20.0,
            )),
            ..Default::default()
        },
        Transform::from_xyz(-width / 2.0 * scale, -y - block_width / 2.0 + 20.0, 13.0),
        CustomUi,
        Anchor::TOP_CENTER,
        NoFrustumCulling,
    ));
}

pub fn spawn_type_ui(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    select: Res<CustomSelect>,
    camera_info: Res<CamerInfo>,
    custom_info: Res<CustomInfo>,
) {
    if select.0 == true {
        return;
    }
    let width = window.width();
    let height = window.height();

    let scale = camera_info.scale;
    let ui_width = width / 3.0 * scale;

    let block_width = ui_width * 0.2;
    let block_height = 150.0;
    let y = 0.4 * height * scale;

    commands
        .spawn((
            Sprite {
                color: WHEAT.into(),
                custom_size: Some(Vec2::new(
                    4.0 * block_width + 30.0 - 40.0,
                    2.0 * y - 10.0 - 40.0,
                )),
                ..Default::default()
            },
            Anchor::TOP_CENTER,
            Transform::from_xyz(-width / 2.0 * scale, y - block_height / 2.0 - 30.0, 0.5),
            CustomUi,
            NoFrustumCulling,
            Pickable::default(),
        ))
        .observe(
            |trigger: On<Pointer<Scroll>>, mut scroll: ResMut<ScrollMove>| {
                //println!("SC: {:?}",trigger.y);
                scroll.0 = 1;
                scroll.1 += trigger.y;
            },
        )
        .with_children(|p| {
            let width = 4.0 * block_width + 30.0 - 40.0;
            let item_nums = custom_info.len;
            p.spawn((
                Transform::from_xyz(-width / 2.0 + 10.0, -10.0, 0.0),
                Visibility::default(),
                Anchor::TOP_LEFT,
                Scroller {
                    id: 1,
                    height: 160.0 * (item_nums as f32 + 1.0),
                    start: -10.0,
                    size: 2.0 * y - 10.0 - 40.0 - 20.0,
                },
                TypeParent,
            ));
        });
}

pub fn spawn_type_children(
    mut commands: Commands,
    q_parent: Single<Entity, With<TypeParent>>,
    window: Single<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    custom_info: Res<CustomInfo>,
    camera_info: Res<CamerInfo>,
) {
    let entity = q_parent.entity();
    if let Ok(mut p) = commands.get_entity(entity) {
        let width = window.width();
        let scale = camera_info.scale;
        let ui_width = width / 3.0 * scale;

        let block_width = ui_width * 0.2;
        let width = 4.0 * block_width + 30.0 - 40.0;
        println!("A");
        p.with_children(|p| {
            let node = (
                Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.5),
                    custom_size: Some(Vec2::new(width - 20.0, 150.0)),
                    ..Default::default()
                },
                Anchor::TOP_LEFT,
            );
            let item_nums = custom_info.len;
            println!("item_nums: {:?}", item_nums);
            for i in 1..=item_nums {
                let num = i;

                p.spawn((
                    node.clone(),
                    Transform::from_xyz(0.0, -160.0 * (i - 1) as f32, 0.6),
                ))
                .with_children(|p| {
                    let image = if let Some(image) = custom_info.image_hash.get(&i) {
                        image.clone()
                    } else {
                        asset_server.load("rock.png")
                    };

                    p.spawn((
                        Sprite {
                            image: image,
                            custom_size: Some(Vec2::new(130.0, 130.0)),
                            color: WHITE.into(),
                            ..Default::default()
                        },
                        Transform::from_xyz(10.0, -10.0, 0.5),
                        Anchor::TOP_LEFT,
                        Pickable::default(),
                        CustomIcon(i),
                    ))
                    .with_child((
                        Sprite::from_color(WHITE, Vec2::new(130.0, 130.0)),
                        Transform::from_xyz(0.0, 0.0, -0.1),
                        Anchor::TOP_LEFT,
                        Pickable::default(),
                    ))
                    .observe(
                        move |_: On<Pointer<Click>>, image_channel: Res<ImageChannel>| {
                            let tx = image_channel.0.clone();
                            println!("Num: {:?}", num);
                            #[cfg(not(target_arch = "wasm32"))]
                            spawn(move || {
                                pollster::block_on(async {
                                    use crate::custom::ReadImage;
                                    use rfd::AsyncFileDialog;

                                    let Some(file) = AsyncFileDialog::new()
                                        .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                                        .pick_file()
                                        .await
                                    else {
                                        return;
                                    };

                                    let bytes = file.read().await;
                                    let _ = tx.send(ReadImage(num, bytes));
                                });
                            });

                            #[cfg(target_arch = "wasm32")]
                            wasm_bindgen_futures::spawn_local(async move {
                                use crate::custom::ReadImage;
                                use rfd::AsyncFileDialog;

                                let Some(file) = AsyncFileDialog::new()
                                    .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                                    .pick_file()
                                    .await
                                else {
                                    return;
                                };

                                let bytes = file.read().await;
                                let _ = tx.send(ReadImage(num, bytes));
                            });
                        },
                    );

                    p.spawn((
                        Text2d("num".to_string()),
                        TextFont {
                            font: asset_server.load(FONTPATH),
                            font_size: 50.0,
                            ..Default::default()
                        },
                        TextColor(BLACK.into()),
                        Anchor::TOP_LEFT,
                        Transform::from_xyz(160.0, -10.0, 0.5),
                    ));

                    p.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(210.0, 70.0)),
                            color: WHITE.into(),
                            ..Default::default()
                        },
                        Transform::from_xyz(150.0, -70.0, 0.5),
                        Anchor::TOP_LEFT,
                        Pickable::default(),
                        Visibility::default(),
                    ))
                    .with_children(|p| {
                        let num = if let Some(num) = custom_info.nums.get(&i) {
                            num.abs()
                        } else {
                            35
                        };

                        p.spawn((
                            TextField::default(),
                            TextFieldStyle {
                                font: TextFont {
                                    font: asset_server.load(FONTPATH),
                                    font_size: 50.0,
                                    ..Default::default()
                                },
                                color: BLACK.into(),
                                placeholder_color: GRAY.into(),
                                ..Default::default()
                            },
                            TextFieldInfo {
                                focus: false,
                                max_length: Some(3),
                                placeholder: Some(num.to_string()),
                                ..Default::default()
                            },
                            Text2d::default(),
                            Transform::from_xyz(10.0, -1.0, 0.5),
                            Visibility::default(),
                            Anchor::TOP_LEFT,
                            CustomNum(i),
                        ))
                        .observe(
                            |trigger: On<EnterEvent>,
                             mut q_field: Query<(
                                &mut TextField,
                                &mut TextFieldInfo,
                                &CustomNum,
                            )>,
                             mut custom_info: ResMut<CustomInfo>| {
                                println!("Enter: {:?}", trigger.text_field.text);
                                if let Ok((mut text, mut info, num)) =
                                    q_field.get_mut(trigger.entity)
                                {
                                    text.text.clear();
                                    info.focus = false;
                                    if let Ok(parsed) =
                                        trigger.text_field.text.trim().parse::<i32>()
                                    {
                                        custom_info.nums.insert(num.0, i32::abs(parsed));
                                        info.placeholder = Some(parsed.abs().to_string());
                                    }
                                }
                            },
                        );
                    });

                    let item_num = num;
                    p.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(50.0, 50.0)),
                            color: Color::NONE,
                            ..Default::default()
                        },
                        Transform::from_xyz(width - 20.0, 0.0, 0.5),
                        Anchor::TOP_RIGHT,
                        Pickable::default(),
                        RemoveIcon,
                    ))
                    .with_child((
                        Text2d::new("✕"),
                        TextFont {
                            font: asset_server.load(FONTPATH),
                            font_size: 50.0,
                            ..Default::default()
                        },
                        Anchor::TOP_RIGHT,
                        Transform::from_xyz(-10.0, 10.0, 0.1),
                    ))
                    .observe(
                        move |_: On<Pointer<Click>>, mut writer: MessageWriter<RemoveType>| {
                            writer.write(RemoveType(item_num));
                        },
                    );
                });
            }
            let box_width = (width - 40.0) / 4.0;

            p.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(2.0 * box_width, 150.0)),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.9),
                    ..Default::default()
                },
                Transform::from_xyz(box_width, -160.0 * item_nums as f32 - 150.0 / 2.0, 0.5),
                Pickable::default(),
            ))
            .with_child((
                Text2d::new("+"),
                TextFont {
                    font: asset_server.load(FONTPATH),
                    font_size: 128.0,
                    ..Default::default()
                },
            ))
            .observe(
                |_: On<Pointer<Click>>,
                 mut custom_info: ResMut<'_, CustomInfo>,
                 asset_server: Res<'_, AssetServer>,
                 mut scroller: Single<&mut Scroller, With<TypeParent>>,
                 mut state: ResMut<NextState<SimState>>| {
                    if custom_info.len == 30 {
                        return;
                    }
                    let index = custom_info.len + 1;
                    state.set(SimState::ReSpawnChildren);
                    custom_info
                        .image_hash
                        .insert(index as i32, asset_server.load("rock.png"));
                    custom_info.nums.insert(index as i32, 35);
                    println!("SC: {:?}", scroller.height);
                    scroller.height += 160.0;
                    custom_info.len += 1;
                },
            );

            p.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(box_width, 150.0)),
                    color: Color::srgba(0.0, 0.0, 1.0, 0.9),
                    ..Default::default()
                },
                Transform::from_xyz(
                    2.5 * box_width + 10.0,
                    -160.0 * item_nums as f32 - 150.0 / 2.0,
                    0.5,
                ),
                Pickable::default(),
            ))
            .with_child((
                Text2d::new("⮎"),
                TextFont {
                    font: asset_server.load(FONTPATH),
                    font_size: 128.0,
                    ..Default::default()
                },
                Transform::from_xyz(0.0, 0.0, 0.5),
            ))
            .observe(
                |_: On<Pointer<Click>>,
                 mut custom_info: ResMut<CustomInfo>,
                 mut state: ResMut<NextState<SimState>>| {
                    let mut rng = rand::rng();
                    state.set(SimState::ReSpawnChildren);
                    let keys: Vec<i32> = custom_info.image_hash.keys().cloned().collect();
                    let image: Vec<Handle<Image>> = keys
                        .iter()
                        .map(|k| custom_info.image_hash[k].clone())
                        .collect();
                    let nums: Vec<i32> = keys.iter().map(|k| custom_info.nums[k].clone()).collect();

                    let mut index: Vec<usize> = (0..keys.len()).collect();
                    index.shuffle(&mut rng);

                    for (new_i, old_i) in index.iter().enumerate() {
                        custom_info
                            .image_hash
                            .insert(keys[new_i], image[*old_i].clone());
                        custom_info.nums.insert(keys[new_i], nums[*old_i]);
                    }
                },
            );
            p.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(box_width, 150.0)),
                    color: Color::srgba(1.0, 0.0, 0.0, 0.9),
                    ..Default::default()
                },
                Transform::from_xyz(
                    3.5 * box_width + 20.0,
                    -160.0 * item_nums as f32 - 150.0 / 2.0,
                    0.5,
                ),
                Pickable::default(),
            ))
            .with_child((
                Text2d::new("↺"),
                TextFont {
                    font: asset_server.load(FONTPATH),
                    font_size: 128.0,
                    ..Default::default()
                },
                Transform::from_xyz(0.0, 10.0, 0.5),
            ))
            .observe(
                |_: On<Pointer<Click>>,
                 custom_info: ResMut<CustomInfo>,
                 mut resize: ResMut<IsResize>,
                 asset_server: Res<AssetServer>| {
                    resize.0 = true;
                    custom_info_reset(custom_info, asset_server);
                },
            );
        });
    }
}

fn remove_type(
    mut msg: MessageReader<RemoveType>,
    mut state: ResMut<NextState<SimState>>,
    mut custom_info: ResMut<CustomInfo>,
) {
    for event in msg.read() {
        println!("M: {:?}", event.0);

        custom_info.image_hash.remove(&event.0);
        custom_info.nums.remove(&event.0);

        for i in (event.0 + 1)..=custom_info.len {
            let i_value = custom_info.image_hash[&i].clone();
            custom_info.image_hash.insert(i - 1, i_value);
            let n_value = custom_info.nums[&i].clone();
            custom_info.nums.insert(i - 1, n_value);
        }

        custom_info.len -= 1;
        state.set(SimState::ReSpawnChildren);
    }
}
