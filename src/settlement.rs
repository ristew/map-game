use bevy::{ecs::system::SystemParam, prelude::*};
use crate::pops::*;
use crate::map::*;
use crate::province::*;
use crate::factor::*;

pub struct Settlement {
    pub name: String,
    pub population: usize,
}

pub struct SettlementPops(pub Vec<PopRef>);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SettlementFactor {
    CarryingCapacity
}

impl FactorType for SettlementFactor {
    fn base_decay(&self) -> FactorDecay {
        match *self {
            _ => FactorDecay::None,
        }
    }
}

#[derive(GameRef, Copy, Clone, Debug)]
pub struct SettlementRef(pub Entity);


impl SettlementRef {
    pub fn carrying_capacity(&self) -> f32 {
        100.0
    }

    pub fn add_pop(&self, world: &mut World, pop: PopRef) {
        world.get_mut::<SettlementPops>(self.0).unwrap().0.push(pop);
    }
}

pub struct Settlements(pub Vec<SettlementRef>);

#[derive(Bundle)]
pub struct SettlementBundle {
    pub info: Settlement,
    pub pops: SettlementPops,
    pub factors: Factors<SettlementFactor>,
}

// #[derive(SystemParam, EntityManager)]
// pub struct SettlementManager<'a> {
//     entity_query: Query<'a, (&'static Settlement, &'static Pops, &'static MapCoordinate, &'static Factors<SettlementFactor>)>,
// }
