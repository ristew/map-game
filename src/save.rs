use std::fs::File;
use std::io::prelude::*;
use bevy::prelude::*;
use bevy::ecs::system::Command;
use serde::{Serialize, Deserialize};

use crate::prelude::*;

use super::map::*;

pub struct SaveMapCommand;

#[derive(Debug, Serialize, Deserialize)]
pub struct MapEntitySaveData {
    pub map_coordinate: Option<MapCoordinate>,
    pub map_tile: Option<MapTile>,
    pub districts: Option<Districts>,
}

/// Saves teh world, one entity at a time
impl Command for SaveMapCommand {
    fn write(
        self: Box<Self>,
        world: &mut World,
    ) {
        let save_file_name = "map.ron";
        let mut file = File::create(save_file_name).unwrap();
        let mut entities = Vec::new();
        for ent in world.query::<Entity>().iter(world) {
            macro_rules! component {
                ( $component:ident ) => {
                    if let Some(&c) = world.get::<$component>(ent) {
                        Some(c)
                    } else {
                        None
                    }
                }
            }

            let esd = MapEntitySaveData {
                map_coordinate: component!(MapCoordinate),
                map_tile: component!(MapTile),
                districts: component!(Districts),
            };
            entities.push(esd);
        }
        let json = serde_json::to_string(&entities).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
}

pub fn load_map_system(
    mut load_map: ResMut<LoadMap>,
) {
    if let Err(e) = File::open("map.ron") {
        eprintln!("error loading map: {}", e);
        create_map();
    }
    load_map.0 = Some("map.ron".to_string());
}
