use std::{cell::RefCell, collections::HashMap, ops::{Deref, DerefMut}, sync::{Arc, Mutex}};

use bevy::{prelude::*};
use crate::{map::*, pops::BasePop, stage::*};

pub struct Provinces {
    dale_map: HashMap<ProvinceId, Province>,
    coord_map: HashMap<MapCoordinate, ProvinceId>,
    last_id: usize,
}


pub trait DaleCollection<T> {
    type IdType;
    fn new_id(&mut self) -> Self::IdType;
    fn get(&self, id: Self::IdType) -> Option<&T>;
    fn get_mut(&mut self, id: Self::IdType) -> Option<DalePtr<T>>;
    fn insert(&mut self, item: T) -> Self::IdType;
}

pub trait DaleId {
    fn new(id: usize) -> Self;
}

pub struct Province {
    pub total_population: isize,
    pub fertility: f64,
    pub settlements: Vec<Settlement>,
}

pub struct DalePtr<T>(RefCell<T>);

impl<T> DalePtr<T> {
    pub fn new(item: T) -> DalePtr<T> {
        Self(RefCell::new(item))
    }
}

impl<T> Deref for DalePtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.borrow().deref()
    }
}

impl<T> DerefMut for DalePtr<T> {
    fn deref_mut(&self) -> &Self::Target {
        self.0.borrow_mut().deref_mut()
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct ProvinceId(usize);

impl DaleCollection<Province> for Provinces {
    type IdType = ProvinceId;

    fn new_id(&mut self) -> Self::IdType {
        self.last_id += 1;
        ProvinceId(self.last_id)
    }

    fn get(&self, id: Self::IdType) -> Option<&Province> {
        self.dale_map.get(&id)
    }

    fn get_mut(&mut self, id: Self::IdType) -> Option<DalePtr<Province>> {
        self.dale_map.get_mut(&id).and_then(|province| Some(DalePtr::new(*province)))
    }

    fn insert(&mut self, item: Province) -> Self::IdType {
        let id = self.new_id();
        self.dale_map.insert(id, item);
        id
    }
}

impl Provinces {
    pub fn get_coord(&self, coord: &MapCoordinate) -> Option<&Province> {
        self.coord_map.get(coord).and_then(|pid| self.get(*pid))
    }
    pub fn get_coord_mut(&mut self, coord: &MapCoordinate) -> Option<DalePtr<Province>> {
        self.coord_map.get(coord)
                      .and_then(|pid| self.get_mut(*pid))
    }
    pub fn insert_at_coord(&mut self, coord: MapCoordinate, province: Province) -> ProvinceId {
        let id = self.insert(province);
        self.coord_map.insert(coord, id);
        id
    }
}

pub struct Settlement(usize);

pub struct SettlementInfo {

    pub pops: Vec<Entity>,
}

fn province_setup(
    provinces: Res<Provinces>,
    tile_query: Query<(&MapTile, &MapCoordinate)>,
    pop_query: Query<(&BasePop, &MapCoordinate)>,
) {
    println!("province setup");
    for (tile, coord) in tile_query.iter() {
        println!("add provinceInfo");
        provinces.insert_at_coord(*coord, Province {
            total_population: 0,
            fertility: tile.tile_type.base_fertility(),
            settlements: Vec::new(),
        });
    }
    for (base_pop, coord) in pop_query.iter() {
        provinces.get_coord_mut(coord).unwrap().total_population += base_pop.size;

    }
}

fn province_pop_tracking_system(
    pop_query: Query<(Entity, &BasePop, &MapCoordinate)>,
    provinces: Res<Provinces>,
) {
    // let mut reset = HashMap::new();
    // for (ent, pop, coord) in pop_query.iter() {
    //     let mut pinfo = provinces.get_coord_mut(coord).unwrap();
    //     if !reset.get(coord).unwrap_or(&false) {
    //         pinfo.total_population = 0;
    //         pinfo.pops = Vec::new();
    //         reset.insert(coord, true);
    //     }
    //     pinfo.total_population += pop.size;
    //     pinfo.pops.push(ent);
    // }
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
            .add_system(province_pop_tracking_system.system())
            .insert_resource(Provinces {
                last_id: 0,
                dale_map: HashMap::new(),
                coord_map: HashMap::new(),
            })
            ;
    }
}
