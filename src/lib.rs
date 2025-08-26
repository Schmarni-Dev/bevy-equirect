use std::collections::HashMap;

use bevy_app::{Plugin, PostUpdate};
use bevy_asset::{
    AssetEvent, AssetHandleProvider, AssetId, AssetPath, AssetServer, Assets, Handle,
    RenderAssetUsages,
};
use bevy_ecs::{
    event::EventReader,
    resource::Resource,
    system::ResMut,
    world::{FromWorld, World},
};
use bevy_image::{Image, TextureFormatPixelInfo};
use wgpu_types::{Extent3d, TextureViewDescriptor, TextureViewDimension};

use crate::convert::CubeSide;

pub mod convert;

pub struct EquirectangularPlugin;
impl Plugin for EquirectangularPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<EquirectManager>();
        app.add_systems(PostUpdate, apply_equirect);
    }
}

fn apply_equirect(
    mut images: ResMut<Assets<Image>>,
    mut manager: ResMut<EquirectManager>,
    mut reader: EventReader<AssetEvent<Image>>,
) {
    for event in reader.read() {
        if let AssetEvent::Added { id } = event
            && let Some(cubemap) = manager.handles.get(id)
        {
            let image = images.get(&cubemap.src).unwrap();
            let image = cubemap_from_equirectangular(image, cubemap.res);
            images.insert(&cubemap.dst, image);
        }
        if let AssetEvent::Unused { id } | AssetEvent::Removed { id } = event {
            manager.handles.remove(id);
        }
    }
}

#[derive(Resource)]
pub struct EquirectManager {
    asset_server: AssetServer,
    image_handle_provider: AssetHandleProvider,
    handles: HashMap<AssetId<Image>, EquirectCubemap>,
}
struct EquirectCubemap {
    src: Handle<Image>,
    dst: Handle<Image>,
    res: u32,
}
impl EquirectManager {
    pub fn load_equirect_as_cubemap<'a>(
        &mut self,
        path: impl Into<AssetPath<'a>>,
        res: u32,
    ) -> Handle<Image> {
        let src = self.asset_server.load(path);

        self.handles
            .entry(src.id())
            .or_insert_with(|| EquirectCubemap {
                src,
                dst: self.image_handle_provider.reserve_handle().typed::<Image>(),
                res,
            })
            .dst
            .clone()
    }
}
impl FromWorld for EquirectManager {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();
        let image_handle_provider = world.resource::<Assets<Image>>().get_handle_provider();
        Self {
            asset_server,
            image_handle_provider,
            handles: HashMap::new(),
        }
    }
}

pub fn cubemap_from_equirectangular(equirect: &Image, cubemap_res: u32) -> Image {
    let format = equirect.texture_descriptor.format;
    let face_size = cubemap_res * cubemap_res * format.pixel_size() as u32;
    let out_size = face_size * 6;
    let mut out = vec![0u8; out_size as usize];
    for face in CubeSide::ALL {
        let data = face.gen_face(
            equirect.width(),
            equirect.height(),
            equirect.data.as_ref().unwrap(),
            cubemap_res,
            format,
        );
        let index = (face_size * face.get_cubemap_index()) as usize;
        let index_end = index + face_size as usize;
        let s = &mut out[index..index_end];
        s.copy_from_slice(&data);
    }

    let mut image = Image::new(
        Extent3d {
            width: cubemap_res,
            height: cubemap_res,
            depth_or_array_layers: 6,
        },
        wgpu_types::TextureDimension::D2,
        out,
        format,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..Default::default()
    });
    image
}
