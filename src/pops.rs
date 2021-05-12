use bevy::{ecs::system::Command, prelude::*};
use std::collections::HashMap;
use crate::{map::{HexMap, LoadMap, MapCoordinate, MapTile, MapTileType}, province::{ProvinceInfo, ProvinceInfos}};
use crate::time::*;
use crate::probability::*;
use crate::stage::*;

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EconomicGood {
    Grain,
    Wine,
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

// pub fn pop_growth_generator_system<T: 'static + Population + Send + Sync> (
//     pops_query: Query<(Entity, &T)>,
// ) {

// }
pub struct Culture {
    name: String,

}

#[derive(Debug, Clone, PartialEq)]
pub struct CultureRef(String);
#[derive(Debug, Clone, PartialEq)]
pub struct ReligionRef(String);
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Class {
    Farmer,
    Laborer,
    Noble,
}
pub struct FarmingPop {
    resource: EconomicGood,
    resource_base_harvest: f64,
    harvest_date: DayOfYear,
}

pub struct NoblePop {
    resource: EconomicGood,
}

pub struct BasePop {
    pub size: isize,
    pub growth: f64,
    pub culture: CultureRef,
    pub religion: ReligionRef,
    pub class: Class,
    pub factors: Vec<String>,
    pub resources: GoodsStorage,
    pub hunger: f64,
}

impl BasePop {
    fn same_group(&self, other: &BasePop) -> bool {
        self.culture == other.culture && self.religion == other.religion && self.class == other.class
    }
}

pub fn farmer_production_system(
    mut farmer_query: Query<(&mut BasePop, &FarmingPop, &MapCoordinate)>,
    tile_type_query: Query<&MapTile>,
    pinfos: Res<ProvinceInfos>,
    date: Res<Date>,
    hex_map: Res<HexMap>,
) {
    if date.is_day {
        for (mut base_pop, farming_pop, coord) in farmer_query.iter_mut() {
            if date.days_after_doy(farming_pop.harvest_date) == 0 {
                let pinfo = pinfos.0.get(&coord).unwrap();
                let tile_entity = if let Some(te) = hex_map.0.get(&coord) {
                    te
                } else {
                    continue;
                };
                let tile_type = tile_type_query.get(**tile_entity).unwrap().tile_type;

                println!("harvest");
                let carrying_capacity = tile_type.base_arable_land() * 100.0 * 15.0;
                let productive_pops = if pinfo.total_population as f64 > carrying_capacity {
                    let excess = pinfo.total_population as f64 - carrying_capacity;
                    println!("harvest excess {}", excess);
                    base_pop.size as f64 * (1.0 - excess / pinfo.total_population as f64).max(0.1)
                } else {
                    base_pop.size as f64
                };
                let total_harvest = dev_mean_sample(0.5, 1.0).clamp(0.5, 2.0) * 250.0 * pinfo.fertility * productive_pops;
                // TODO: discriminate major landholding populations for capacity
                base_pop.resources.set_resource_factor(farming_pop.resource, 0.1);
                base_pop.resources.add_resources(farming_pop.resource, total_harvest);

            }
        }
    }
}

// fn pop_growth_system(
//     mut pop_query: Query<(&mut BasePop, &MapCoordinate)>,
//     pinfos: Res<ProvinceInfos>,
//     date: Res<Date>,
// ) {
//     if date.is_month {
//         for (mut pop, coord) in pop_query.iter_mut() {
//             pop.size += 5;
//             pinfos.0.get_mut(&coord).unwrap().total_population += 5;
//         }
//     }
// }

fn pop_growth_system(
    mut pop_query: Query<(&mut BasePop, &MapCoordinate)>,
    pinfos: Res<ProvinceInfos>,
    date: Res<Date>,
) {
    if date.is_month {
        for (mut pop, coord) in pop_query.iter_mut() {
            let required_food = 16.0 * pop.size as f64;
            let mut pinfo = pinfos.0.get_mut(&coord).unwrap();
            if let Some(deficit) = pop.resources.use_goods_deficit(EconomicGood::Grain, required_food) {
                pop.hunger += deficit / required_food;
                pop.hunger = pop.hunger.min(5.0);
                println!("hunger! {}", pop.hunger);
            } else {
                pop.hunger = 0.0;
            }
            let pop_growth = pop.size as f64 * (1.0 - pop.hunger) / 600.0;
            pop.growth = pop_growth;
            let newpops = dev_mean_sample((pop.size as f64 / 1000.0).max(0.5), pop_growth).round() as isize;
            if newpops < 0 {
                // println!("size {} growth {} newpops {}", pop.size, pop_growth, newpops);
            }
            pop.size += newpops;
            pinfo.total_population += newpops;
        }
    }
}

struct PopChange {
    size: isize,
    pop_entity: Entity,
}

impl Command for PopChange {
    fn write(self: Box<Self>, world: &mut World) {
        let mut base_pop = world.entity_mut(self.pop_entity).get_mut::<BasePop>().unwrap();
        println!("popchange: {} {}", base_pop.size, self.size);
        base_pop.size += self.size;
    }
}

struct PopSpawn {
    size: isize,
    coord: MapCoordinate,
    class: Class,
    culture: CultureRef,
    religion: ReligionRef,
}

impl Command for PopSpawn {
    fn write(self: Box<Self>, world: &mut World) {
        let pop_ent = {
            let mut pop = world
                .spawn();
            pop
                .insert(BasePop {
                    size: self.size,
                    culture: self.culture,
                    religion: self.religion,
                    class: self.class,
                    factors: Vec::new(),
                    resources: GoodsStorage(HashMap::new()),
                    hunger: 0.0,
                    growth: 0.0,
                })
                .insert(self.coord.clone());
            if self.class == Class::Farmer {
                pop
                    .insert(FarmingPop {
                        resource: EconomicGood::Grain,
                        harvest_date: DayOfYear {
                            day: 15,
                            month: 1,
                        },
                        resource_base_harvest: 250.0,
                    });
            }
            pop.id()
        };

        world.get_resource_mut::<ProvinceInfos>().unwrap().0.get_mut(&self.coord).unwrap().pops.push(pop_ent);
    }
}


struct MigrationEvent {
    source_pop_entity: Entity,
    target_coord: MapCoordinate,
    size: isize,
}

fn migration_event_system(
    mut commands: Commands,
    mut pinfos: ResMut<ProvinceInfos>,
    base_pop_query: Query<&BasePop>,
    mut migration_events: EventReader<MigrationEvent>,
) {
    for evt in migration_events.iter() {
        println!("migration event {:?}", evt.target_coord);
        commands.add(PopChange {
            size: -evt.size,
            pop_entity: evt.source_pop_entity,
        });
        let source_pop = base_pop_query.get(evt.source_pop_entity).unwrap();
        let mut found_pop = false;
        for target_pop_ent in pinfos.0.get(&evt.target_coord).unwrap().pops.iter() {
            println!("check target?");
            let target_pop = base_pop_query.get(*target_pop_ent).unwrap();
            if source_pop.same_group(&target_pop) {
                println!("found same pop");
                commands.add(PopChange {
                    size: evt.size,
                    pop_entity: *target_pop_ent,
                });
                found_pop = true;
                break;
            } else {
                println!("not same pop?");
            }
        }
        if !found_pop {
            // have to add a new pop
            commands.add(PopSpawn {
                size: evt.size,
                coord: evt.target_coord,
                class: source_pop.class,
                culture: source_pop.culture.clone(),
                religion: source_pop.religion.clone(),
            });
        }
    }
}

fn pop_migration_system(
    mut commands: Commands,
    pop_query: Query<(Entity, &BasePop, &MapCoordinate)>,
    tile_type_query: Query<&MapTile>,
    pinfos: Res<ProvinceInfos>,
    date: Res<Date>,
    hex_map: Res<HexMap>,
    spawned_pops: Res<SpawnedPops>,
    mut migration_events: EventWriter<MigrationEvent>,
) {
    if spawned_pops.0 && date.is_day {
        // let migrated_to = HashMap::new();
        for (ent, pop, coord) in pop_query.iter() {
            let migration_factor = pop.hunger + 50.0;
            if let Some(target_coord) = coord.neighbors_shuffled_iter().next() {
                // println!("try migrate {:?} to {:?}", coord, target_coord);
                let target_tile = if let Some(te) = hex_map.0.get(&target_coord) {
                    te
                } else {
                    continue;
                };
                if tile_type_query.get(**target_tile).map_or(MapTileType::None, |tt| tt.tile_type) != MapTileType::Plains {
                    println!("not a plains!");
                    continue;
                }
                let target_factor = pinfos.0.get(&target_coord).unwrap().total_population;
                if pop.size > 100 && migration_factor > target_factor as f64 {
                    if individual_event(0.05) {
                        println!("migrate {:?} to {:?}", coord, target_coord);
                        let evt = MigrationEvent {
                            source_pop_entity: ent,
                            target_coord,
                            size: 50,
                        };
                        migration_events.send(evt);
                    }
                }
            }
        }
    }
}

pub struct SpawnedPops(bool);

pub fn spawn_pops(
    mut commands: Commands,
    tiles_query: Query<(&MapCoordinate, &MapTile)>,
    load_map: Res<LoadMap>,
    mut spawned_pops: ResMut<SpawnedPops>,
    pinfos: ResMut<ProvinceInfos>,
) {
    if load_map.0 != None || spawned_pops.0 {
        return;
    }
    println!("pop setup");
    for (coord, tile) in tiles_query.iter() {
        pinfos.0.insert(*coord, ProvinceInfo {
            total_population: 0,
            fertility: 1.0,
            pops: Vec::new(),
        });
        if individual_event(0.1) && tile.tile_type == MapTileType::Plains {
            println!("spawn pop for {:?}", coord);
            commands.add(PopSpawn {
                size: 1000,
                class: Class::Farmer,
                culture: CultureRef("Default".to_string()),
                religion: ReligionRef("Default".to_string()),
                coord: *coord,
            });
            // let pop_ent = commands
            //     .spawn()
            //     .insert(BasePop {
            //         size: 1000,
            //         culture: CultureRef("Default".to_string()),
            //         religion: ReligionRef("Default".to_string()),
            //         class: Class::Farmer,
            //         factors: Vec::new(),
            //         resources: GoodsStorage(HashMap::new()),
            //         hunger: 0.0,
            //         growth: 0.0,
            //     })
            //     .insert(FarmingPop {
            //         resource: EconomicGood::Grain,
            //         harvest_date: DayOfYear {
            //             day: 15,
            //             month: 1,
            //         },
            //         resource_base_harvest: 250.0,
            //     })
            //     .insert(coord.clone())
            //     .id();
            // pinfos.0.insert(*coord, ProvinceInfo {
            //     total_population: 1000,
            //     fertility: 1.0,
            //     pops: vec![pop_ent],
            // });
        }
        spawned_pops.0 = true;
    }
}

pub struct PopPlugin;

impl Plugin for PopPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_stage_after(InitStage::LoadMap, InitStage::LoadPops, SystemStage::single_threaded())
            .add_system(farmer_production_system.system())
            .add_system(pop_growth_system.system())
            .add_system(spawn_pops.system())
            .add_system(pop_migration_system.system())
            .add_system_to_stage(FinishStage::Main, migration_event_system.system())
            .insert_resource(SpawnedPops(false))
            .add_event::<MigrationEvent>()
            ;

    }
}
