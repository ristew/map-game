use bevy::{core::FixedTimestep, ecs::{component::Component, system::Command, world::EntityRef, system::SystemParam}, prelude::*};
use rand::{Rng, distributions::Slice, prelude::SliceRandom, random, thread_rng};
use rand_distr::Uniform;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use crate::prelude::*;
use crate::{constant::{DAY_LABEL, DAY_TIMESTEP}, map::*, province::{Province, ProvinceMap, ProvinceRef, ProvinceSettlements}};
use crate::time::*;
use crate::probability::*;
use crate::stage::*;
use crate::factor::*;
use crate::settlement::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PopFactor {
    Prominence,
    Demand(GoodType),
    PopulationPressure,
}

impl FactorType for PopFactor {
    fn base_decay(&self) -> FactorDecay {
        match *self {
            PopFactor::Prominence => FactorDecay::Exponential(0.01),
            PopFactor::Demand(good) => FactorDecay::None,
            PopFactor::PopulationPressure => FactorDecay::None,
        }
    }

    fn default_amount(&self) -> f32 {
        match self {
            _ => 0.0,
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

#[game_ref]
pub struct PopRef(pub Entity);


// pub type PopQuery<'w> = Query<'w, (&'w Pop, &'w FarmingPop, &'w MapCoordinate)>;

pub struct Pop {
    pub size: isize,
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
pub struct KidBuffer(pub VecDeque<isize>);

impl KidBuffer {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn size(&self) -> isize {
        self.0.iter().fold(0, |acc, e| acc + e)
    }

    pub fn spawn(&mut self, babies: isize) -> isize {
        // println!("spawn babies {}", babies);
        self.0.push_front(babies);
        // println!("{:?}", self);
        if self.0.len() > 12 {
            self.0.pop_back().unwrap()
        } else {
            babies
        }
    }

    pub fn starve(&mut self) -> isize {
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

impl FactorType for CultureFactor {
    fn base_decay(&self) -> FactorDecay {
        todo!()
    }

    fn default_amount(&self) -> f32 {
        match self {
            _ => 0.0,
        }
    }
}

#[game_ref]
pub struct CultureRef(pub Entity);
pub struct Culture {
    pub name: String,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PolityFactor {

}

impl FactorType for PolityFactor {
    fn base_decay(&self) -> FactorDecay {
        todo!()
    }

    fn default_amount(&self) -> f32 {
        match self {
            _ => 0.0,
        }
    }
}

#[game_ref]
pub struct PolityRef(pub Entity);

impl PolityRef {
    pub fn color(&self) -> Color {
        let r = (self.0.id() % 8).pow(3) % 8;
        let b = (self.0.id() % 8).pow(2) % 8;
        let g = self.0.id() % 8;
        Color::rgb(r as f32 / 8.0, g as f32 / 8.0, b as f32 / 8.0)
    }
}

// pub type PolityQuery<'w> = Query<'w, (&'w Polity)>;

pub struct Polity {
    pub name: String,
}

pub struct Hunger {

}

pub fn pop_eat_system(
    date: Res<CurrentDate>,
    mut pop_query: Query<(&Factors<PopFactor>)>,
) {

}

pub fn growth_system(
    mut commands: Commands,
    date: Res<CurrentDate>,
    mut pop_query: Query<(Entity, &mut Pop, &mut KidBuffer)>,
) {
    if !date.is_year {
        return;
    }
    println!("growth_system {}", *date);
    for (pop_ent, mut pop, mut kb) in pop_query.iter_mut() {
        let babies = positive_isample(2, pop.size * 4 / 100);
        let deaths = positive_isample(2, pop.size / 50);
        let new = kb.spawn(babies) as isize - deaths as isize;
        pop.size = pop.size + new;
        if pop.size < 0 {
            commands.add(PopDieCommand(PopRef(pop_ent)));
        }
    }
}

pub struct PopDieCommand(pub PopRef);

impl Command for PopDieCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let settlement = *self.0
            .get::<SettlementRef>(world);
        settlement
            .get_mut::<SettlementPops>(world)
            .remove_pop(self.0);
        world
            .despawn(self.0.entity());
    }
}

pub fn harvest_system(
    date: Res<CurrentDate>,
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
        // println!("size {} comf {}", settlement_size, comfortable_limit);
        if settlement_size as f32 > comfortable_limit {
            pop_factors.add(PopFactor::PopulationPressure, 0.2);
            // population pressure on available land, seek more
            // world.add_command(Box::new(PopSeekMigrationCommand {
            //     pop: pop.clone(),
            //     pressure: (pop_size / comfortable_limit).powi(2),
            // }))
        } else {
            pop_factors.add(PopFactor::PopulationPressure, -0.2);
        }
        if settlement_size as f32 > carrying_capacity {
            println!("less is farmed");
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

#[game_ref]
pub struct LanguageRef(pub Entity);

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum LanguageFactor {

}

impl FactorType for LanguageFactor {
    fn base_decay(&self) -> FactorDecay {
        todo!()
    }

    fn default_amount(&self) -> f32 {
        match self {
            _ => 0.0,
        }
    }
}

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

    pub fn generate_name(&self, max_middle: isize) -> String {
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

// pub enum PopCommand {
//     SeekMigration(PopRef),
//     Migrate(PopRef, ProvinceRef)
// }

// pub struct PopCommands(Vec<PopCommand>);

// pub fn pop_command_system(
//     mut pop_commands: ResMut<PopCommands>,
// ) {
//     for cmd in pop_commands.0.iter() {
//         match cmd {
//             PopCommand::SeekMigration(pop) => {
//                 let coordinate =
//             },
//             PopCommand::Migrate(pop, province) => {},
//         }
//     }

//     pop_commands.0.clear();
// }


pub struct PopSeekMigrationCommand {
    pub pop: PopRef,
    pub pressure: f32,
}

impl Command for PopSeekMigrationCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let pop_size = self.pop.get::<Pop>(world).size;
        let coordinate = *self
            .pop
            .accessor(world)
            .get_ref::<ProvinceRef>()
            .get::<MapCoordinate>();
        let random_point = coordinate.random_local();
        if random_point == coordinate {
            // got unlucky, just die
            return;
        }

        let province_map = world.get_resource::<ProvinceMap>().unwrap();
        let province_maybe = province_map.0.get(&random_point);
        // missed the map?
        if province_maybe.is_none() {
            return;
        }
        let pref = province_maybe.unwrap();
        if !pref.get::<MapTile>(world).tile_type.inhabitable() {
            return;
        }
        let mut target_value = self.pressure.powf(1.5);
        if let Some(settlement) = pref.try_get::<SettlementRef>(world) {
            println!("settlement?? {:?}", settlement);
            return;
            target_value -= 1.0;
            if settlement.get::<CultureRef>(world) != self.pop.get::<CultureRef>(world) {
                target_value -= 2.0;
            } else {
                println!("meld");
            }
        }
        if individual_event(logistic(target_value)) {
            // println!("really migrate?? {:?}", self.pop);
            let migration_status = {
                let arrival = world.get_resource::<CurrentDate>().unwrap().date.days_after(30);
                let migrating = pop_size / 5;
                MigrationStatus {
                    dest: *pref,
                    migrating,
                    settlement: None,
                    arrival,
                }
            };
            // println!("lose {} people of {}", migration_status.migrating, pop_size);
            self.pop.clear_factor(world, PopFactor::PopulationPressure);
            world
                .get_mut::<Pop>(self.pop.entity())
                .unwrap()
                .size -= migration_status.migrating;
            world
                .entity_mut(self.pop.entity())
                .insert(migration_status);
        }

            //             let settlement_carrying_capacity = settlement.get().carrying_capacity(world);
            //             if (settlement.get().population(world) as f32) < settlement_carrying_capacity / 4.0 {
            //                 let size = (self.pop.get().size / 4).min((settlement_carrying_capacity / 4.0).round() as isize);
            //                 self.pop.get_mut().migration_status = Some(MigrationStatus {
            //                     migrating: size,
            //                     dest: target_province_id.clone(),
            //                     date: world.date.day + 60,
            //                     settlement: Some(settlement.clone()),
            //                 });
            //                 world.events.add_deferred(Rc::new(MigrationDoneEvent(self.pop.clone())), world.date.day + 60);
            //                 return;
            //             }
            //         }
            //     }
            //     if individual_event(logistic(target_value)) {
            //         // println!("migrate {:?} to {}", self.pop, random_point);
            //         let size = self.pop.get().size / 5;
            //         self.pop.get_mut().migration_status = Some(MigrationStatus {
            //             migrating: size,
            //             dest: target_province_id.clone(),
            //             date: world.date.day + 60,
            //             settlement: None,
            //         });
            //         world.events.add_deferred(Rc::new(MigrationDoneEvent(self.pop.clone())), world.date.day + 60);
            //     }
            // }
    }
}



pub struct MigrationStatus {
    pub dest: ProvinceRef,
    pub migrating: isize,
    pub settlement: Option<SettlementRef>,
    pub arrival: Date,
}


fn pop_migration_system(
    mut commands: Commands,
    mut migrating_pops: Query<(Entity, &mut Pop, &MigrationStatus, &PopLanguage, &CultureRef, &PolityRef)>,
    date: Res<CurrentDate>,
) {
    if !date.is_day {
        return;
    }
    for (pop_ent, mut pop, migration_status, language, &culture, &polity) in migrating_pops.iter_mut() {
        if migration_status.arrival.is_after(date.date) {
            if let Some(settlement) = migration_status.settlement {
                commands.add(SpawnPopCommand {
                    province: migration_status.dest,
                    settlement,
                    language: language.language,
                    culture,
                    polity,
                    size: migration_status.migrating,
                })
            } else {
                commands.add(SpawnSettlementCommand {
                    province: migration_status.dest,
                    language: language.language,
                    culture,
                    polity,
                    size: migration_status.migrating,
                })
            }
            commands
                .entity(pop_ent)
                .remove::<MigrationStatus>();
        }
    }
}

pub struct PopMigrateCommand {
    pub pop: PopRef,
}

impl Command for PopMigrateCommand {

    fn write(self: Box<Self>, world: &mut World) {
        // // println!("finally migrate {:?}", self.pop);
        // if let Some(settlement_id) = self.settlement.clone() {
        //     settlement_id.get_mut().accept_migrants(world, self.pop.clone(), self.migrating);
        // } else {
        //     add_settlement(world, self.pop.get().culture.clone(), self.dest.clone(), self.pop.get().polity.clone(), self.migrating);
        // }
        // self.pop.get_mut().size -= self.migrating;
    }
}

pub struct GlobalPopulation(pub isize);
pub struct MaxProvincePopulation(pub isize);

fn global_population_system(
    mut global_pop: ResMut<GlobalPopulation>,
    pops: Query<&Pop>,
) {
    global_pop.0 = 0;
    for pop in pops.iter() {
        global_pop.0 += pop.size;
    }
}

pub struct PopPlugin;

impl Plugin for PopPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let pop_systems = SystemSet::new()
            .with_system(harvest_system.system().label(DAY_LABEL))
            .with_system(growth_system.system().label(DAY_LABEL))
            .with_system(pop_migration_system.system().label(DAY_LABEL))
            .with_system(global_population_system.system().label(DAY_LABEL));
            // .with_run_criteria(
            //     FixedTimestep::step(0.0001)
            //         // labels are optional. they provide a way to access the current
            //         // FixedTimestep state from within a system
            //         .with_label(DAY_TIMESTEP),
            // )
        app
            .add_startup_stage_after(InitStage::LoadMap, InitStage::LoadPops, SystemStage::single_threaded())
            .add_system_set_to_stage(DayStage::Main, pop_systems)
            .insert_resource(GlobalPopulation(0))
            // .add_system(pop_growth_system.system())
            // .add_system(spawn_pops.system())
            // .add_system(pop_migration_system.system())
            // .add_system_to_stage(FinishStage::Main, migration_event_system.system())
            // .insert_resource(SpawnedPops(false))
            // .add_event::<MigrationEvent>()
            ;

    }
}
