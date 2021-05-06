use bevy::prelude::*;
use dashmap::DashMap;
use crate::{map::*, pops::BasePop, stage::*};

pub struct ProvinceInfos(pub DashMap<MapCoordinate, ProvinceInfo>);

pub struct ProvinceInfo {
    pub total_population: isize,
    pub fertility: f64,
}

fn province_setup(
    province_infos: Res<ProvinceInfos>,
    tile_query: Query<(&MapTile, &MapCoordinate)>,
    pop_query: Query<(&BasePop, &MapCoordinate)>,
) {
    println!("province setup");
    for (tile, coord) in tile_query.iter() {
        province_infos.0.insert(*coord, ProvinceInfo {
            total_population: 0,
            fertility: tile.tile_type.base_fertility(),
        });
    }
    for (base_pop, coord) in pop_query.iter() {
        province_infos.0.get_mut(&coord).unwrap().total_population += base_pop.size;
    }
}

fn province_pop_tracking_system(
    pop_changed_query: Query<(&BasePop, &MapCoordinate), Changed<BasePop>>,
    province_infos: Res<ProvinceInfos>,
) {
}

pub enum ProvinceModifier {
    RichSoil, // +50% fertility
    RockySoil, // -50% fertility
    Alluvial, // +100% fertility!!
}

pub struct ProvincePlugin;

impl Plugin for ProvincePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_stage_after(InitStage::LoadPops, InitStage::LoadProvinces, SystemStage::single_threaded())
            .add_startup_system_to_stage(InitStage::LoadProvinces, province_setup.system())
            .insert_resource(ProvinceInfos(DashMap::new()))
            ;
    }
}
