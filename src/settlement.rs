use bevy::{ecs::system::SystemParam, prelude::*};
use crate::pops::*;
use crate::map::*;
use crate::province::*;

pub struct Settlement {
    pub name: String,
    pub population: usize,
}

pub struct SettlementPops(pub Vec<Entity>);

pub struct SettlementFactorType;

#[derive(GameRef)]
pub struct SettlementRef(pub Entity);


impl SettlementRef {
    pub fn carrying_capacity(&self) -> f32 {
        100.0
    }

    pub fn population(self, sm: &SettlementManager, pm: &PopManager) -> usize {
        let mut total_pop = 0;
        for &pop_ref in sm.get_component::<Pops>(self).0.iter() {
            total_pop += pm.get_component::<Pop>(pop_ref).size;
        }
        total_pop
    }
}

pub struct Settlements(pub Vec<SettlementRef>);

pub struct SettlementBundle {
    info: Settlement,
    pops: SettlementPops,
    factors: Factors<SettlementFactorType>,
}

#[derive(SystemParam, EntityManager)]
pub struct SettlementManager<'a> {
    entity_query: Query<'a, (&'static Settlement, &'static Pops, &'static MapCoordinate, &'static Factors<SettlementFactorType>)>,
}
