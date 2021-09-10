#![allow(unused_variables)]
#![allow(unused_imports)]
extern crate bevy;
#[macro_use]
extern crate macros;
#[macro_use]
extern crate lazy_static;

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
pub mod settlement;
pub mod factor;
pub mod agent;
pub mod gameref;

pub mod prelude {
        pub use crate::{macros::GameRef, PopRef, map::MapCoordinate, settlement::SettlementRef, pops::{Pop, PopFactor, CultureRef, LanguageRef}, probability::individual_event, province::{Province, ProvinceRef}, stage::DayStage, time::{Date, CurrentDate}, gameref::GameRef};
        pub use crate::constant::DAY_LABEL;
        pub use crate::factor::{FactorType, Factors, Factored};
}

use agent::AgentPlugin;
use bevy::{core::FixedTimestep, diagnostic::{ FrameTimeDiagnosticsPlugin, DiagnosticsPlugin }, prelude::*, sprite::SpriteSettings};
use bevy_tilemap::prelude::TilemapDefaultPlugins;
use province::ProvincePlugin;
use settlement::SettlementPlugin;
// fuck yo namespace
use ui::*;
use map::*;
use tag::*;
use constant::*;
use pops::*;
use save::*;
use input::*;
use camera::*;
use time::{TimePlugin, day_run_criteria_system};
use stage::*;

pub use crate::{pops::PopRef, settlement::SettlementRef};

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
        // .add_system(camera_zoom_system.system())
        .add_system(map_editor_painting_system.system())
        .add_stage_after(
            CoreStage::Update,
            DayStage::Init,
            SystemStage::parallel()
                // .with_run_criteria(
                //     FixedTimestep::step(0.0001)
                //         // labels are optional. they provide a way to access the current
                //         // FixedTimestep state from within a system
                //         .with_label(DAY_TIMESTEP),
                // )
        )
        .add_stage_after(
            CoreStage::Update,
            DayStage::Main,
            SystemStage::parallel()
                .with_run_criteria(
                    FixedTimestep::step(0.001)
                        // labels are optional. they provide a way to access the current
                        // FixedTimestep state from within a system
                        .with_label(DAY_TIMESTEP),
                )
        )
        // .insert_resource(SpriteSettings { frustum_culling_enabled: true })
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapDefaultPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(TimePlugin)
        .add_plugin(AgentPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(PopPlugin)
        .add_plugin(SettlementPlugin)
        .add_plugin(ProvincePlugin)
        .add_plugin(DiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
