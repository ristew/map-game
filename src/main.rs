pub mod ui;
pub mod tag;
pub mod map;
pub mod constant;
pub mod pops;
pub mod probability;
pub mod save;
pub mod input;
pub mod camera;
pub mod time;
pub mod province;
pub mod stage;
pub mod modifier;

use std::{collections::HashMap, error::Error, fs::File, io::Read};

use modifier::ModifierPlugin;
use province::ProvincePlugin;
// fuck yo namespace
use ui::*;
use map::*;
use tag::*;
use constant::*;
use pops::*;
use save::*;
use input::*;
use camera::*;
use time::TimePlugin;
use stage::*;
use macroquad::prelude::*;
use strum::{EnumIter, IntoEnumIterator};
use serde::{Deserialize, Serialize};

pub struct RenderContext<'a> {
    terrain_textures: HashMap<TerrainType, Texture2D>,
    province_colors: HashMap<MapCoordinate, Color>,
    camera: &'a Camera2D,
}

impl<'a> RenderContext<'a> {
    pub fn load_textures(&mut self) {
        for terrain_type in TerrainType::iter() {
            let path_str = format!("assets/textures/{:?}.png", terrain_type);
        }
    }
    pub fn terrain_texture(&self, terrain_type: TerrainType) -> Texture2D {
        *self.terrain_textures.get(&terrain_type).unwrap()
    }

    pub fn map_coord_to_window_pos(&self, coord: MapCoordinate) -> Vec2 {
        let (x, y) = coord.pixel_pos();
        Vec2::new(self.camera.zoom.x * (x + self.camera.offset.x), self.camera.zoom.y * (y + self.camera.offset.y))
    }

    pub fn province_color(&self, coord: MapCoordinate) -> Color {
        *self.province_colors.get(&coord).unwrap_or(&WHITE)
    }
}

pub struct PopId(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
pub enum TerrainType {
    Plains,
    Hills,
    Mountains,
    Forested,
    Marsh,
    Desert,
    Arid,
    Islands,
    Sea,
    Ocean,
    Lake,
}

pub struct CultureId(u32);

pub struct Culture {

}

pub struct Province {
    terrain_type: TerrainType,
    local_name: String,
    coord: MapCoordinate,
    names: HashMap<CultureId, String>,
    pops: Vec<PopId>,
}

impl Province {
    pub fn render(&self, ctx: &RenderContext) {
        let texture = ctx.terrain_texture(self.terrain_type);
        let pos = ctx.map_coord_to_window_pos(self.coord);
        let color = ctx.province_color(self.coord);

        draw_texture(texture, pos.x, pos.y, color);
    }
}

pub struct Map(pub HashMap<MapCoordinate, Province>);

// what we save from the map editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseProvince {
    terrain_type: TerrainType,
    local_name: String,
    coord: MapCoordinate,
}

pub struct GameState<'a> {
    map: Map,
    ctx: RenderContext<'a>,
}

impl<'a> GameState<'a> {
    pub fn draw_map(&self) {
        for province in self.map.0.values() {
            province.render(&self.ctx);
        }
    }

    pub fn load_map(&mut self) -> Result<(), Box<dyn Error>> {
        let map_file_name = "map.ron";
        let mut file = File::open(map_file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let bases: Vec<BaseProvince> = serde_json::from_str(&contents)?;

        for base in bases {
            self.map.0.insert(base.coord, Province {
                terrain_type: base.terrain_type,
                local_name: base.local_name,
                coord: base.coord,
                names: HashMap::new(),
                pops: Vec::new(),
            });
        }
        Ok(())
    }
}

#[macroquad::main("map-game")]
async fn main() {
    let mut camera: Camera2D = Camera2D {
        zoom: vec2(1.0, 1.0),
        target: vec2(0.0, 0.0),
        ..Default::default()
    };
    let mut game_state = GameState {
        map: Map(HashMap::new()),
        ctx: RenderContext {
            terrain_textures: HashMap::new(),
            province_colors: HashMap::new(),
            camera: &camera,
        }
    };
    game_state.ctx.load_textures();
    if let Err(e) = game_state.load_map() {
        eprintln!("{}", e);
    }
    set_camera(&camera);
    loop {
        clear_background(WHITE);
        let delta = get_frame_time();
        game_state.draw_map();
        next_frame().await
    }
}
