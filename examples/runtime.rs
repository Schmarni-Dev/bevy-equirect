use std::{env, path::PathBuf};

use bevy::{core_pipeline::Skybox, input::common_conditions::input_just_pressed, prelude::*};
use bevy_app::{App, AppExit, Startup};
use bevy_asset::{AssetPlugin, UnapprovedPathMode};
use bevy_ecs::system::{Commands, ResMut};
use bevy_equirect::{EquirectManager, EquirectangularPlugin};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            unapproved_path_mode: UnapprovedPathMode::Allow,
            ..Default::default()
        }))
        .add_plugins(EquirectangularPlugin)
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update.run_if(input_just_pressed(KeyCode::Space)))
        .run()
}

fn update(
    mut cmds: Commands,
    cam: Single<Entity, With<FlyCam>>,
    mut equirect: ResMut<EquirectManager>,
) {
    let args = env::args_os().nth(1).unwrap();
    let path = PathBuf::from(args);

    let image = equirect.load_equirect_as_cubemap(path, 1024);
    cmds.entity(*cam).insert(Skybox {
        image,
        brightness: 1000.0,
        ..Default::default()
    });
}

fn setup(mut cmds: Commands) {
    cmds.spawn((Camera3d::default(), FlyCam));
}
