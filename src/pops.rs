use bevy::{ecs::{component::Component, system::Command, world::EntityRef, system::SystemParam}, prelude::*};
use rand::{Rng, distributions::Slice, thread_rng};
use rand_distr::Uniform;
use std::collections::HashMap;
use std::hash::Hash;
use crate::{map::{HexMap, LoadMap, MapCoordinate, MapTile, MapTileType}, province::{Province, ProvinceMap, ProvinceRef}};
use crate::time::*;
use crate::probability::*;
use crate::stage::*;
use crate::settlement::*;

pub trait GameRef {
    type Factor: Eq + Hash;

    fn entity(&self) -> Entity;

    // fn get<'a, T>(&self, manager: &'a dyn EntityManager) -> &'a T where T: Component {
    //     manager.get_component::<T>(self.entity())
    // }
}

pub trait FactorType {
    fn base_decay(&self) -> FactorDecay;
}

pub enum FactorDecay {
    Linear(f32),
    Exponential(f32),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PopFactor {
    Prominence,
    Demand(GoodType),
}

impl FactorType for PopFactor {
    fn base_decay(&self) -> FactorDecay {
        match *self {
            PopFactor::Prominence => FactorDecay::Exponential(0.01),
            PopFactor::Demand(good) => FactorDecay::None,
        }
    }
}

pub struct Factor<T> where T: FactorType {
    ftype: T,
    amount: f32,
    target: f32,
    decay: FactorDecay,
}

impl<T> Factor<T> where T: FactorType + Eq + Hash + Copy {
    pub fn decay(&mut self) -> f32 {
        let this_decay = match self.decay {
            FactorDecay::Linear(n) => n,
            FactorDecay::Exponential(n) => (self.amount - self.target) * n,
            FactorDecay::None => 0.0,
        };
        self.amount = (self.amount - this_decay).max(self.target);
        this_decay
    }

    pub fn add(&mut self, amt: f32) {
        self.amount += amt;
    }

    fn base(ftype: T) -> Self {
        Self {
            ftype,
            amount: 0.0,
            target: 0.0,
            decay: ftype.base_decay(),
        }
    }
}

pub struct Factors<T> where T: FactorType + Eq + Hash {
    pub inner: HashMap<T, Factor<T>>,
}


impl<T> Factors<T> where T: FactorType + Eq + Hash + Copy {
    pub fn decay(&mut self) {
        for factor in self.inner.values_mut() {
            factor.decay();
        }
    }

    pub fn add(&mut self, ftype: T, amt: f32) {
        if !self.inner.contains_key(&ftype) {
            self.inner.insert(ftype, Factor {
                ftype,
                amount: 0.0,
                target: 0.0,
                decay: ftype.base_decay(),
            });
        }

        self.inner.get_mut(&ftype).unwrap().amount += amt;
    }

    pub fn factor(&self, ftype: T) -> f32 {
        self.inner.get(&ftype).map(|f| f.amount).unwrap_or(0.0)
    }
}

pub trait EntityManager<R> where R: GameRef {
    fn get_component<T>(&self, ent: R) -> &T where T: Component;
    fn get_factor(&self, entity: R, factor: R::Factor) -> f32;
}

pub trait EntityManagerType {
    type Ref: GameRef;
}

// #[derive(SystemParam, EntityManager)]
// pub struct PopManager<'a> {
//     entity_query: Query<'a, (&'static Pop, &'static FarmingPop, &'static MapCoordinate)>,
// }

#[derive(Bundle)]
pub struct PopBundle {
    pub base: Pop,
    pub farming: Option<FarmingPop>,
    pub province: ProvinceRef,
    pub culture: CultureRef,
    pub polity: PolityRef,
    pub language: PopLanguage,
    pub storage: GoodStorage,
    pub factors: Factors<PopFactor>,
}

#[derive(GameRef)]
pub struct PopRef(pub Entity);


// pub type PopQuery<'w> = Query<'w, (&'w Pop, &'w FarmingPop, &'w MapCoordinate)>;

pub struct Pop {
    pub size: usize,
}

pub struct PopLanguage {
    pub language: LanguageRef,
    pub drift: f32,
}

pub struct Pops(pub Vec<Entity>);

pub struct FarmingPop {
    pub good: GoodType,
}

pub struct PopProvince(pub Entity);
pub struct PopCulture(pub Entity);
pub struct PopPolity(pub Entity);

// #[derive(GameRef<'w>)]
// pub struct CultureRef<'w>(pub Entity);

// pub type CultureQuery<'w> = Query<'w, (&'w Culture,)>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CultureFactor {

}

#[derive(GameRef)]
pub struct CultureRef(pub Entity);
pub struct Culture {
    pub name: String,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PolityFactor {

}
#[derive(GameRef)]
pub struct PolityRef(pub Entity);

// pub type PolityQuery<'w> = Query<'w, (&'w Polity)>;

pub struct Polity {
    pub name: String,
}



pub fn harvest_system(
    farming_pop_query: Query<(&Pop, &FarmingPop, &SettlementRef)>,
    settlement: Query<&Settlement>,
    settlement_factors: Query<&Factors<SettlementFactor>>,
) {
    for (pop, farming_pop, &settlement_ref) in farming_pop_query.iter() {
        let mut farmed_amount = pop.size as f32;
        let carrying_capacity = settlement_factors.get(settlement_ref.0).unwrap().factor(SettlementFactor::CarryingCapacity);
        let comfortable_limit = carrying_capacity / 2.0;
        let settlement_size = settlement.get(settlement_ref.0).unwrap().population;
        if settlement_size as f32 > comfortable_limit {
            // population pressure on available land, seek more
            // world.add_command(Box::new(PopSeekMigrationCommand {
            //     pop: pop.clone(),
            //     pressure: (pop_size / comfortable_limit).powi(2),
            // }))
        }
        if settlement_size as f32 > carrying_capacity {
            farmed_amount = carrying_capacity + (farmed_amount - carrying_capacity).sqrt();
        }
        // if random::<f32>() > 0.9 {
        //     // println!("failed harvest! halving farmed goods");
        //     farmed_amount *= 0.7;
        // }
        // world.add_command(Box::new(SetGoodsCommand {
        //     good_type: farmed_good,
        //     amount: farmed_amount * 300.0,
        //     pop: pop.clone(),
        // }));
    }
}

pub struct GoodStorage(pub HashMap<GoodType, f32>);

impl GoodStorage {
    pub fn amount(&self, good: GoodType) -> f32 {
        *self.0.get(&good).unwrap_or(&0.0)
    }

    pub fn consume(&mut self, good: GoodType, amount: f32) -> Option<f32> {
        if let Some(mut stored) = self.0.get_mut(&good) {
            if *stored < amount {
                let deficit = amount - *stored;
                *stored = 0.0;
                Some(deficit)
            } else {
                *stored -= amount;
                None
            }
        } else {
            Some(amount)
        }
    }

    pub fn add(&mut self, good: GoodType, amount: f32) -> f32 {
        if let Some(stored) = self.0.get_mut(&good) {
            *stored += amount;
            *stored
        } else {
            self.0.insert(good, amount);
            amount
        }
    }

    pub fn set(&mut self, good: GoodType, amount: f32) {
        *self.0.get_mut(&good).unwrap() = amount;
    }

    // pub fn try_eat_diet(&self, diet: Diet) -> Vec<(GoodType, f32)> {
    //     let mut bad_res = Vec::new();

    //     for part in diet.0.iter() {
    //         if self.amount(part.0) < part.1 {
    //             bad_res.push(*part);
    //         }
    //     }

    //     bad_res
    // }
    //
}

pub struct Language {
    pub name: String,
    pub vowels: Vec<String>,
    pub initial_consonants: Vec<String>,
    pub middle_consonants: Vec<String>,
    pub end_consonants: Vec<String>,
}

pub struct LanguageRef(pub Entity);

fn list_filter_chance(list: &Vec<String>, chance: f32) -> Vec<String> {
    list.iter()
        .filter_map(|v| {
            if rand::random::<f32>() < chance {
                Some(v.clone())
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
}

fn map_string(list: Vec<&str>) -> Vec<String> {
    list.iter()
        .map(|v| String::from(*v))
        .collect::<Vec<String>>()
}

pub fn sample_list(list: &Vec<String>) -> String {
    rand::thread_rng().sample(Slice::new(list).unwrap()).clone()
}

impl Language {
    pub fn new() -> Self {
        let vowel_chance = 0.75;
        let vowels = list_filter_chance(
            &map_string(vec![
                "a", "ae", "e", "i", "ei", "u", "o", "oi", "au", "ou", "ee", "ea", "oa",
            ]),
            0.75,
        );
        let consonants = list_filter_chance(
            &map_string(vec![
                "b", "c", "d", "f", "g", "h", "j", "k", "l", "m", "n", "p", "r", "s", "t", "v",
                "w", "z", "ss", "th", "st", "ch", "sh",
            ]),
            0.75,
        );

        let initial_consonants = list_filter_chance(&consonants, 0.50);
        let middle_consonants = list_filter_chance(&consonants, 0.75);
        let end_consonants = list_filter_chance(&consonants, 0.50);

        let mut newlang = Self {
            name: "".to_owned(),
            vowels,
            initial_consonants,
            middle_consonants,
            end_consonants,
        };

        newlang.name = newlang.generate_name(2);
        newlang
    }

    pub fn maybe_vowel(&self, chance: f32) -> Option<String> {
        if rand::random::<f32>() < chance {
            Some(sample_list(&self.vowels))
        } else {
            None
        }
    }

    pub fn generate_name(&self, max_middle: usize) -> String {
        let mut name: String = String::new();
        name += &self.maybe_vowel(0.3).unwrap_or("".to_owned());
        name += &sample_list(&self.initial_consonants);
        for i in 0..thread_rng().sample(Uniform::new(0, max_middle)) {
            name += &sample_list(&self.vowels);
            name += &sample_list(&self.middle_consonants);
        }
        name += &sample_list(&self.vowels);
        name += &sample_list(&self.end_consonants);
        name += &self.maybe_vowel(0.3).unwrap_or("".to_owned());
        name
        // to_title_case(name.as_str())
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum GoodType {
    Wheat,
    Barley,
    OliveOil,
    Fish,
    Wine,
    Iron,
    Copper,
    Tin,
    Bronze,
    Silver,
    Gold,
    Lead,
    Salt,
    PurpleDye,
    Marble,
    Wood,
    Textiles,
    LuxuryClothes,
    Slaves, // ?? how to handle
}

use GoodType::*;
lazy_static! {
    pub static ref FOOD_GOODS: Vec<GoodType> = vec![Wheat, Barley, Fish, OliveOil, Salt, Wine,];
}

#[derive(PartialEq)]
pub struct Satiety {
    pub base: f32,
    pub luxury: f32,
}

impl std::ops::Add for Satiety {
    type Output = Satiety;

    fn add(self, rhs: Self) -> Self::Output {
        Satiety {
            base: self.base + rhs.base,
            luxury: self.luxury + rhs.luxury,
        }
    }
}

impl std::ops::AddAssign for Satiety {
    fn add_assign(&mut self, rhs: Self) {
        *self = Satiety {
            base: self.base + rhs.base,
            luxury: self.luxury + rhs.luxury,
        };
    }
}

impl std::ops::Mul<Satiety> for f32 {
    type Output = Satiety;

    fn mul(self, rhs: Satiety) -> Self::Output {
        Satiety {
            base: rhs.base * self,
            luxury: rhs.luxury * self,
        }
    }
}

pub enum ConsumableGoodCatagory {
    Tier1,
    Tier2,
    Tier3,
}

impl GoodType {
    pub fn base_satiety(&self) -> Satiety {
        match *self {
            Wheat => Satiety {
                base: 3300.0,
                luxury: 0.1,
            },
            Barley => Satiety {
                base: 3300.0,
                luxury: 0.0,
            },
            OliveOil => Satiety {
                base: 8800.0,
                luxury: 0.3,
            },
            Fish => Satiety {
                base: 1500.0,
                luxury: 0.2,
            },
            Wine => Satiety {
                base: 500.0,
                luxury: 1.0,
            },
            _ => Satiety {
                base: 0.0,
                luxury: 0.0,
            },
        }
    }

    pub fn max_consumed_monthly_per_capita(&self) -> f32 {
        match *self {
            Wheat => 22.5, // 3300 calories per kg at 2500 calories per day = 0.75 kg/day, I'm bad at math
            Barley => 22.5,
            OliveOil => 3.0,
            Fish => 30.0, // a kg of fish a day, the life...
            Wine => 10.0, // ~ half a bottle a day
            _ => 0.0,
        }
    }

    pub fn consumable_good_catagory(&self) -> Option<ConsumableGoodCatagory> {
        match *self {
            Wheat => Some(ConsumableGoodCatagory::Tier3),
            Barley => Some(ConsumableGoodCatagory::Tier3),
            OliveOil => Some(ConsumableGoodCatagory::Tier2),
            Fish => Some(ConsumableGoodCatagory::Tier2),
            Wine => Some(ConsumableGoodCatagory::Tier1),
            _ => None,
        }
    }
}


pub struct PopPlugin;

impl Plugin for PopPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_stage_after(InitStage::LoadMap, InitStage::LoadPops, SystemStage::single_threaded())
            .add_system(harvest_system.system())
            // .add_system(pop_growth_system.system())
            // .add_system(spawn_pops.system())
            // .add_system(pop_migration_system.system())
            // .add_system_to_stage(FinishStage::Main, migration_event_system.system())
            // .insert_resource(SpawnedPops(false))
            // .add_event::<MigrationEvent>()
            ;

    }
}
