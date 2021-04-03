use bevy::prelude::*;
use std::collections::HashMap;
use super::probability::*;

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

pub fn pop_growth_generator_system<T: 'static + Population + Send + Sync> (
    pops_query: Query<(Entity, &T)>,
) {

}
