use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection, WindowOrigin, CameraProjection},
    ecs::{DynamicBundle, EntityBuilder},
    input::mouse::MouseWheel,
    math::clamp,
};
use super::tag::*;
use super::map::{MapCoordinate, MapTileType, MapTile, TileMap};

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
    commands: &mut Commands,
    mut interaction_query: Query<
        (&UiButton, &Interaction),
        (Mutated<Interaction>, With<Button>),
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
}
pub struct UiButton(UiButtonType);

pub struct UiBuilder<'a> {
    commands: &'a mut Commands,
    materials: Res<'a, UiMaterials>,
    default_font: Handle<Font>,
    parent_stack: Vec<Entity>,
    children_stack: Vec<Vec<Entity>>,
    last_entity: Option<Entity>,
}


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
        let t = self.text_info(s);
        self.spawn(t);
        self
    }
    pub fn spawn_info_row(&mut self) -> &mut Self {
        self.spawn_info_row_material(UiMaterialType::BackgroundInfo)
    }
    pub fn spawn_info_row_material(&mut self, material_type: UiMaterialType) -> &mut Self {
        self.spawn(NodeBundle {
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
        })
    }
    pub fn spawn_info_box(&mut self) -> &mut Self {
        self.spawn_info_box_material(UiMaterialType::BackgroundInfo)
    }
    pub fn spawn_info_box_material(&mut self, material_type: UiMaterialType) -> &mut Self {
        self.spawn(NodeBundle {
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
            });
        self
    }
    pub fn spawn_button(&mut self) -> &mut Self {
        self.spawn_button_material(UiMaterialType::DefaultButton)
    }
    pub fn spawn_button_material(&mut self, material_type: UiMaterialType) -> &mut Self {
        let b = self.button(material_type);
        self.spawn(b)
    }
    pub fn spawn_text_button<T>(&mut self, ui_button_type: UiButtonType, s: T) -> &mut Self where T: Into<String> {
        self.spawn_text_button_material(ui_button_type, s, UiMaterialType::DefaultButton)
    }
    pub fn spawn_text_button_material<T>(&mut self, ui_button_type: UiButtonType, s: T, material_type: UiMaterialType) -> &mut Self where T: Into<String> {
        let t = self.text_info(s);
        let b = self.button(material_type);
        self.spawn(b)
            .with(UiButton(ui_button_type))
            .children()
            .spawn(t)
            .end_children()
    }
    pub fn text_info<T>(&mut self, s: T) -> TextBundle where T: Into<String> {
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

    pub fn button(&mut self, material_type: UiMaterialType) -> ButtonBundle {
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
    commands: &mut Commands,
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

pub fn setup_ui<'a>(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    ui_materials: Res<'a, UiMaterials>
) {
    let default_font = asset_server.load("fonts/DejaVuSansMono.ttf");
    commands.spawn((MapEditor::default(),));
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
        .spawn_info_row()
        .children()
        .spawn_text_button_material(
            UiButtonType::ChangeTileType(MapTileType::Water),
            "Water",
            UiMaterialType::WaterButton,
        )
        .spawn_text_button_material(
            UiButtonType::ChangeTileType(MapTileType::Land),
            "Land",
            UiMaterialType::LandButton,
        )
        .spawn_text_button_material(
            UiButtonType::ChangeTileType(MapTileType::None),
            "None",
            UiMaterialType::DefaultButton,
        )
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
        .end_children()
        .spawn_info_row()
        .children()
        .spawn_text_button_material(
            UiButtonType::BrushSizeType(1),
            "Brush 1",
            UiMaterialType::DefaultButton,
        )
        .spawn_text_button_material(
            UiButtonType::BrushSizeType(3),
            "Brush 3",
            UiMaterialType::DefaultButton,
        )
        .end_children()
        .end_children();
    println!("done with ui setup");
}
