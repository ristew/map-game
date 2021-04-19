use std::fs::File;
use std::io::prelude::*;
use bevy::prelude::*;
use bevy::ecs::system::Command;
use serde::{Serialize, Deserialize};

use super::map::*;

pub struct SaveCommand;

#[derive(Debug, Serialize, Deserialize)]
pub struct EntitySaveData {
    pub map_coordinate: Option<MapCoordinate>,
    pub map_tile: Option<MapTile>,
}

/// Saves teh world, one entity at a time
impl Command for SaveCommand {
    fn write(
        self: Box<Self>,
        world: &mut World,
    ) {
        let save_file_name = "save.ron";
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

            let esd = EntitySaveData {
                map_coordinate: component!(MapCoordinate),
                map_tile: component!(MapTile),
            };
            entities.push(esd);
        }
        let json = serde_json::to_string(&entities).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
}

pub fn load_map_system(
    commands: Commands,
    commands2: Commands,
    texture_atlas_handle: Res<TileTextureAtlas>,
    texture_atlas_handle2: Res<TileTextureAtlas>,
) {
    if let Err(e) = load_map_system_err(commands, texture_atlas_handle) {
        eprintln!("error loading map: {}", e);
        create_map(commands2, texture_atlas_handle2);
    }

}

pub fn load_map_system_err(
    commands: Commands,
    texture_atlas_handle: Res<TileTextureAtlas>,
) -> Result<(), Box<dyn std::error::Error>> {
    let save_file_name = "save.ron";
    let mut file = File::open(save_file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let entities: Vec<EntitySaveData> = serde_json::from_str(&contents)?;
    // println!("{:?}", entities);
    load_map(entities, commands, texture_atlas_handle)
}
