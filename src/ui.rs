use std::rc::Rc;
use std::cell::RefCell;
use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection, WindowOrigin, CameraProjection},
    input::mouse::MouseWheel,
    ecs::{
        component::Component, system::EntityCommands
    },
};
use super::tag::*;
use super::map::{MapCoordinate, MapTileType, MapTile, TileMap};
use super::save::*;

pub struct UiMaterials {
    background_info: Handle<ColorMaterial>,
    default_button: Handle<ColorMaterial>,
    land_button: Handle<ColorMaterial>,
    water_button: Handle<ColorMaterial>,
}

impl UiMaterials {
    pub fn from_material_type(&self, material_type: UiMaterialType) -> Handle<ColorMaterial> {
        match material_type {
            UiMaterialType::BackgroundInfo => self.background_info.clone(),
            UiMaterialType::DefaultButton => self.default_button.clone(),
            UiMaterialType::LandButton => self.land_button.clone(),
            UiMaterialType::WaterButton => self.water_button.clone(),
        }
    }
}

pub enum UiMaterialType {
    BackgroundInfo,
    DefaultButton,
    LandButton,
    WaterButton,
}
pub fn selected_info_system(
    selected_query: Query<(&MapCoordinate, &Selected)>,
    mut ui_element_query: Query<(&mut SelectedInfoText, &mut Text, &Transform)>,
) {
    for (coord, _) in selected_query.iter() {
        let coord_string = format!("{}, {}", coord.x, coord.y);
        for (mut info_text, mut text, transform) in ui_element_query.iter_mut() {
            info_text.0 = coord_string.clone();
            text.sections[0].value = coord_string.clone();
        }
    }
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

pub struct ZoomLevel(f32);

pub fn change_zoom_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut zoom_level: ResMut<ZoomLevel>,
    mut camera_query: Query<(&Camera, &mut OrthographicProjection, &MapCamera)>,
) {
    for event in mouse_wheel_events.iter() {
        //zoom_level.0 = clamp(zoom_level.0 + event.y * 0.1, 0.5, 2.0);
        zoom_level.0 += event.y * 0.1;
        println!("zoom_level.0 {}", zoom_level.0);
        for (_, mut projection, _) in camera_query.iter_mut() {
            projection.scale = zoom_level.0;
            println!("scale projection {:?}", projection.get_projection_matrix());
        }
    }
}

pub fn camera_zoom_system(
    zoom_level: Res<ZoomLevel>,
    windows: Res<Windows>,
) {
    // for (_, mut projection) in camera_query.iter_mut() {
    //     let window = windows.get_primary().unwrap();
    //     let nw = window.width() / zoom_level.0;
    //     let nh = window.height() / zoom_level.0;
    //     projection.update(nw, nh);
    // }
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
                    println!("paint mode");
                    for mut map_editor in map_editor_query.iter_mut() {
                        map_editor.change_tile_type = Some(typ);
                    }
                },
                UiButtonType::BrushSizeType(v) => {
                    for mut map_editor in map_editor_query.iter_mut() {
                        map_editor.brush_size = v;
                    }
                },
                UiButtonType::SaveMap => {
                    commands.add(SaveCommand);
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
    world_map: Res<TileMap>,
    mut map_tile_query: Query<&mut MapTile>,
) {
    for map_editor in map_editor_query.iter() {
        if let Some(change_tile_type) = map_editor.change_tile_type {
            let mut change_entities = Vec::new();
            for (e, _, coord) in hold_pressed_tile_query.iter() {
                change_entities.push(e);
                if map_editor.brush_size > 1 {
                    for ent in world_map.neighbors_iter(*coord) {
                        change_entities.push(*ent);
                    }
                }
            }
            for e in change_entities {
                if let Ok(mut map_tile) = map_tile_query.get_mut(e) {
                    map_tile.tile_type = change_tile_type;
                }
            }
        }
    }
}

pub struct SelectedInfoText(String);
#[derive(Debug, Clone, PartialEq)]
pub enum UiButtonType {
    ChangeTileType(MapTileType),
    BrushSizeType(usize),
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
                    width: Val::Percent(20.0),
                    height: Val::Percent(50.0)
                },
                padding: Rect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Stretch,
                align_content: AlignContent::FlexStart,
                justify_content: JustifyContent::FlexStart,
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
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(UiMaterials {
        background_info: materials.add(Color::rgb(0.7, 0.7, 0.4).into()),
        default_button: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
        land_button: materials.add(Color::rgb(0.2, 0.8, 0.2).into()),
        water_button: materials.add(Color::rgb(0.1, 0.2, 0.8).into()),
    });
    commands.insert_resource(ZoomLevel(1.0));
}

pub fn ui_info_box(
    mut commands: Commands,
    builder: UiBuilder,
) -> Entity {
    let mut info_box = commands
        .spawn_bundle(builder.info_box());
    info_box.insert(UiContainer);
    info_box.with_children(|parent| {
        parent.spawn_bundle(builder.text_info("Select a tile"))
            .insert(SelectedInfoText("Select a tile".to_string()));
        parent.spawn_bundle(builder.info_row()).with_children(|parent| {
            parent.spawn_bundle(builder.button_material(UiMaterialType::WaterButton))
                .insert(UiButton(UiButtonType::ChangeTileType(MapTileType::Water)))
                .with_children(|parent| {
                    parent.spawn_bundle(builder.text_info("Water"));
                });
            parent.spawn_bundle(builder.button_material(UiMaterialType::LandButton))
                .insert(UiButton(UiButtonType::ChangeTileType(MapTileType::Land)))
                .with_children(|parent| {
                    parent.spawn_bundle(builder.text_info("Land"));
                });
        });
        parent.spawn_bundle(builder.info_row()).with_children(|parent| {
            parent.spawn_bundle(builder.button())
                .insert(UiButton(UiButtonType::BrushSizeType(1)))
                .with_children(|parent| {
                    parent.spawn_bundle(builder.text_info("Brush 1"));
                });
            parent.spawn_bundle(builder.button())
                .insert(UiButton(UiButtonType::BrushSizeType(3)))
                .with_children(|parent| {
                    parent.spawn_bundle(builder.text_info("Brush 3"));
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

pub fn setup_ui<'a>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_materials: Res<'a, UiMaterials>
) {
    let default_font = asset_server.load("fonts/DejaVuSansMono.ttf");
    commands.spawn_bundle((MapEditor::default(),));
    let mut builder = UiBuilder {
        materials: ui_materials,
        default_font: default_font,
    };

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MapCamera);
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(UiCamera);
    ui_info_box(commands, builder);
        // .children()
        // .spawn_text_info("Select a tile")
        // .with()
        // .spawn_text_info("Tile details")
        // .spawn_text_info("Select a tile")
        // .spawn_info_row()
        // .children()
        // .spawn_text_button_material(
        //     UiButtonType::ChangeTileType(MapTileType::Water),
        //     "Water",
        //     UiMaterialType::WaterButton,
        // )
        // .spawn_text_button_material(
        //     UiButtonType::ChangeTileType(MapTileType::Land),
        //     "Land",
        //     UiMaterialType::LandButton,
        // )
        // .spawn_text_button_material(
        //     UiButtonType::ChangeTileType(MapTileType::None),
        //     "None",
        //     UiMaterialType::DefaultButton,
        // )
        // .spawn_button_material(UiMaterialType::WaterButton)
        // .with(ChangeTileType(MapTileType::Water))
        // .children()
        // .spawn_text_info("Water")
        // .end_children()
        // .spawn_button_material(UiMaterialType::LandButton)
        // .with(ChangeTileType(MapTileType::Land))
        // .children()
        // .spawn_text_info("Land")
        // .end_children()
        // .spawn_button_material(UiMaterialType::DefaultButton)
        // .with(ChangeTileType(MapTileType::None))
        // .children()
        // .spawn_text_info("None")
        // .end_children()
        // .end_children()
        // .spawn_info_row()
        // .children()
        // .spawn_text_button_material(
        //     UiButtonType::BrushSizeType(1),
        //     "Brush 1",
        //     UiMaterialType::DefaultButton,
        // )
        // .spawn_text_button_material(
        //     UiButtonType::BrushSizeType(3),
        //     "Brush 3",
        //     UiMaterialType::DefaultButton,
        // )
        // .end_children()
        // .spawn_text_button(
        //     UiButtonType::SaveMap,
        //     "Save",
        // )
        // .end_children();
    println!("done with ui setup");
}
