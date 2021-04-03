extern crate bevy;

pub mod ui;
pub mod tag;
pub mod map;
pub mod constant;
pub mod pops;
pub mod probability;

use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection, WindowOrigin, CameraProjection},
    diagnostic::{ FrameTimeDiagnosticsPlugin, DiagnosticsPlugin },
    input::{
        ElementState,
        mouse::{MouseButtonInput},
    },
};
// fuck yo namespace
use ui::*;
use map::*;
use tag::*;
use constant::*;
use pops::*;

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

fn tile_hold_pressed_system(
    commands: &mut Commands,
    windows: ResMut<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &Transform, &OrthographicProjection, &MapCamera)>,
    hold_pressed_query: Query<(Entity, &HoldPressed, &MapCoordinate)>,
    world_map: Res<TileMap>,
) {
    let window = windows.get_primary().unwrap();
    if mouse_button_input.pressed(MouseButton::Left) {
        if let Some(cur_pos) = window.cursor_position() {
            for (camera, camera_transform, ortho_proj, _) in camera_query.iter() {
                let world_pos = Vec2::new(
                    cur_pos.x + ortho_proj.left + camera_transform.translation.x,
                    cur_pos.y + ortho_proj.bottom + camera_transform.translation.y,
                );
                let coord = MapCoordinate::from_pixel_pos(world_pos);
                if let Some(entity) = world_map.0.get(&coord) {
                    for (last_hold_pressed_entity, _, held_coord) in hold_pressed_query.iter() {
                        if coord != *held_coord {
                            commands.remove_one::<HoldPressed>(last_hold_pressed_entity);
                        }
                    }
                    commands.insert_one(**entity, HoldPressed {});
                }
            }
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        for (last_hold_pressed_entity, _, _) in hold_pressed_query.iter() {
            commands.remove_one::<HoldPressed>(last_hold_pressed_entity);
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
        for (transform, mut visible, _) in visible_query.iter_mut() {
            if transform.translation.x < left_bound
                || transform.translation.x > right_bound
                || transform.translation.y < bottom_bound
                || transform.translation.y > top_bound {
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
        .add_startup_system(setup_ui_assets.system())
        .add_startup_stage("ui_setup", ui_setup)
        .add_startup_stage("world_setup", world_setup)
        .add_system(camera_movement_system.system())
        .add_system(camera_view_check.system())
        .add_system(tile_select_system.system())
        .add_system(tile_hold_pressed_system.system())
        .add_system(selected_info_system.system())
        .add_system(change_button_system.system())
        .add_system(change_zoom_system.system())
        .add_system(camera_zoom_system.system())
        .add_system(map_editor_painting_system.system())
        .add_system(economic_system::<FarmerPopulation, FarmingResource>.system())
        .add_plugins(DefaultPlugins)
        .add_plugin(MapPlugin)
        .add_plugin(DiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
