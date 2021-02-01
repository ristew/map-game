extern crate bevy;

use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection, WindowOrigin, CameraProjection},
    diagnostic::{ FrameTimeDiagnosticsPlugin, DiagnosticsPlugin },
    input::{
        ElementState,
        mouse::{MouseButtonInput},
    },
    ecs::DynamicBundle,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;

const TILE_SIZE: f32 = 16.0;
const SQRT_3: f32 = 1.73205080757;

pub enum MapTileType {
    Land,
    Water,
}

pub struct UiMaterials {
    background_info: Handle<ColorMaterial>,
    default_button: Handle<ColorMaterial>,
}

pub struct MapTile {
    tile_type: MapTileType,
}

impl MapTileType {
    pub fn color(self) -> Color {
        match self {
            Self::Land => Color::rgb(0.2, 0.7, 0.1),
            Self::Water => Color::rgb(0.0, 0.2, 0.7),
            _ => Color::rgb(0.0, 0.0, 0.0),
        }
    }
}

pub struct TileTextureAtlas(Handle<TextureAtlas>);

#[derive(Debug, Hash, PartialEq)]
pub struct MapCoordinate {
    x: isize,
    y: isize,
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
}

pub struct TileMap(HashMap<MapCoordinate, Arc<Entity>>);

pub struct Selected;

pub struct SelectOutline;

pub fn create_map_tile(
    commands: &mut Commands,
    texture_atlas_handle: &Res<TileTextureAtlas>,
    x: isize,
    y: isize,
    tile_type: MapTileType
) -> Entity {
    let tile_material = match tile_type {
        MapTileType::Land => TextureAtlasSprite::new(3),
        MapTileType::Water => TextureAtlasSprite::new(5),
    };
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

pub fn setup_assets(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    let texture_handle = asset_server.load("hextiles.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 3, 23);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(TileTextureAtlas(texture_atlas_handle));
    commands.insert_resource(UiMaterials {
        background_info: materials.add(Color::rgb(0.7, 0.7, 0.4).into()),
        default_button: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
    });
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

pub struct SelectedInfoText(String);

pub struct UiBuilder<'a> {
    commands: &'a mut Commands,
    materials: Res<'a, UiMaterials>,
    default_font: Handle<Font>,
    parent_stack: Vec<Entity>,
    children_stack: Vec<Vec<Entity>>,
    last_entity: Option<Entity>,
}

pub struct MapCamera;

impl <'a> UiBuilder<'a> {
    pub fn spawn(&mut self, bundle: impl Send + Sync + DynamicBundle + 'static) -> &mut Self {
        self.commands.spawn(bundle);
        self.last_entity = self.commands.current_entity();
        let children_len = self.children_stack.len();
        if children_len > 0 {
            if let Some(mut children) = self.children_stack.get_mut(children_len - 1) {
                children.push(self.last_entity.unwrap());
            }
        }
        self
    }
    pub fn setup(&mut self) -> &mut Self {
        self.spawn(OrthographicCameraBundle::new_2d())
            .with(MapCamera)
            .spawn(UiCameraBundle::default());
        self
    }
    pub fn with(&mut self, component: impl Component) -> &mut Self {
        self.commands.with(component);
        self
    }
    pub fn children(&mut self) -> &mut Self {
        self.children_stack.push(Vec::new());
        self.parent_stack.push(self.last_entity.unwrap());
        self
    }
    pub fn end_children(&mut self) -> &mut Self {
        let parent = self.parent_stack.pop().unwrap();
        let children = self.children_stack.pop().unwrap();
        println!("children length: {}", children.len());
        self.commands.push_children(parent, children.as_slice());
        self
    }
    pub fn spawn_text_info<T>(&mut self, s: T) -> &mut Self where T: Into<String> {
        self.spawn(self.text_info(s));
        self
    }
    pub fn spawn_info_box(&mut self) -> &mut Self {
        self.spawn(NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Percent(20.0),
                        height: Val::Percent(50.0)
                    },
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Stretch,
                    align_content: AlignContent::FlexStart,
                    justify_content: JustifyContent::FlexStart,
                    ..Default::default()
                },
                material: self.materials.background_info.clone(),
                ..Default::default()
            });
        self
    }
    pub fn text_info<T>(&self, s: T) -> TextBundle where T: Into<String> {
        TextBundle {
            text: Text::with_section(
                s,
                TextStyle {
                    font: self.default_font.clone(),
                    font_size: 14.0,
                    color: Color::BLACK,
                },
                Default::default(),
            ),
            style: Style {
                size: Size {
                    width: Val::Auto,
                    height: Val::Px(16.0),
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn text_button(&self) -> ButtonBundle {
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: self.materials.default_button.clone(),
            ..Default::default()
        }
    }
}

pub fn setup_ui<'a>(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    ui_materials: Res<'a, UiMaterials>
) {
    let default_font = asset_server.load("fonts/DejaVuSansMono.ttf");
    let mut builder = UiBuilder {
        commands,
        materials: ui_materials,
        default_font: default_font,
        children_stack: Default::default(),
        parent_stack: Default::default(),
        last_entity: None,
    };

    builder
        .setup()
        .spawn_info_box()
        .children()
        .spawn_text_info("Select a tile")
        .with(SelectedInfoText("Select a tile".to_string()))
        .spawn_text_info("Tile details")
        .spawn_text_info("Select a tile")
        .end_children();


        // .with_children(|parent| {
        //     parent.spawn(builder.text_button())
        //     .with_children(|parent| {
        //         parent.spawn(builder.text_info("button"));
        //     });
        // })
        // .with_children(|parent| {
        //     parent.spawn(builder.text_info("Select a tile"))
        //         .with(SelectedInfoText("Select a tile".to_string()));
        //     parent.spawn(builder.text_info("Tile Information: "));
        // });
    println!("done with ui setup");
}
fn button_system(
    ui_materials: Res<UiMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Mutated<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
            }
        }
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&MapCoordinate, &mut Transform)>) {
    for (pos, mut transform) in q.iter_mut() {
        let (x, y) = pos.pixel_pos();
        transform.translation = Vec3::new(x, y, transform.translation.z);
    }
}

fn camera_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut q: Query<(&Camera, &mut Transform, &MapCamera)>
) {
    let mut translation: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    let mut moved = false;
    if keyboard_input.pressed(KeyCode::Left) {
        translation.x -= 3.0;
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        translation.x += 3.0;
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        translation.y -= 3.0;
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        translation.y += 3.0;
        moved = true;
    }
    if moved {
        for (camera, mut transform, _) in q.iter_mut() {
            transform.translation += translation;
        }
    }
}

fn tile_select_system(
    commands: &mut Commands,
    windows: ResMut<Windows>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    camera_query: Query<(&Camera, &Transform, &OrthographicProjection, &MapCamera)>,
    selected_query: Query<(Entity, &Selected)>,
    select_outline_query: Query<(Entity, &SelectOutline)>,
    world_map: Res<TileMap>,
    texture_atlas_handle: Res<TileTextureAtlas>,
) {
    let window = windows.get_primary().unwrap();
    for event in mouse_button_input_events.iter() {
        if event.button == MouseButton::Left && event.state == ElementState::Pressed {
            if let Some(cur_pos) = window.cursor_position() {
                for (camera, camera_transform, ortho_proj, _) in camera_query.iter() {
                    let world_pos = Vec2::new(
                        cur_pos.x + ortho_proj.left + camera_transform.translation.x,
                        cur_pos.y + ortho_proj.bottom + camera_transform.translation.y,
                    );
                    let coord = MapCoordinate::from_pixel_pos(world_pos);
                    if let Some(entity) = world_map.0.get(&coord) {
                        for (last_selected_entity, _) in selected_query.iter() {
                            println!("remove selected");
                            commands.remove_one::<Selected>(last_selected_entity);
                        }
                        for (select_outline, _) in select_outline_query.iter() {
                            println!("remove and despawn");
                            commands.remove_one::<Sprite>(select_outline);
                            commands.despawn(select_outline);
                        }
                        commands.insert_one(**entity, Selected {});
                        let sprite = TextureAtlasSprite::new(0);
                        let (selected_x, selected_y) = coord.pixel_pos();
                        commands
                            .spawn(SpriteSheetBundle {
                                texture_atlas: texture_atlas_handle.0.as_weak(),
                                sprite,
                                transform: Transform {
                                    translation: Vec3::new(selected_x, selected_y, 1.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with(SelectOutline)
                            .with(coord);
                    }
                }
            }
        }
    }
}

pub fn selected_info_system(
    selected_query: Query<(&MapCoordinate, &Selected)>,
    mut ui_element_query: Query<(&mut SelectedInfoText, &mut Text, &Transform)>,
) {
    for (coord, _) in selected_query.iter() {
        let coord_string = format!("{}, {}", coord.x, coord.y);
        for (mut info_text, mut text, transform) in ui_element_query.iter_mut() {
            println!("set text {}", coord_string);
            println!("{:?}", text);
            println!("{:?}", transform.translation);
            info_text.0 = coord_string.clone();
            text.sections[0].value = coord_string.clone();
        }
    }
}

pub fn selected_highlight_system(
    mut selected_query: Query<(&Selected, &mut TextureAtlasSprite)>
) {
    for (_, mut selected_sprite) in selected_query.iter_mut() {
        selected_sprite.color = Color::BLUE;
    }
}

pub fn camera_view_check(
    mut camera_query: Query<(&Camera, &Transform, &OrthographicProjection, &MapCamera)>,
    mut visible_query: Query<(&Transform, &mut Visible, &MapCoordinate)>,
) {
    for (camera, camera_transform, ortho_proj, _) in &mut camera_query.iter() {
        let left_bound = ortho_proj.left + camera_transform.translation.x - TILE_SIZE;
        let right_bound = ortho_proj.right + camera_transform.translation.x + TILE_SIZE;
        let bottom_bound = ortho_proj.bottom + camera_transform.translation.y - TILE_SIZE;
        let top_bound = ortho_proj.top + camera_transform.translation.y + TILE_SIZE;
        println!("{}, {}, {}, {}", left_bound, right_bound, bottom_bound, top_bound);
        for (transform, mut visible, _) in visible_query.iter_mut() {
            if transform.translation.x < left_bound
                || transform.translation.x > right_bound
                || transform.translation.y < bottom_bound
                || transform.translation.y > top_bound {
                    println!("{:?}", transform.translation);
                    visible.is_visible = false;
                } else {
                    visible.is_visible = true;
                }
        }
    }
}

fn main() {
    let mut world_setup = SystemStage::single(create_map.system());
    let mut ui_setup = SystemStage::single(setup_ui.system());

    App::build()
        .add_startup_system(setup_assets.system())
        .add_startup_stage("ui_setup", ui_setup)
        .add_startup_stage("world_setup", world_setup)
        .add_system(position_translation.system())
        .add_system(camera_movement_system.system())
        .add_system(camera_view_check.system())
        .add_system(tile_select_system.system())
        .add_system(selected_info_system.system())
        .add_system(button_system.system())
        .add_plugins(DefaultPlugins)
        .add_plugin(DiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
