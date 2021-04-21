extern crate bevy;

pub mod ui;
pub mod tag;
pub mod map;
pub mod constant;
pub mod pops;
pub mod probability;
pub mod save;
pub mod input;
pub mod camera;
pub mod time;
pub mod province;
pub mod stage;

use bevy::{diagnostic::{ FrameTimeDiagnosticsPlugin, DiagnosticsPlugin }, prelude::*, sprite::SpriteSettings};
use province::ProvincePlugin;
// fuck yo namespace
use ui::*;
use map::*;
use tag::*;
use constant::*;
use pops::*;
use save::*;
use input::*;
use camera::*;
use time::TimePlugin;
use stage::*;

pub fn setup_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    let texture_handle = asset_server.load("hextiles.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 3, 23);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(TileTextureAtlas(texture_atlas_handle));
}


fn main() {
    App::build()
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_assets.system())
        .add_system(camera_movement_system.system())
        .add_system(tile_select_system.system())
        .add_system(tile_hold_pressed_system.system())
        // .add_system(camera_zoom_system.system())
        .add_system(map_editor_painting_system.system())
        .add_system(info_box_change_system.system())
        // .insert_resource(SpriteSettings { frustum_culling_enabled: true })
        .add_plugins(DefaultPlugins)
        .add_plugin(UiPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(PopPlugin)
        .add_plugin(ProvincePlugin)
        .add_plugin(TimePlugin)
        .add_plugin(DiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
