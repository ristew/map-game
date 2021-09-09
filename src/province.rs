use std::{cell::RefCell, collections::HashMap, ops::{Deref, DerefMut}, sync::{Arc, Mutex}};
use std::fmt::Display;

use crate::prelude::*;
use bevy::{ecs::system::{Command, SystemParam}, prelude::*};
use crate::{map::*, pops::*, settlement::Settlements, stage::*};
use crate::factor::*;

#[derive(GameRef, Debug, Copy, Clone)]
pub struct ProvinceRef(pub Entity);

#[derive(Debug)]
pub struct Province {
    pub total_population: usize,
    pub fertility: f64,
}

pub struct ProvincePops(pub Vec<Entity>);
pub struct ProvinceSettlements(pub Vec<SettlementRef>);

pub struct ProvinceMap(pub HashMap<MapCoordinate, ProvinceRef>);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ProvinceFactor {

}

impl FactorType for ProvinceFactor {
    fn base_decay(&self) -> FactorDecay {
        FactorDecay::Linear(0.001)
    }
}

// #[derive(SystemParam, EntityManager)]
// pub struct ProvinceManager<'a> {
//     entity_query: Query<'a, (&'static Province, &'static Settlements, &'static MapCoordinate, &'static Factors<ProvinceFactorType>)>,
// }

pub struct ResetProvinceMap;

impl Command for ResetProvinceMap {
    fn write(self: Box<Self>, world: &mut World) {
        let mut inner = HashMap::new();
        let mut query = world.query::<(Entity, &Province, &MapCoordinate)>();
        for (ent, _, &coord) in query.iter(&world) {
            inner.insert(coord, ProvinceRef(ent));
        }
        let mut provinces = world.get_resource_mut::<ProvinceMap>().unwrap();

        provinces.0 = inner;
    }
}

fn province_pop_tracking_system(
    mut commands: Commands,
    load_map: Res<LoadMap>,
    pop_query: Query<(Entity, &Pop, &ProvinceRef)>,
    mut province_query: Query<(&mut Province, &mut ProvincePops)>,
) {
    if load_map.0 != None {
        return;
    }
    for (mut province, mut pops) in province_query.iter_mut() {
        province.total_population = 0;
        pops.0 = vec![];
    }
    for (ent, pop, pop_province) in pop_query.iter() {
        let (mut province, mut pops) = province_query.get_mut(pop_province.entity()).unwrap();
        province.total_population += pop.size;
        pops.0.push(ent);
    }
}

pub enum ProvinceModifier {
    RichSoil, // +50% fertility
    RockySoil, // -50% fertility
    Alluvial, // +100% fertility!!
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Terrain {
    Plains,
    Hills,
    Mountains,
    Desert,
    Marsh,
    Forest,
    Ocean,
}

impl Display for Terrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_str())
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Self::Hills
    }
}

// impl Factored for Terrain {
//     fn factor(&self, world: &World, factor: FactorType) -> Option<Factor> {
//         match factor {
//             FactorType::CarryingCapacity => Some(match *self {
//                 Terrain::Plains => Factor::factor(1.0),
//                 Terrain::Hills => Factor::factor(0.7),
//                 Terrain::Mountains => Factor::factor(0.2),
//                 Terrain::Desert => Factor::factor(0.1),
//                 Terrain::Marsh => Factor::factor(0.5),
//                 Terrain::Forest => Factor::factor(0.5),
//                 Terrain::Ocean => Factor::factor(0.0),
//             }),
//             FactorType::SettlementRating => Some(match *self {
//                 Terrain::Plains => Factor::factor(1.0),
//                 // slightly prefer hills for defensibility
//                 Terrain::Hills => Factor::factor(1.1),
//                 Terrain::Mountains => Factor::factor(0.2),
//                 Terrain::Desert => Factor::factor(0.1),
//                 Terrain::Marsh => Factor::factor(0.5),
//                 Terrain::Forest => Factor::factor(0.5),
//                 Terrain::Ocean => Factor::factor(0.0),
//             }),
//         }
//     }
// }

// impl Terrain {
//     pub fn color(self) -> Color {
//         match self {
//             Terrain::Plains => Color::new(0.5, 0.9, 0.5, 1.0),
//             Terrain::Hills => Color::new(0.4, 0.7, 0.4, 1.0),
//             Terrain::Mountains => Color::new(0.5, 0.5, 0.3, 1.0),
//             Terrain::Desert => Color::new(1.0, 1.0, 0.8, 1.0),
//             Terrain::Marsh => Color::new(0.3, 0.6, 0.6, 1.0),
//             Terrain::Forest => Color::new(0.2, 0.7, 0.3, 1.0),
//             Terrain::Ocean => Color::new(0.1, 0.4, 0.7, 1.0),
//         }
//     }
// }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Climate {
    Tropical,
    Dry,
    Mild,
    Cold,
}

// impl Factored for Climate {
//     fn factor(&self, world: &World, factor: FactorType) -> Option<Factor> {
//         match factor {
//             FactorType::CarryingCapacity => Some(match *self {
//                 Climate::Tropical => Factor::factor(1.2),
//                 Climate::Dry => Factor::factor(0.7),
//                 Climate::Mild => Factor::factor(1.0),
//                 Climate::Cold => Factor::factor(0.7),
//             }),
//             FactorType::SettlementRating => Some(match *self {
//                 Climate::Tropical => Factor::factor(0.8),
//                 Climate::Dry => Factor::factor(0.6),
//                 Climate::Mild => Factor::factor(1.0),
//                 Climate::Cold => Factor::factor(0.8),
//             }),
//             _ => None,
//         }
//     }
// }

impl Default for Climate {
    fn default() -> Self {
        Self::Mild
    }
}

pub struct ProvincePlugin;

impl Plugin for ProvincePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_stage_after(InitStage::LoadPops, InitStage::LoadProvinces, SystemStage::single_threaded())
            .add_system(province_pop_tracking_system.system())
            // .insert_resource(Provinces {
            //     last_id: 0,
            //     dale_map: HashMap::new(),
            //     coord_map: HashMap::new(),
            // })
            .insert_resource(ProvinceMap(HashMap::new()))
            ;
    }
}
