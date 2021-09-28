use bevy::ecs::system::Command;
use bevy::{asset::LoadState, prelude::*, sprite::TextureAtlasBuilder};
use bevy::app::Plugin;
use pathfinding::directed::astar;
use std::collections::VecDeque;
use std::{collections::{HashMap, HashSet}, convert::TryInto, fs::File, io::{Read, Write}};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use bevy_tilemap::{point::Point3, prelude::*};
use rand::seq::SliceRandom;
use rand::{random, thread_rng};
use crate::formula::{FactorSubject, Formula, FormulaFn, FormulaSystem};
use crate::prelude::*;
use crate::probability::individual_event;
use crate::settlement::{Settlement, SettlementBundle, SettlementPops};
use crate::{input::CurrentOverlayType, province::{Province, ProvinceMap}, time::Date};
use crate::stage::{DayStage, InitStage};
use crate::factor::{FST, FactorRef};

use crate::{SettlementRef, pops::*};
use crate::constant::*;
use crate::save::*;
use crate::province::*;

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct MapCoordinate {
    pub x: isize,
    pub y: isize,
}

impl MapCoordinate {
    pub fn z(&self) -> isize {
        -self.x - self.y
    }

    pub fn pixel_pos(&self) -> (f32, f32) {
        let tile_x = (TILE_SIZE_X - 10.0) * (self.x as f32) + 10.0;
        (tile_x,
         TILE_SIZE_Y * (self.y as f32 + 1.0 + 0.5 * self.x as f32))
    }

    pub fn from_pixel_pos(pos: Vec2) -> Self {
        let coord_x = (pos.x - 10.0) / (TILE_SIZE_X - 10.0);
        let coord_y = pos.y / TILE_SIZE_Y - 0.5 * coord_x - 1.0;
        Self::from_cube_round(Vec2::new(coord_x, coord_y))
    }

    pub fn random_local(&self) -> MapCoordinate {
        let directions = vec![(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1), (0, 0)];
        let dir = directions.choose(&mut thread_rng()).unwrap();
        MapCoordinate {
            x: self.x + dir.0,
            y: self.y + dir.1,
        }
    }

    pub fn from_cube_round(pos: Vec2) -> Self {
        let x = pos.x;
        let y = pos.y;
        let z = -x - y;
        let mut rx = x.round();
        let mut ry = y.round();
        let rz = z.round();
        let xdiff = (rx - x).abs();
        let ydiff = (ry - y).abs();
        let zdiff = (rz - z).abs();
        if xdiff > ydiff + zdiff {
            rx = -ry - rz;
        } else if ydiff > zdiff {
            ry = -rx - rz;
        // } else {
        //     rz = -rx - ry;
        }
        Self {
            x: rx as isize,
            y: ry as isize,
        }
    }

    pub fn distance(self, other: MapCoordinate) -> isize {
        ((self.x - other.x).abs() + (self.y - other.y).abs() + (self.z() - other.z()).abs()) / 2
    }

    pub fn path_to(self, other: MapCoordinate) -> Vec<MapCoordinate> {
        astar::astar(
            &self,
            |coord| coord.neighbors_iter().map(|coord| (coord, 1)),
            |coord| coord.distance(self),
            |coord| *coord == self,
        ).unwrap().0
    }

    pub fn hex_side(self, other: MapCoordinate) -> HexSide {
        HexSide::N
    }

    pub fn from_window_pos(pos: Vec2, ) -> Self {
        Self::from_pixel_pos(pos)
    }

    pub fn neighbors(&self) -> Vec<MapCoordinate> {
        let mut ns = Vec::new();
        let directions = vec![
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, 0),
            (-1, 1),
            (0, 1)
        ];
        for (dx, dy) in directions {
            ns.push(MapCoordinate {
                x: self.x + dx,
                y: self.y + dy,
            });
        }
        ns
    }

    pub fn neighbors_shuffled(&self) -> Vec<MapCoordinate> {
        let mut result = self.neighbors();
        result.shuffle(&mut thread_rng());
        result
    }

    pub fn neighbors_iter(&self) -> MapCoordinateIter {
        MapCoordinateIter {
            neighbors: self.neighbors(),
        }
    }

    pub fn neighbors_shuffled_iter(&self) -> MapCoordinateIter {
        MapCoordinateIter {
            neighbors: self.neighbors_shuffled(),
        }
    }

    pub fn neighbors_in_radius(&self, radius: isize) -> Vec<MapCoordinate> {
        let mut items = Vec::new();
        for x in -radius..(radius + 1) {
            let min = (-radius).max(-x - radius);
            let max = radius.min(-x + radius);
            for y in min..(max + 1) {
                items.push(MapCoordinate { x: self.x + x, y: self.y + y });
            }
        }
        items
    }
    pub fn neighbors_in_radius_iter(&self, radius: isize) -> MapCoordinateIter {
        MapCoordinateIter {
            neighbors: self.neighbors_in_radius(radius),
        }
    }
    fn point3(&self) -> Point3 {
        Point3::new(self.x as i32, self.y as i32, 0)
    }
}

pub struct MapCoordinateIter {
    neighbors: Vec<MapCoordinate>,
}

impl Iterator for MapCoordinateIter {
    type Item = MapCoordinate;

    fn next(&mut self) -> Option<MapCoordinate> {
        self.neighbors.pop()
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapTileType {
    Plains,
    Water,
    Desert,
    Mountain,
    None,
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapTile {
    pub tile_type: MapTileType,
}

impl MapTileType {
    pub fn color(self) -> Color {
        match self {
            Self::Plains => Color::rgb(0.2, 0.7, 0.1),
            Self::Water => Color::rgb(0.0, 0.2, 0.7),
            Self::Desert => Color::rgb(0.7, 0.7, 0.5),
            Self::Mountain => Color::rgb(0.2, 0.2, 0.2),
            _ => Color::rgb(0.0, 0.0, 0.0),
        }
    }

    pub fn sprite(&self) -> isize {
        match self {
            MapTileType::Plains => 3,
            MapTileType::Water => 5,
            _ => 0,
        }
    }

    pub fn base_fertility(&self) -> f64 {
        match self {
            MapTileType::Plains => 10.0,
            _ => 0.0,
        }
    }

    pub fn base_arable_land(&self) -> f64 {
        match self {
            MapTileType::Plains => 0.7,
            _ => 0.0,
        }
    }

    pub fn inhabitable(&self) -> bool {
        match self {
            &MapTileType::Desert => false,
            &MapTileType::Water => false,
            _ => true,
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HexSide {
    N,
    NE,
    SE,
    S,
    SW,
    NW,
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BasicPlains {
    pub arable_factor: f32,
    pub fertility: f32,
}

pub struct TileTextureAtlas(pub Handle<TextureAtlas>);
pub struct HexMap(pub HashMap<MapCoordinate, Arc<Entity>>);

impl HexMap {
    pub fn neighbors_iter(&self, coord: MapCoordinate) -> HexMapIterator {
        let mut items = Vec::new();
        for neighbor in coord.neighbors_iter() {
            if let Some(item) = self.0.get(&neighbor) {
                items.push(item.clone());
            }
        }
        HexMapIterator {
            tiles: items,
        }
    }

    pub fn neighbors_in_radius_iter(&self, coord: MapCoordinate, radius: isize) -> HexMapIterator {
        let mut items = Vec::new();
        for neighbor in coord.neighbors_in_radius_iter(radius) {
            if let Some(item) = self.0.get(&neighbor) {
                items.push(item.clone());
            }
        }
        HexMapIterator {
            tiles: items,
        }
    }
}

pub struct HexMapIterator {
    tiles: Vec<Arc<Entity>>,
}

impl Iterator for HexMapIterator {
    type Item = Arc<Entity>;

    fn next(&mut self) -> Option<Arc<Entity>> {
        self.tiles.pop()
    }
}

pub fn create_map_tile(
    commands: &mut Commands,
    texture_atlas_handle: &Res<TileTextureAtlas>,
    x: isize,
    y: isize,
    tile_type: MapTileType
) -> Entity {
    let tile_material = tile_type.sprite();
    let province_ent = {
        let mut ent = commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.0.as_weak(),
                sprite: TextureAtlasSprite::new(tile_material as u32),
                ..Default::default()
            });
        ent
            .insert(MapCoordinate { x, y })
            .insert(MapTile{ tile_type })
            .insert(Province {
                total_population: 0,
                fertility: 30.0,
            })
            .insert(ProvincePops(Vec::new()))
            ;
        ent.id()
    };

    if individual_event(0.1) {
        commands.add(SpawnCultureCommand {
            province: ProvinceRef(province_ent),
        })
    }
    province_ent
}

pub struct SpawnCultureCommand {
    province: ProvinceRef,
}

impl Command for SpawnCultureCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let language = Language::new();
        let name = language.generate_name(2);
        let mut polity_name = language.generate_name(2);
        let language_ent = {
            let mut language_builder = world.spawn();
            language_builder
                .insert(language);
            language_builder.id()
        };
        let culture_ent = {
            let mut culture_builder = world.spawn();
            culture_builder
                .insert(Culture {
                    name,
                });
            culture_builder.id()
        };
        let mut polity_ent = {
            world
                .spawn()
                .insert(Polity { name: polity_name })
                .id()
        };
        let polity = PolityRef(polity_ent);
        let spawn_pop_command = SpawnSettlementCommand {
            province: self.province,
            language: LanguageRef(language_ent),
            culture: CultureRef(culture_ent),
            polity,
            size: 100,
        };

        Box::new(spawn_pop_command).write(world);
    }
}

pub struct SpawnSettlementCommand {
    pub province: ProvinceRef,
    pub language: LanguageRef,
    pub culture: CultureRef,
    pub size: isize,
    pub polity: PolityRef,
}

impl Command for SpawnSettlementCommand {
    fn write(self: Box<Self>, world: &mut World) {
        // TODO: only spawn polity if not in parent admin zone
        if let Some(settlement) = self.province.try_get::<Settlement>(world) {
            println!("someone already here! {:?}", settlement);
            return;
        }
        let name = self.language.get::<Language>(world).generate_name(2);
        let coordinate = *self.province.get::<MapCoordinate>(world);
        let settlement = SettlementRef({
            world.spawn()
                .insert_bundle(SettlementBundle {
                    info: Settlement {
                        name,
                        population: 0,
                    },
                    pops: SettlementPops(Vec::new()),
                    province: self.province,
                    polity: self.polity,
                    coordinate,
                })
                .id()
        });
        world.get_resource::<FormulaSystem<FST>>().unwrap().set_factor(&settlement.fst(FactorType::SettlementCarryingCapacity), 100.0);

        let formula = Formula::new(
            vec![
                (settlement.factor_ref(), FactorType::SettlementCarryingCapacity),
            ],
            |carrying_capacity| {
                carrying_capacity / 2.0
            },
            (settlement.factor_ref(), FactorType::SettlementPressure),
        );
        world
            .get_entity_mut(self.province.entity())
            .unwrap()
            .insert(settlement);
        Box::new(SpawnPopCommand {
            province: self.province,
            settlement,
            language: self.language,
            culture: self.culture,
            size: self.size,
            polity: self.polity,
        }).write(world);
    }
}

pub struct SpawnPopCommand {
    pub province: ProvinceRef,
    pub settlement: SettlementRef,
    pub language: LanguageRef,
    pub culture: CultureRef,
    pub size: isize,
    pub polity: PolityRef,
}

impl Command for SpawnPopCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let pop_ent = {
            let bundle = {
                PopBundle {
                    base: Pop { size: self.size },
                    province: self.province,
                    culture: self.culture,
                    settlement: self.settlement,
                    polity: self.polity,
                    language: PopLanguage {
                        language: self.language,
                        drift: 0.0,
                    },
                    storage: GoodStorage(HashMap::new()),
                    kid_buffer: KidBuffer(VecDeque::new()),
                }
            };
            world.spawn()
                .insert_bundle(bundle)
                .insert(FarmingPop { good: GoodType::Wheat })
                .id()
        };
        self.settlement.add_pop(world, PopRef(pop_ent));
    }
}

// bootstraps, sonny boy
pub fn create_map() {
    let save_file_name = "map.ron";
    let mut file = File::create(save_file_name).unwrap();
    let mut map_esds = Vec::new();
    for i in 0..200 {
        for j in 0..150 {
            let coord = MapCoordinate { x: i, y: j - (i / 2) };
            map_esds.push(MapEntitySaveData {
                map_coordinate: Some(coord),
                map_tile: Some(MapTile{ tile_type: MapTileType::Water }),
                districts: None,
            });
        }
    }
    let json = serde_json::to_string(&map_esds).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}


#[derive(Default, Clone)]
struct SpriteHandles {
    handles: Vec<HandleUntyped>,
    atlas_loaded: bool,
}

#[derive(Default, Clone)]
pub struct TileSpriteIndices(pub HashMap<MapTileType, usize>);

fn load_tile_map_system(
    mut commands: Commands,
    mut sprite_handles: ResMut<SpriteHandles>,
    mut tile_sprite_indices: ResMut<TileSpriteIndices>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    asset_server: Res<AssetServer>,
) {
    if sprite_handles.atlas_loaded {
        return;
    }

    // Lets load all our textures from our folder!
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    if let LoadState::Loaded =
        asset_server.get_group_load_state(sprite_handles.handles.iter().map(|handle| handle.id))
    {
        for handle in sprite_handles.handles.iter() {
            let texture = textures.get(handle).unwrap();
            texture_atlas_builder.add_texture(handle.clone_weak().typed::<Texture>(), &texture);
        }

        let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
        macro_rules! load_tile_sprite_index {
            ( $tt:ident ) => {
                println!("load {}", stringify!($tt));
                let texture: Handle<Texture> = asset_server.get_handle(format!("textures/{}.png", stringify!($tt)).as_str());
                tile_sprite_indices.0.insert(MapTileType::$tt, texture_atlas.get_texture_index(&texture).unwrap());
            }
        }
        load_tile_sprite_index!(Plains);
        load_tile_sprite_index!(Desert);
        load_tile_sprite_index!(Mountain);
        load_tile_sprite_index!(Water);
        load_tile_sprite_index!(None);
        let atlas_handle = texture_atlases.add(texture_atlas);
        let tilemap = Tilemap::builder()
            .auto_chunk()
            .auto_spawn(2, 2)
            // .dimensions(25, 25)
            .chunk_dimensions(250, 250, 1)
            .topology(GridTopology::HexX)
            .texture_dimensions(37, 32)
            // .tile_scale(32.0, 32.0, 32.0)
            .texture_atlas(atlas_handle)
            .finish()
            .unwrap();
        commands
            .spawn()
            .insert_bundle(TilemapBundle {
                tilemap,
                visible: Default::default(),
                transform: Default::default(),
                global_transform: Default::default(),
            });
        sprite_handles.atlas_loaded = true;
    } else {
        println!("no sprite handles");
    }
}

pub struct LoadMap(pub Option<String>);
fn build_world(
    mut commands: Commands,
    mut load_map: ResMut<LoadMap>,
    mut query: Query<&mut Tilemap>,
    mut hex_map: ResMut<HexMap>,
    tile_sprite_indices: Res<TileSpriteIndices>,
) {
    if load_map.0 == None {
        return;
    }
    if let Some(mut map) = query.iter_mut().next() {
        let save_file_name = load_map.0.as_ref().unwrap();
        let mut file = File::open(save_file_name).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let entities: Vec<MapEntitySaveData> = serde_json::from_str(&contents).unwrap();
        // let mut tiles = Vec::new();
        for esd in &entities {
            let mut ecmds = commands.spawn();
            macro_rules! load_component {
                ( $name:ident ) => {
                    if let Some(c) = esd.$name {
                        ecmds.insert(c);
                    }
                }
            }
            if let Some(map_tile) = esd.map_tile {
                // let tile_material = map_tile.tile_type.sprite();
                // ecmds.insert_bundle(SpriteSheetBundle {
                //     texture_atlas: texture_atlas_handle.0.as_weak(),
                //     sprite: tile_material,
                //     ..Default::default()
                // });
                hex_map.0.insert(esd.map_coordinate.unwrap(), Arc::new(ecmds.id()));
                let point = esd.map_coordinate.unwrap().point3();
                map.insert_tile(Tile {
                    point,
                    sprite_index: *tile_sprite_indices.0.get(&map_tile.tile_type).unwrap(),
                    ..Default::default()
                });
                map.spawn_chunk_containing_point(point).unwrap();
                load_component!(map_coordinate);
                load_component!(map_tile);
                let province_ent = {
                    ecmds
                        .insert(Province {
                            total_population: 0,
                            fertility: 30.0,
                        })
                        .insert(ProvincePops(Vec::new()));
                    ecmds.id()
                };
                if individual_event(0.1) && esd.map_tile.unwrap().tile_type.inhabitable() {
                    commands.add(SpawnCultureCommand {
                        province: ProvinceRef(province_ent),
                    });
                }
            }
        }
        // for chunk in &chunks {
        //     map.spawn_chunk(chunk).unwrap();
        // }
        // map.insert_tiles(tiles).unwrap();
        commands.add(ResetProvinceMap);
        load_map.0 = None;
        println!("world built");
    }
}

fn setup_tile_sprite_handles_system(mut tile_sprite_handles: ResMut<SpriteHandles>, asset_server: Res<AssetServer>) {
    tile_sprite_handles.handles = asset_server.load_folder("textures").unwrap();
}



pub fn position_translation(mut q: Query<(&MapCoordinate, &mut Transform)>) {
    for (pos, mut transform) in q.iter_mut() {
        let (x, y) = pos.pixel_pos();
        transform.translation = Vec3::new(x, y, transform.translation.z);
    }
}

fn map_tile_type_changed_system(
    query: Query<(&MapTile, &MapCoordinate), Changed<MapTile>>,
    tile_sprite_indices: Res<TileSpriteIndices>,
    load_map: Res<LoadMap>,
    mut tile_map_query: Query<&mut Tilemap>,
) {
    if load_map.0 != None {
        return;
    }
    for (map_tile, coord) in query.iter() {
        for mut tile_map in tile_map_query.iter_mut() {
            if let Some(mut tile) = tile_map.get_tile_mut(coord.point3(), 0) {
                let new_sprite = *tile_sprite_indices.0.get(&map_tile.tile_type).unwrap();
                tile.index = new_sprite;
            }
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiverSize {
    Small,
    Medium,
    Large,
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiverSegment {
    size: RiverSize,
    coordinate: MapCoordinate,
    from: HexSide,
    to: HexSide,
}

pub struct River {
    segments: Vec<RiverSegment>,
}

impl River {
    pub fn generate_river_from_points(
        points: Vec<MapCoordinate>,
        size: RiverSize,
    ) -> Self {
        let mut segments = Vec::new();
        for i in 0..points.len() - 1 {
            let p1 = points[i];
            let p2 = points[i + 1];
            let path = p1.path_to(p2);
        }
        println!("Segements {:?}", segments);
        Self {
            segments,
        }
    }
}

pub enum OverlayCommand {
    Map(HashMap<MapCoordinate, Color>),
    Clear,
}

fn show_overlay_system(
    tiles_query: Query<(&MapTile, &MapCoordinate)>,
    overlay_command: Res<OverlayCommand>,
    load_map: Res<LoadMap>,
    date: Res<CurrentDate>,
    tile_sprite_indices: Res<TileSpriteIndices>,
    mut tile_map_query: Query<&mut Tilemap>,
) {
    if load_map.0 != None {
        return;
    }
    match &*overlay_command {
        OverlayCommand::Map(map) => {
            if overlay_command.is_changed() {
                for (coord, color) in map.iter() {
                    let point = coord.point3();
                    for mut tile_map in tile_map_query.iter_mut() {
                        let mut tile = tile_map.get_tile_mut(point, 0).unwrap();
                        tile.color = *color;
                        let sprite_index = *tile_sprite_indices.0.get(&MapTileType::None).unwrap();
                        tile.index = sprite_index;
                    }
                }
            }
        },
        OverlayCommand::Clear => {
            if overlay_command.is_changed() {
                for (map_tile, coord) in tiles_query.iter() {
                    let point = coord.point3();
                    for mut tile_map in tile_map_query.iter_mut() {
                        let mut tile = tile_map.get_tile_mut(point, 0).unwrap();
                        tile.color = Color::WHITE;
                        let sprite_index = *tile_sprite_indices.0.get(&map_tile.tile_type).unwrap();
                        tile.index = sprite_index;
                    }
                }
            }
        },
    }
}

pub fn pop_overlay_system(
    mut frame: Local<isize>,
    mut overlay_command: ResMut<OverlayCommand>,
    tile_coord_query: Query<&MapCoordinate, With<MapTile>>,
    province_map: Res<ProvinceMap>,
    province_query: Query<&Province>,
    current_overlay: Res<CurrentOverlayType>,
    date: Res<CurrentDate>,
    mut tile_map_query: Query<&mut Tilemap>,
) {
    *frame += 1;
    if *frame % 20 == 0 && *current_overlay == CurrentOverlayType::ProvincePop {
        let mut pop_map = HashMap::new();
        let mut max_pop = 0;
        for coord in tile_coord_query.iter() {
            let pop = province_query.get(province_map.0.get(coord).unwrap().0).map(|pi| pi.total_population).unwrap_or(0);
            pop_map.insert(coord, pop);
            if pop > max_pop {
                max_pop = pop;
            }
        }
        for (coord, pop) in pop_map.iter() {
            let color = if *pop > 0 {
                let brightness = *pop as f32 / max_pop as f32;
                Color::rgb(brightness, 0.2, 0.2)
            } else {
                Color::rgb(0.0, 0.2, 0.2)
            };
            let point = coord.point3();
            for mut tile_map in tile_map_query.iter_mut() {
                let mut tile = tile_map.get_tile_mut(point, 0).unwrap();
                tile.color = color;
            }
        }
        // *overlay_command = OverlayCommand::Map(tint_map);
    }
}

pub fn polity_overlay_system(
    mut frame: Local<isize>,
    mut overlay_command: ResMut<OverlayCommand>,
    tile_sprite_indices: Res<TileSpriteIndices>,
    tile_coord_query: Query<&MapCoordinate, With<MapTile>>,
    polity_query: Query<(&PolityRef, &MapCoordinate)>,
    current_overlay: Res<CurrentOverlayType>,
    date: Res<CurrentDate>,
    mut tile_map_query: Query<&mut Tilemap>,
) {
    *frame += 1;
    if *frame % 20 == 0 && *current_overlay == CurrentOverlayType::Polity {
        // println!("polity overlay change");
        for (&polity, &coordinate) in polity_query.iter() {
            let color = polity.color();
            // println!("polity overlay: {:?} {:?} {:?}", polity, coordinate, color);
            let point = coordinate.point3();
            for mut tile_map in tile_map_query.iter_mut() {
                let mut tile = tile_map.get_tile_mut(point, 0).unwrap();
                tile.color = color;
                let sprite_index = *tile_sprite_indices.0.get(&MapTileType::None).unwrap();
                tile.index = sprite_index;
            }
        }
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_stage(InitStage::LoadMap, SystemStage::single_threaded())
            .add_startup_system_to_stage(InitStage::LoadMap, load_map_system.system())
            .add_startup_system_to_stage(InitStage::LoadMap, setup_tile_sprite_handles_system.system())
            .init_resource::<SpriteHandles>()
            .init_resource::<TileSpriteIndices>()
            .add_event::<OverlayCommand>()
            .insert_resource(LoadMap(None))
            .insert_resource(HexMap(HashMap::new()))
            .insert_resource(OverlayCommand::Clear)
            .insert_resource(CurrentOverlayType::None)
            .add_system(load_tile_map_system.system())
            .add_system(build_world.system())
            .add_system(pop_overlay_system.system())
            .add_system(polity_overlay_system.system())
            .add_system(show_overlay_system.system())
            .add_system_to_stage(DayStage::Main, map_tile_type_changed_system.system())
            .add_system(position_translation.system());
    }
}
