use bevy::{ecs::system::SystemParam, prelude::*};
use crate::prelude::*;
use crate::constant::DAY_LABEL;
use crate::pops::*;
use crate::map::*;
use crate::province::*;
use crate::factor::*;
use crate::stage::DayStage;
use crate::time::Date;

pub struct Settlement {
    pub name: String,
    pub population: usize,
}

pub struct SettlementPops(pub Vec<PopRef>);

impl SettlementPops {
    pub fn remove_pop(&mut self, pop: PopRef) {
        self.0.retain(|i| *i != pop);
    }
}

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
    pub province: ProvinceRef,
    pub factors: Factors<SettlementFactor>,
}

fn settlement_info_system(
    date: Res<CurrentDate>,
    pop_query: Query<&Pop>,
    mut settlement_query: Query<(&mut Settlement, &SettlementPops)>,
) {
    if !date.is_month {
        return;
    }

    for (mut settlement, pops) in settlement_query.iter_mut() {
        let mut total_pop = 0;
        for pop_ref in pops.0.iter() {
            let pop = pop_query.get(pop_ref.0).unwrap();
            total_pop += pop.size;
        }
        settlement.population = total_pop;
    }
}

// #[derive(SystemParam, EntityManager)]
// pub struct SettlementManager<'a> {
//     entity_query: Query<'a, (&'static Settlement, &'static Pops, &'static MapCoordinate, &'static Factors<SettlementFactor>)>,
// }

pub struct SettlementPlugin;

impl Plugin for SettlementPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system_to_stage(DayStage::Main, settlement_info_system.system());
    }
}
