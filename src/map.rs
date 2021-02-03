use bevy::prelude::*;
use bevy::app::Plugin;
use std::collections::HashMap;
use std::sync::Arc;
use super::tag::*;
use super::constant::*;

#[derive(Debug, Hash, PartialEq, Copy, Clone)]
pub struct MapCoordinate {
    pub x: isize,
    pub y: isize,
}

impl Eq for MapCoordinate {

}


impl MapCoordinate {
    pub fn z(&self) -> isize {
        -self.x - self.y
    }

    pub fn pixel_pos(&self) -> (f32, f32) {
        (TILE_SIZE * 1.5 * (self.x as f32), TILE_SIZE * SQRT_3 * ((self.y as f32) + 0.5 * (self.x as f32)))
    }

    pub fn from_pixel_pos(pos: Vec2) -> Self {
        let coord_x = pos.x / (TILE_SIZE * 1.5);
        let coord_y = (SQRT_3 * pos.y / 3.0 - pos.x / 3.0) / (TILE_SIZE);
        Self::from_cube_round(Vec2::new(coord_x, coord_y))
    }

    pub fn from_cube_round(pos: Vec2) -> Self {
        let x = pos.x;
        let y = pos.y;
        let z = -x - y;
        let mut rx = x.round();
        let mut ry = y.round();
        let mut rz = z.round();
        let xdiff = (rx - x).abs();
        let ydiff = (ry - y).abs();
        let zdiff = (rz - z).abs();
        if xdiff > ydiff && xdiff > zdiff {
            rx = -ry - rz;
        } else if ydiff > zdiff {
            ry = -rx - rz;
        } else {
            rz = -rx - ry;
        }
        Self {
            x: rx as isize,
            y: ry as isize,
        }
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

    pub fn neighbors_iter(&self) -> MapCoordinateIter {
        MapCoordinateIter {
            neighbors: self.neighbors(),
        }
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

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum MapTileType {
    Land,
    Water,
    None,
}

pub struct MapTile {
    pub tile_type: MapTileType,
}

impl MapTileType {
    pub fn color(self) -> Color {
        match self {
            Self::Land => Color::rgb(0.2, 0.7, 0.1),
            Self::Water => Color::rgb(0.0, 0.2, 0.7),
            _ => Color::rgb(0.0, 0.0, 0.0),
        }
    }

    pub fn sprite(&self) -> TextureAtlasSprite {
        match self {
            MapTileType::Land => TextureAtlasSprite::new(3),
            MapTileType::Water => TextureAtlasSprite::new(5),
            _ => TextureAtlasSprite::new(0),
        }
    }
}

pub struct TileTextureAtlas(pub Handle<TextureAtlas>);
pub struct TileMap(pub HashMap<MapCoordinate, Arc<Entity>>);

impl TileMap {
    pub fn neighbors_iter(&self, coord: MapCoordinate) -> TileMapIterator {
        let mut items = Vec::new();
        for neighbor in coord.neighbors_iter() {
            if let Some(item) = self.0.get(&neighbor) {
                items.push(item.clone());
            }
        }
        TileMapIterator {
            tiles: items,
        }
    }
}

pub struct TileMapIterator {
    tiles: Vec<Arc<Entity>>,
}

impl Iterator for TileMapIterator {
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
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.0.as_weak(),
            sprite: tile_material,
            ..Default::default()
        })
        .with(MapCoordinate { x, y })
        .with(MapTile{ tile_type })
        .current_entity()
        .unwrap()
}

pub fn create_map(
    commands: &mut Commands,
    texture_atlas_handle: Res<TileTextureAtlas>,
) {
    let mut map: TileMap = TileMap(HashMap::new());
    for i in 0..100 {
        for j in 0..100 {
            let coord = MapCoordinate { x: i, y: j - (i / 2) };
            let tile = create_map_tile(commands, &texture_atlas_handle, i, j - (i / 2), MapTileType::Land);
            map.0.insert(MapCoordinate { x: i, y: j - (i / 2) }, Arc::new(tile));
        }
    }

    commands.insert_resource(map);
}

pub fn position_translation(windows: Res<Windows>, mut q: Query<(&MapCoordinate, &mut Transform)>) {
    for (pos, mut transform) in q.iter_mut() {
        let (x, y) = pos.pixel_pos();
        transform.translation = Vec3::new(x, y, transform.translation.z);
    }
}

pub fn map_tile_type_changed_system(
    mut query: Query<(&MapTile, &mut TextureAtlasSprite), Mutated<MapTile>>,
) {
    for (map_tile, mut sprite) in query.iter_mut() {
        println!("change sprite");
        let new_sprite = map_tile.tile_type.sprite();
        sprite.index = new_sprite.index;
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system(map_tile_type_changed_system.system())
            .add_system(position_translation.system());
    }
}