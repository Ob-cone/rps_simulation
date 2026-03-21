use std::{
    collections::HashMap,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

use bevy::{
    app::{App, PreStartup, Update},
    asset::{AssetServer, Assets, Handle, RenderAssetUsages},
    ecs::{
        resource::Resource,
        system::{Res, ResMut},
    },
    image::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Debug, Resource)]
pub struct CustomInfo {
    pub size: f32,
    pub image_hash: HashMap<i32, Handle<Image>>,
}

impl Default for CustomInfo {
    fn default() -> Self {
        Self {
            size: 1.0,
            image_hash: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ReadImage(pub i32, pub Vec<u8>);
#[derive(Debug, Resource)]
pub struct ImageChannel(pub Sender<ReadImage>, pub Mutex<Receiver<ReadImage>>);
pub fn custom_plugin(app: &mut App) {
    let (tx, rx) = channel::<ReadImage>();

    app.insert_resource(CustomInfo::default())
        .insert_resource(ImageChannel(tx, Mutex::new(rx)))
        .add_systems(PreStartup, custom_info_reset)
        .add_systems(Update, add_image);
}

fn custom_info_reset(mut custom_info: ResMut<CustomInfo>, asset_server: Res<AssetServer>) {
    custom_info.size = 2.0;
    custom_info.image_hash.clear();
    custom_info
        .image_hash
        .insert(1, asset_server.load("rock.png"));
    custom_info
        .image_hash
        .insert(2, asset_server.load("paper.png"));
    custom_info
        .image_hash
        .insert(3, asset_server.load("scissors.png"));
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
