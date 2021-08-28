use bevy::{ecs::{component::Component, system::Command, world::EntityRef}, prelude::*};
use rand::{Rng, distributions::Slice, thread_rng};
use rand_distr::Uniform;
use std::collections::HashMap;
use crate::{map::{HexMap, LoadMap, MapCoordinate, MapTile, MapTileType}, modifier::{self, ModifierStorage, ModifierType}, province::{Province, Provinces}};
use crate::time::*;
use crate::probability::*;
use crate::stage::*;

pub trait GameRef {
    fn get<T>(&self, world: &World) -> &T where T: Component;
}

pub enum SettlementFactorType {
    CarryingCapacity,
}

pub enum FactorDecay {
    Linear(f32),
    Exponential(f32),
    None,
}

pub struct Factor<T> {
    ftype: T,
    amount: f32,
    decay: FactorDecay,
}

impl<T> Factor<T> {
    pub fn decay(&mut self) -> f32 {
        let this_decay = match self.decay {
            FactorDecay::Linear(n) => n,
            FactorDecay::Exponential(n) => self.amount * n,
            FactorDecay::None => 0.0,
        };
        self.amount = (self.amount - this_decay).max(0.0);
        this_decay
    }

    pub fn add(&mut self, amt: f32) {
        self.amount += amt;
    }
}

pub struct Factors<T> {
    inner: HashMap<T, Factor<T>>,
}

impl<T> Factors<T> where T: Eq + Hash {
    pub fn decay(&mut self) {
        for factor in self.0.values().iter_mut() {
            factor.decay();
        }
    }

    pub fn add(&mut self, ftype: T, amt: f32) {
        if !self.inner.contains_key(ftype) {
            self.inner.insert(ftype, 0.0);
        }

        *self.inner.get_mut().unwrap() += amt;
    }

    pub fn factor(&self, ftype: T) -> f32 {
        self.inner.get(&ftype).unwrap_or(0.0)
    }
}

pub struct Settlement {
    name: String,
}

#[derive(GameRef)]
pub struct SettlementRef(pub Entity);

impl SettlementRef {
    pub fn carrying_capacity(&self, world: &World) -> f32 {
        100.0
    }

    pub fn population(&self, world: &World) -> usize {
        let mut total_pop = 0;
        for pop_ref in self.get::<Pops>(world).0.iter() {
            total_pop += pop_ref.get::<Pop>(world).size;
        }
        total_pop
    }
}

pub struct Pop {
    size: usize,
}

#[derive(GameRef)]
pub struct PopRef(pub Entity);

pub struct Pops(pub Vec<PopRef>);

pub struct FarmingPop {
    good: EconomicGood,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EconomicGood {
    Grain,
    Wine,
}

pub fn harvest_system(
    world: &World,
    pop_query: Query<(&Pop, &FarmingPop, &SettlementRef)>,
) {
    for (pop, farming_pop, settlement_ref) in pop_query.iter() {
        let mut farmed_amount = pop.size as f32;
        let carrying_capacity = settlement_ref.carrying_capacity(world);
        let comfortable_limit = carrying_capacity / 2.0;
        let pop_size = settlement_ref.population(world) as f32;
        if pop_size > comfortable_limit {
            // population pressure on available land, seek more
            // world.add_command(Box::new(PopSeekMigrationCommand {
            //     pop: pop.clone(),
            //     pressure: (pop_size / comfortable_limit).powi(2),
            // }))
        }
        if pop_size > carrying_capacity {
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

pub struct GoodsStorage(pub HashMap<EconomicGood, f64>);

impl GoodsStorage {
    pub fn add_resources(&mut self, good: EconomicGood, amount: f64) {
        if let Some(mut current_res) = self.0.get_mut(&good) {
            *current_res += amount;
        } else {
            self.0.insert(good, amount);
        }
    }

    pub fn set_resource_factor(&mut self, good: EconomicGood, factor: f64) {
        if let Some(mut current_res) = self.0.get_mut(&good) {
            *current_res *= factor;
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
            self.0.insert(good, 0.0);
            Some(amount)
        }
    }
}

pub struct Language {
    pub name: String,
    pub vowels: Vec<String>,
    pub initial_consonants: Vec<String>,
    pub middle_consonants: Vec<String>,
    pub end_consonants: Vec<String>,
}

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

pub struct PopPlugin;

impl Plugin for PopPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_stage_after(InitStage::LoadMap, InitStage::LoadPops, SystemStage::single_threaded())
            // .add_system(farmer_production_system.system())
            // .add_system(pop_growth_system.system())
            // .add_system(spawn_pops.system())
            // .add_system(pop_migration_system.system())
            // .add_system_to_stage(FinishStage::Main, migration_event_system.system())
            // .insert_resource(SpawnedPops(false))
            // .add_event::<MigrationEvent>()
            ;

    }
}
