use bevy::prelude::*;
use std::collections::HashMap;
use crate::{map::{LoadMap, MapCoordinate, MapTile, MapTileType}, province::{ProvinceInfo, ProvinceInfos}};
use crate::time::*;
use crate::probability::*;
use crate::stage::*;

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EconomicGood {
    Grain,
}

pub trait Population {
    fn alive(&self) -> usize;
    fn set_alive(&mut self, alive: usize);
}

pub struct FarmerPopulation {
    pub alive: usize
}

impl Population for FarmerPopulation {
    fn alive(&self) -> usize {
        self.alive
    }

    fn set_alive(&mut self, alive: usize) {
        self.alive = alive;
    }
}

pub trait EconomicResource {
    fn target(&self) -> f64;
    fn current(&self) -> f64;
    fn product(&self) -> EconomicGood;
}

pub struct FarmingResource {
    pub target: f64,
    pub current: f64,
}

impl EconomicResource for FarmingResource {
    fn target(&self) -> f64 {
        self.target
    }

    fn current(&self) -> f64 {
        self.current
    }

    fn product(&self) -> EconomicGood {
        EconomicGood::Grain
    }
}

pub struct GoodsStorage(pub HashMap<EconomicGood, f64>);

impl GoodsStorage {
    pub fn add_resources(&mut self, good: EconomicGood, amount: f64) {
        if let Some(mut current_res) = self.0.get_mut(&good) {
            *current_res += amount;
        } else {
            self.0.insert(good, amount);
        }
    }

    pub fn use_goods_deficit(&mut self, good: EconomicGood, amount: f64) -> Option<f64> {
        if let Some(current_res) = self.0.get_mut(&good) {
            *current_res -= amount;
            if *current_res < 0.0 {
                let amt = -*current_res;
                *current_res = 0.0;
                Some(amt)
            } else {
                None
            }
        } else {
            self.0.insert(good, amount);
            Some(amount)
        }
    }
}

// pub struct FarmingBundle {
//     farmer_population: FarmerPopulation,
//     farming_resource: FarmingResource,
//     goods_storage: GoodsStorage,
// }

pub fn economic_system<T: 'static + Population + Send + Sync, U: 'static + EconomicResource + Send + Sync>(
    mut econ_query: Query<(&T, &U, &mut GoodsStorage)>
) {
    for (pop, res, mut storage) in econ_query.iter_mut() {
        let goods_produced = pop.alive() as f64 * res.current();
        // println!("produced {} goods of type {:?}", goods_produced, res.product());
        let new_total = storage.0.get(&res.product()).unwrap_or(&0.0) + goods_produced;
        storage.0.insert(res.product(), new_total);
    }
}

pub fn demand_system<T: 'static + Population + Send + Sync, U: 'static + EconomicResource + Send + Sync>(
) {

}

pub struct PopGrowthEvent(Entity, usize);
pub struct PopGrowthEventSpawner {
    pop_ent: Entity,
}

impl EventSpawner for PopGrowthEventSpawner {
    type Event = PopGrowthEvent;

    fn spawn(&self) -> Self::Event {
        PopGrowthEvent(self.pop_ent, 10)
    }
}

// pub fn pop_growth_generator_system<T: 'static + Population + Send + Sync> (
//     pops_query: Query<(Entity, &T)>,
// ) {

// }
pub struct Culture {
    name: String,

}

pub struct CultureRef(String);
pub struct ReligionRef(String);
pub enum Class {
    Farmer,
    Laborer,
    Noble,
}
pub struct FarmingPop {
    resource: EconomicGood,
    resource_base_harvest: f64,
    harvest_date: DayOfYear,
}

pub struct BasePop {
    pub size: isize,
    pub growth: f64,
    pub culture: CultureRef,
    pub religion: ReligionRef,
    pub class: Class,
    pub factors: Vec<String>,
    pub resources: GoodsStorage,
    pub hunger: f64,
}

pub fn farmer_production_system(
    mut farmer_query: Query<(&mut BasePop, &FarmingPop, &MapCoordinate)>,
    pinfos: Res<ProvinceInfos>,
    date: Res<Date>,
) {
    if date.is_day {
        for (mut base_pop, farming_pop, coord) in farmer_query.iter_mut() {
            if date.days_after_doy(farming_pop.harvest_date) == 0 {
                println!("harvest");
                let total_harvest = 250.0 * pinfos.0.get(&coord).unwrap().fertility * base_pop.size as f64;
                base_pop.resources.add_resources(farming_pop.resource, total_harvest);

            }
        }
    }
}

// fn pop_growth_system(
//     mut pop_query: Query<(&mut BasePop, &MapCoordinate)>,
//     pinfos: Res<ProvinceInfos>,
//     date: Res<Date>,
// ) {
//     if date.is_month {
//         for (mut pop, coord) in pop_query.iter_mut() {
//             pop.size += 5;
//             pinfos.0.get_mut(&coord).unwrap().total_population += 5;
//         }
//     }
// }

fn pop_growth_system(
    mut pop_query: Query<(&mut BasePop, &MapCoordinate)>,
    pinfos: Res<ProvinceInfos>,
    date: Res<Date>,
) {
    if date.is_month {
        for (mut pop, coord) in pop_query.iter_mut() {
            let required_food = 16.0 * pop.size as f64;
            let mut pinfo = pinfos.0.get_mut(&coord).unwrap();
            if let Some(deficit) = pop.resources.use_goods_deficit(EconomicGood::Grain, required_food) {
                pop.hunger += deficit / pop.size as f64;
            } else {
                pop.hunger = 0.0;
            }
            let pop_growth = pop.size as f64 * (1.0 - pop.hunger / 10.0) / 600.0;
            pop.growth = pop_growth;
            let newpops = dev_mean_sample((pop.size as f64 / 1000.0).max(0.5), pop_growth).round() as isize;
            pop.size += newpops;
            pinfo.total_population += newpops;
        }
    }
}

pub struct SpawnedPops(bool);

pub fn spawn_pops(
    mut commands: Commands,
    tiles_query: Query<(&MapCoordinate, &MapTile)>,
    load_map: Res<LoadMap>,
    mut spawned_pops: ResMut<SpawnedPops>,
    pinfos: ResMut<ProvinceInfos>,
) {
    if load_map.0 != None || spawned_pops.0 {
        return;
    }
    println!("pop setup");
    for (coord, tile) in tiles_query.iter() {
        if tile.tile_type == MapTileType::Plains {
            println!("spawn pop for {:?}", coord);
            commands
                .spawn()
                .insert(BasePop {
                    size: 1000,
                    culture: CultureRef("Default".to_string()),
                    religion: ReligionRef("Default".to_string()),
                    class: Class::Farmer,
                    factors: Vec::new(),
                    resources: GoodsStorage(HashMap::new()),
                    hunger: 0.0,
                    growth: 0.0,
                })
                .insert(FarmingPop {
                    resource: EconomicGood::Grain,
                    harvest_date: DayOfYear {
                        day: 15,
                        month: 1,
                    },
                    resource_base_harvest: 250.0,
                })
                .insert(coord.clone());
            pinfos.0.insert(*coord, ProvinceInfo {
                total_population: 1000,
                fertility: 1.0,
            });
        } else {
            pinfos.0.insert(*coord, ProvinceInfo {
                total_population: 0,
                fertility: 0.0,
            });
        }
        spawned_pops.0 = true;
    }
}

pub struct PopPlugin;

impl Plugin for PopPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_stage_after(InitStage::LoadMap, InitStage::LoadPops, SystemStage::single_threaded())
            .add_system(farmer_production_system.system())
            .add_system(pop_growth_system.system())
            .add_system(spawn_pops.system())
            .insert_resource(SpawnedPops(false))
            ;

    }
}
