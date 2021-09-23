use bevy::{ecs::system::SystemParam, prelude::*};
use crate::prelude::*;
use crate::constant::DAY_LABEL;
use crate::pops::*;
use crate::map::*;
use crate::province::*;
use crate::factor::*;
use crate::stage::DayStage;
use crate::time::Date;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct District {
    pub terrain: Terrain,
    // 0.0-1.0
    pub forested: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Districts([District; 3]);

// For a hex with r=5, the area is ~65km2, 6500 hectares, up to 650 comfortable farms
#[derive(Debug, Clone)]
pub struct Settlement {
    pub name: String,
    pub population: isize,
}

pub struct SettlementPops(pub Vec<PopRef>);

impl SettlementPops {
    pub fn remove_pop(&mut self, pop: PopRef) {
        self.0.retain(|i| *i != pop);
    }

    pub fn add_pop(&mut self, pop: PopRef) {
        self.0.push(pop);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SettlementFactor {
    CarryingCapacity,
    EliteOwnership,
}

impl FactorType for SettlementFactor {
    fn base_decay(&self) -> FactorDecay {
        match *self {
            _ => FactorDecay::None,
        }
    }

    fn default_amount(&self) -> f32 {
        match self {
            _ => 0.0,
        }
    }
}

#[game_ref]
pub struct SettlementRef(pub Entity);


impl SettlementRef {
    pub fn carrying_capacity(&self, districts: &Districts) -> f32 {
        let mut cc = 0;
        for district in districts.0.iter() {
            cc += district.terrain.carrying_capacity();
        }
        cc as f32
    }

    pub fn add_pop(&self, world: &mut World, pop: PopRef) {
        world.get_mut::<SettlementPops>(self.0).unwrap().add_pop(pop);
    }
}

#[derive(Bundle)]
pub struct SettlementBundle {
    pub info: Settlement,
    pub pops: SettlementPops,
    pub province: ProvinceRef,
    pub polity: PolityRef,
    pub factors: Factors<SettlementFactor>,
    pub coordinate: MapCoordinate,
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
