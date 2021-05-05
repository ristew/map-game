use bevy::{
    prelude::*,
};
use crate::{province::ProvinceInfos, time::{GamePaused, GameSpeed}};
use crate::time::Date;

use super::tag::*;
use super::map::{MapCoordinate, MapTileType, MapTile, HexMap};
use super::save::*;

const INFO_BAR_HEIGHT: f32 = 20.0;

pub struct UiMaterials {
    background_info: Handle<ColorMaterial>,
    default_button: Handle<ColorMaterial>,
    plains_button: Handle<ColorMaterial>,
    water_button: Handle<ColorMaterial>,
    desert_button: Handle<ColorMaterial>,
    mountain_button: Handle<ColorMaterial>,
}

impl UiMaterials {
    pub fn from_material_type(&self, material_type: UiMaterialType) -> Handle<ColorMaterial> {
        match material_type {
            UiMaterialType::BackgroundInfo => self.background_info.clone(),
            UiMaterialType::DefaultButton => self.default_button.clone(),
            UiMaterialType::PlainsButton => self.plains_button.clone(),
            UiMaterialType::WaterButton => self.water_button.clone(),
            UiMaterialType::DesertButton => self.desert_button.clone(),
            UiMaterialType::MountainButton => self.mountain_button.clone(),
        }
    }
}

pub enum UiMaterialType {
    BackgroundInfo,
    DefaultButton,
    PlainsButton,
    WaterButton,
    MountainButton,
    DesertButton,
}

#[derive(Debug)]
pub struct MapEditor {
    change_tile_type: Option<MapTileType>,
    brush_size: usize,
}

impl Default for MapEditor {
    fn default() -> Self {
        Self {
            change_tile_type: None,
            brush_size: 1,
        }
    }
}



pub fn change_button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&UiButton, &Interaction),
        (Changed<Interaction>, With<Button>),
    >,
    mut map_editor_query: Query<&mut MapEditor>,
) {
    for (ui_button, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            match ui_button.0 {
                UiButtonType::ChangeTileType(MapTileType::None) => {
                    println!("exit paint mode");
                    for mut map_editor in map_editor_query.iter_mut() {
                        map_editor.change_tile_type = None;
                    }
                },
                UiButtonType::ChangeTileType(typ) => {
                    println!("paint mode {:?}", typ);
                    for mut map_editor in map_editor_query.iter_mut() {
                        map_editor.change_tile_type = Some(typ);
                    }
                },
                UiButtonType::BrushSizeType(v) => {
                    for mut map_editor in map_editor_query.iter_mut() {
                        map_editor.brush_size += v as usize;
                    }
                },
                UiButtonType::SaveMap => {
                    commands.add(SaveMapCommand);
                }
            }
            // for (map_editor_entity, _) in map_editor_query.iter() {
            //     if *change_tile_type != ChangeTileType(MapTileType::None) {
            //         println!("paint mode");
            //         commands.insert_one(map_editor_entity, change_tile_type.clone());
            //     } else {
            //     }
            // }
        }
    }
}

pub fn map_editor_painting_system(
    map_editor_query: Query<&MapEditor>,
    hold_pressed_tile_query: Query<(Entity, &HoldPressed, &MapCoordinate)>,
    world_map: Res<HexMap>,
    info_box_mode: Res<InfoBoxMode>,
    mut map_tile_query: Query<&mut MapTile>,
) {
    if *info_box_mode == InfoBoxMode::MapDrawingMode {
        for map_editor in map_editor_query.iter() {
            if let Some(change_tile_type) = map_editor.change_tile_type {
                let mut change_entities = Vec::new();
                for (e, _, coord) in hold_pressed_tile_query.iter() {
                    change_entities.push(e);
                    for ent in world_map.neighbors_in_radius_iter(*coord, map_editor.brush_size as isize) {
                        change_entities.push(*ent);
                    }
                }
                for e in change_entities {
                    if let Ok(mut map_tile) = map_tile_query.get_mut(e) {
                        println!("change tile type: {:?}", map_tile.tile_type);
                        map_tile.tile_type = change_tile_type;
                    }
                }
            }
        }
    }
}

pub struct SelectedInfoText;
#[derive(Debug, Clone, PartialEq)]
pub enum UiButtonType {
    ChangeTileType(MapTileType),
    BrushSizeType(isize),
    SaveMap,
}
pub struct UiButton(UiButtonType);

pub struct UiBuilder<'a> {
    materials: Res<'a, UiMaterials>,
    default_font: Handle<Font>,
}


impl <'a> UiBuilder<'a> {
    pub fn info_row(&self) -> NodeBundle {
        self.info_row_material(UiMaterialType::BackgroundInfo)
    }
    pub fn info_row_material(&self, material_type: UiMaterialType) -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Auto,
                    height: Val::Px(16.0),
                },
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                align_content: AlignContent::FlexStart,
                justify_content: JustifyContent::FlexStart,
                ..Default::default()
            },
            material: self.materials.from_material_type(material_type),
            ..Default::default()
        }
    }
    pub fn info_box(&self) -> NodeBundle {
        self.info_box_material(UiMaterialType::BackgroundInfo)
    }
    pub fn info_box_material(&self, material_type: UiMaterialType) -> NodeBundle {
        NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Px(200.0),
                    height: Val::Percent(50.0)
                },
                padding: Rect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::ColumnReverse,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::FlexStart,
                align_content: AlignContent::FlexStart,
                justify_content: JustifyContent::FlexStart,
                ..Default::default()
            },
            material: self.materials.from_material_type(material_type),
            ..Default::default()
        }
    }
    pub fn info_bar(&self, window_top: f32) -> NodeBundle {
        self.info_bar_material(UiMaterialType::BackgroundInfo, window_top)
    }
    pub fn info_bar_material(&self, material_type: UiMaterialType, window_top: f32) -> NodeBundle {
        println!("window top: {}", window_top);
        NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Px(INFO_BAR_HEIGHT)
                },
                padding: Rect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Stretch,
                align_content: AlignContent::FlexStart,
                justify_content: JustifyContent::FlexStart,
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(0.0),
                    bottom: Val::Px(window_top - INFO_BAR_HEIGHT),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: self.materials.from_material_type(material_type),
            ..Default::default()
        }
    }
    pub fn button(&self) -> ButtonBundle {
        self.button_material(UiMaterialType::DefaultButton)
    }
    // pub fn spawn_text_button<T>(&'a mut self, ui_button_type: UiButtonType, s: T) -> NodeBundle where T: Into<String> {
    //     self.text_button_material(ui_button_type, s, UiMaterialType::DefaultButton)
    // }
    // pub fn text_button_material<T>(&'a mut self, ui_button_type: UiButtonType, s: T, material_type: UiMaterialType) -> &mut Self where T: Into<String> {
    //     let t = self.text_info(s);
    //     let b = self.button(material_type);
    //     self.spawn(b)
    //         .with(UiButton(ui_button_type))
    //         .children()
    //         .spawn(t)
    //         .end_children()
    // }
    pub fn text_info<T>(&self, s: T) -> TextBundle where T: Into<String> {
        TextBundle {
            text: Text::with_section(
                s,
                TextStyle {
                    font: self.default_font.clone(),
                    font_size: 12.0,
                    color: Color::BLACK,
                },
                Default::default(),
            ),
            style: Style {
                // size: Size {
                //     width: Val::Auto,
                //     height: Val::Px(16.0),
                // },
                max_size: Size::new(Val::Px(200.0), Val::Auto),
                flex_wrap: FlexWrap::Wrap,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn button_material(&self, material_type: UiMaterialType) -> ButtonBundle {
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(16.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: self.materials.from_material_type(material_type),
            ..Default::default()
        }
    }
}

pub fn setup_ui_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(UiMaterials {
        background_info: materials.add(Color::rgb(0.7, 0.7, 0.4).into()),
        default_button: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
        plains_button: materials.add(MapTileType::Plains.color().into()),
        water_button: materials.add(MapTileType::Water.color().into()),
        desert_button: materials.add(MapTileType::Desert.color().into()),
        mountain_button: materials.add(MapTileType::Mountain.color().into()),
    });
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InfoBoxMode {
    MapDrawingMode,
    ProvinceInfoMode,
}

pub fn ui_info_box(
    commands: &mut Commands,
    builder: &UiBuilder,
) -> Entity {
    let mut info_box = commands
        .spawn_bundle(builder.info_box());
    info_box
        .insert(UiContainer)
        .insert(InfoBoxMode::MapDrawingMode)
        .with_children(|parent| {
            parent.spawn_bundle(builder.info_row()).with_children(|parent| {
                parent.spawn_bundle(builder.button_material(UiMaterialType::WaterButton))
                    .insert(UiButton(UiButtonType::ChangeTileType(MapTileType::Water)))
                    .with_children(|parent| {
                        parent.spawn_bundle(builder.text_info("Water"));
                    });
                parent.spawn_bundle(builder.button_material(UiMaterialType::PlainsButton))
                    .insert(UiButton(UiButtonType::ChangeTileType(MapTileType::Plains)))
                    .with_children(|parent| {
                        parent.spawn_bundle(builder.text_info("Plains"));
                    });
                parent.spawn_bundle(builder.button_material(UiMaterialType::MountainButton))
                    .insert(UiButton(UiButtonType::ChangeTileType(MapTileType::Mountain)))
                    .with_children(|parent| {
                        parent.spawn_bundle(builder.text_info("Mountain"));
                    });
                parent.spawn_bundle(builder.button_material(UiMaterialType::DesertButton))
                    .insert(UiButton(UiButtonType::ChangeTileType(MapTileType::Desert)))
                    .with_children(|parent| {
                        parent.spawn_bundle(builder.text_info("Desert"));
                    });
            });
            parent.spawn_bundle(builder.text_info(""))
                .insert(InfoTag::BrushSize);
            parent.spawn_bundle(builder.info_row()).with_children(|parent| {
                parent.spawn_bundle(builder.button())
                    .insert(UiButton(UiButtonType::BrushSizeType(-1)))
                    .with_children(|parent| {
                        parent.spawn_bundle(builder.text_info("-1"));
                    });
                parent.spawn_bundle(builder.button())
                    .insert(UiButton(UiButtonType::BrushSizeType(1)))
                    .with_children(|parent| {
                        parent.spawn_bundle(builder.text_info("+1"));
                    });
            });
            parent.spawn_bundle(builder.button())
                .insert(UiButton(UiButtonType::SaveMap))
                .with_children(|parent| {
                    parent.spawn_bundle(builder.text_info("Save"));
                });
        });
    info_box.id()
}

pub fn province_info_box(
    commands: &mut Commands,
    builder: &UiBuilder,
) -> Entity {
    let mut province_info_box = commands
        .spawn_bundle(builder.info_box());
    province_info_box
        .insert(UiContainer)
        .insert(InfoBoxMode::ProvinceInfoMode)
        .with_children(|parent| {
            parent.spawn_bundle(builder.text_info(""))
                .insert(InfoTag::SelectedProvinceName);
            parent.spawn_bundle(builder.text_info(""))
                .insert(InfoTag::SelectedProvincePopulation);
        })
        ;
    province_info_box.id()
}

fn info_box_system(
    mut info_boxes: Query<(&InfoBoxMode, &mut Style)>,
    info_box_mode: Res<InfoBoxMode>,
) {
    for (box_mode, mut style) in info_boxes.iter_mut() {
        if *box_mode == *info_box_mode {
            style.display = Display::Flex;
        } else {
            style.display = Display::None;
        }
    }
}

pub fn info_tag_system(
    mut info_tag_query: Query<(&InfoTag, &mut Text)>,
    selected_query: Query<(&MapCoordinate, &MapTile, &Selected)>,
    map_editor_query: Query<&MapEditor>,
    province_infos: Res<ProvinceInfos>,
    date: Res<Date>,
    game_speed: Res<GameSpeed>,
    game_paused: Res<GamePaused>,
) {
    for (info_tag, mut text) in info_tag_query.iter_mut() {
        let info_string = match *info_tag {
            InfoTag::ProvincePopulation(coord) => format!("Total population: {}", province_infos.0.get(&coord).unwrap().total_population),
            InfoTag::ProvinceName(coord) => format!("{:?}", coord),
            InfoTag::SelectedProvinceName => {
                if let Some((coord, map_tile, _)) = selected_query.iter().next() {
                    format!("{:?}\n{:?}", coord, map_tile.tile_type)
                } else {
                    "Select a province".to_string()
                }
            },
            InfoTag::SelectedProvincePopulation => {
                if let Some((coord, map_tile, _)) = selected_query.iter().next() {
                    // monads and strife
                    format!("population: {:?}", province_infos.0.get(&coord).map(|pinfo| pinfo.total_population).unwrap_or(0))
                } else {
                    "".to_string()
                }
            },
            InfoTag::BrushSize => format!("{}", map_editor_query.iter().next().map(|me| me.brush_size).unwrap_or(0)),
            InfoTag::DateDisplay => format!("({}{}) year {}, {}/{}", game_speed.0, game_paused.0.then(|| "paused").unwrap_or(""), date.year, date.month, date.day),
            t => format!("{:?}", t),
        };
        text.sections[0].value = info_string;
    }
}

#[derive(Debug, Clone, Copy,)]
pub enum InfoTag {
    ProvincePopulation(MapCoordinate),
    ProvinceName(MapCoordinate),
    DateDisplay,
    SelectedProvinceName,
    SelectedProvincePopulation,
    BrushSize,
}
// descriptor to set ui and create changeable text objects
pub enum UiComponent {
    InfoText(InfoTag)
}

impl UiComponent {
    fn render(
        &self,
        builder: &UiBuilder,
        mut commands: Commands,
    ) -> Entity {
        let mut base = commands.spawn();
        match self {
            Self::InfoText(info_tag) => {
                base.insert_bundle(builder.text_info(""))
                    .insert(info_tag.clone());
            }
        };
        base.id()
    }
}
pub struct InfoBoxChangeCommand;

pub fn ui_info_bar(
    commands: &mut Commands,
    builder: &UiBuilder,
    window_top: f32,
) -> Entity {
    let mut info_bar = commands
        .spawn_bundle(builder.info_bar(window_top));
    info_bar
        .insert(UiContainer)
        .insert(InfoBar)
        .with_children(|parent| {
            parent.spawn_bundle(builder.text_info(""))
                .insert(InfoTag::DateDisplay);
        });

    info_bar.id()
}

fn info_bar_position_system(
    windows: Res<Windows>,
    mut info_bar_query: Query<(&mut Style, &InfoBar)>,
) {
    for (mut style, _) in info_bar_query.iter_mut() {
        style.position.bottom = Val::Px(windows.get_primary().unwrap().height() - INFO_BAR_HEIGHT);
    }
}

// TODO: fix this upstream?
fn issue_1135_system(
    mut text_style_visible_query: Query<(&Text, &Node, &mut Visible)>,
) {
    for (text, node, mut visible) in text_style_visible_query.iter_mut() {
        if node.size == Vec2::ZERO {
            visible.is_visible = false;
        } else {
            visible.is_visible = true;
        }
    }
}

pub fn setup_ui<'a>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_materials: Res<'a, UiMaterials>,
    windows: Res<Windows>,
) {
    let default_font = asset_server.load("fonts/DejaVuSansMono.ttf");
    commands.spawn_bundle((MapEditor::default(),));
    let builder = UiBuilder {
        materials: ui_materials,
        default_font: default_font,
    };

    ui_info_box(&mut commands, &builder);
    province_info_box(&mut commands, &builder);
    let window = windows.get_primary().unwrap();
    ui_info_bar(&mut commands, &builder, window.height());
    println!("done with ui setup");
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let ui_setup = SystemStage::single(setup_ui.system());
        app
            .add_startup_system(setup_ui_assets.system())
            .add_startup_stage("ui_setup", ui_setup)
            .insert_resource(InfoBoxMode::ProvinceInfoMode)
            .add_system(info_tag_system.system())
            .add_system(change_button_system.system())
            .add_system(info_box_system.system())
            .add_system(issue_1135_system.system())
            .add_system(info_bar_position_system.system());
    }
}
