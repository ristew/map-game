use bevy::prelude::*;
use std::collections::HashMap;
use crate::map::{MapCoordinate, MapTile, MapTileType};
use crate::time::*;
use crate::probability::*;

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
        let base_amt: f64 = 0.0;
        let current_res = self.0.get(&good).unwrap_or(&base_amt);
        self.0.insert(good, current_res + amount);
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
    harvest_date: DayOfYear,
}

pub struct BasePop {
    size: usize,
    culture: CultureRef,
    religion: ReligionRef,
    class: Class,
    factors: Vec<String>,
    resources: GoodsStorage,
}

pub fn farmer_production_system(
    mut farmer_query: Query<(&mut BasePop, &FarmingPop, &MapCoordinate)>,
    current_date: Res<Date>,
) {
    for (mut base_pop, farming_pop, coord) in farmer_query.iter_mut() {
        if current_date.days_after_doy(farming_pop.harvest_date) == 0 {
            println!("harvest");
            base_pop.resources.add_resources(farming_pop.resource, 10.0);
        }
    }
}

fn setup_pops(
    mut commands: Commands,
    tiles_query: Query<(&MapCoordinate, &MapTile)>,
) {
    for (coord, tile) in tiles_query.iter() {
        if tile.tile_type == MapTileType::Land {
            commands
                .spawn()
                .insert(BasePop {
                    size: 100,
                    culture: CultureRef("Default".to_string()),
                    religion: ReligionRef("Default".to_string()),
                    class: Class::Farmer,
                    factors: Vec::new(),
                    resources: GoodsStorage(HashMap::new()),
                })
                .insert(FarmingPop {
                    resource: EconomicGood::Grain,
                    harvest_date: DayOfYear {
                        day: 15,
                        month: 1,
                    }
                })
                .insert(coord.clone());
        }
    }
}

pub struct PopPlugin;

impl Plugin for PopPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_system(setup_pops.system())
            .add_system(farmer_production_system.system())
            ;

    }
}
