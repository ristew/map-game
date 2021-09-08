use bevy::{core::FixedTimestep, ecs::{component::Component, system::Command, world::EntityRef, system::SystemParam}, prelude::*};
use rand::{Rng, distributions::Slice, prelude::SliceRandom, random, thread_rng};
use rand_distr::Uniform;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use crate::{constant::{DAY_LABEL, DAY_TIMESTEP}, map::*, province::{Province, ProvinceMap, ProvinceRef}};
use crate::time::*;
use crate::probability::*;
use crate::stage::*;
use crate::factor::*;
use crate::settlement::*;

pub trait GameRef {
    type Factor: Eq + Hash;

    fn entity(&self) -> Entity;

    // fn get<'a, T>(&self, manager: &'a dyn EntityManager) -> &'a T where T: Component {
    //     manager.get_component::<T>(self.entity())
    // }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PopFactor {
    Prominence,
    Demand(GoodType),
    MigrationDesire,
}

impl FactorType for PopFactor {
    fn base_decay(&self) -> FactorDecay {
        match *self {
            PopFactor::Prominence => FactorDecay::Exponential(0.01),
            PopFactor::Demand(good) => FactorDecay::None,
            PopFactor::MigrationDesire => FactorDecay::None,
        }
    }
}


// #[derive(SystemParam, EntityManager)]
// pub struct PopManager<'a> {
//     entity_query: Query<'a, (&'static Pop, &'static FarmingPop, &'static MapCoordinate)>,
// }

#[derive(Bundle)]
pub struct PopBundle {
    pub base: Pop,
    pub province: ProvinceRef,
    pub culture: CultureRef,
    pub settlement: SettlementRef,
    pub polity: PolityRef,
    pub language: PopLanguage,
    pub storage: GoodStorage,
    pub factors: Factors<PopFactor>,
    pub kid_buffer: KidBuffer,
}

#[derive(GameRef, PartialEq, Eq, Copy, Clone, Debug)]
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


#[derive(Debug)]
pub struct KidBuffer(pub VecDeque<usize>);

impl KidBuffer {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn size(&self) -> usize {
        self.0.iter().fold(0, |acc, e| acc + e)
    }

    pub fn spawn(&mut self, babies: usize) -> usize {
        // println!("spawn babies {}", babies);
        self.0.push_front(babies);
        // println!("{:?}", self);
        if self.0.len() > 12 {
            self.0.pop_back().unwrap()
        } else {
            babies
        }
    }

    pub fn starve(&mut self) -> usize {
        let cohort = sample(3.0).abs().min(12.0) as usize;
        if self.0.len() > cohort {
            let cohort_size = self.0[cohort];
            let dead_kids = positive_isample(cohort_size / 20 + 2, cohort_size / 5 + 1);
            // println!("cohort {}, size {}, dead {}", cohort, cohort_size, dead_kids);
            self.0[cohort] = (cohort_size - dead_kids).max(0);
            cohort_size - self.0[cohort]
        } else {
            0
        }
    }
}
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


pub fn growth_system(
    date: Res<Date>,
    mut pop_query: Query<(&mut Pop, &mut KidBuffer)>,
) {
    if !date.is_year {
        return;
    }
    println!("growth_system {}", *date);
    for (mut pop, mut kb) in pop_query.iter_mut() {
        let babies = positive_isample(2, pop.size * 4 / 100);
        let deaths = positive_isample(2, pop.size / 50);

        let adults = kb.spawn(babies);
        pop.size += adults - deaths;
        if pop.size <= 0 {
            println!("dead pop!!");
        }
    }
}

pub fn harvest_system(
    date: Res<Date>,
    mut farming_pop_query: Query<(&Pop, &SettlementRef, &FarmingPop, &mut Factors<PopFactor>)>,
    settlement: Query<&Settlement>,
    settlement_factors: Query<&Factors<SettlementFactor>>,
) {
    if !date.is_year {
        return;
    }
    for (pop, &settlement_ref, farming_pop, mut pop_factors) in farming_pop_query.iter_mut() {
        let mut farmed_amount = pop.size as f32;
        let carrying_capacity = settlement_factors.get(settlement_ref.0).unwrap().factor(SettlementFactor::CarryingCapacity);
        let comfortable_limit = carrying_capacity / 2.0;
        let settlement_size = settlement.get(settlement_ref.0).unwrap().population;
        println!("size {} comf {}", settlement_size, comfortable_limit);
        if settlement_size as f32 > comfortable_limit {
            pop_factors.add(PopFactor::MigrationDesire, 0.2);
            // population pressure on available land, seek more
            // world.add_command(Box::new(PopSeekMigrationCommand {
            //     pop: pop.clone(),
            //     pressure: (pop_size / comfortable_limit).powi(2),
            // }))
        } else {
            pop_factors.add(PopFactor::MigrationDesire, -0.2);
        }
        if settlement_size as f32 > carrying_capacity {
            farmed_amount = carrying_capacity + (farmed_amount - carrying_capacity).sqrt();
        }
        if random::<f32>() > 0.9 {
            // println!("failed harvest! halving farmed goods");
            farmed_amount *= 0.7;
        }
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
    let target_len = (list.len() as f32 * chance).round() as usize;
    let mut targets = list.clone();
    targets.shuffle(&mut thread_rng());
    targets
        .iter()
        .take(target_len)
        .map(|t| t.to_owned())
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
        let pop_systems = SystemSet::new()
            .with_system(harvest_system.system().label(DAY_LABEL))
            .with_system(growth_system.system().label(DAY_LABEL));
            // .with_run_criteria(
            //     FixedTimestep::step(0.0001)
            //         // labels are optional. they provide a way to access the current
            //         // FixedTimestep state from within a system
            //         .with_label(DAY_TIMESTEP),
            // )
        app
            .add_startup_stage_after(InitStage::LoadMap, InitStage::LoadPops, SystemStage::single_threaded())
            .add_system_set_to_stage(DayStage::Main, pop_systems)
            // .add_system(pop_growth_system.system())
            // .add_system(spawn_pops.system())
            // .add_system(pop_migration_system.system())
            // .add_system_to_stage(FinishStage::Main, migration_event_system.system())
            // .insert_resource(SpawnedPops(false))
            // .add_event::<MigrationEvent>()
            ;

    }
}
