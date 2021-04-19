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

use bevy::{
    prelude::*,
    diagnostic::{ FrameTimeDiagnosticsPlugin, DiagnosticsPlugin },
};
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
    let world_setup = SystemStage::single(load_map_system.system());

    App::build()
        .add_startup_system(setup_assets.system())
        .add_startup_stage("world_setup", world_setup)
        .add_system(camera_movement_system.system())
        .add_system(camera_view_check.system())
        .add_system(tile_select_system.system())
        .add_system(tile_hold_pressed_system.system())
        .add_system(change_zoom_system.system())
        // .add_system(camera_zoom_system.system())
        .add_system(map_editor_painting_system.system())
        .add_system(economic_system::<FarmerPopulation, FarmingResource>.system())
        .add_plugins(DefaultPlugins)
        .add_plugin(UiPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(TimePlugin)
        .add_plugin(DiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
