use std::time::Duration;

use bevy::{input::{
        ElementState,
        mouse::MouseButtonInput,
    }, prelude::*, render::{camera::{ActiveCameras, Camera, OrthographicProjection}, draw::OutsideFrustum}};
use bevy_egui::EguiContext;
use crate::{camera::ZoomLevel, map::{HexMap, MapCoordinate, TileTextureAtlas}, tag::{HoldPressed, MapCamera, SelectOutline, Selected, UiContainer}, time::{GamePaused, GameSpeed}, ui::InfoBoxMode};
use crate::time::DateTimer;


pub fn tile_hold_pressed_system(
    mut commands: Commands,
    windows: ResMut<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &Transform, &OrthographicProjection, &MapCamera)>,
    hold_pressed_query: Query<(Entity, &HoldPressed, &MapCoordinate)>,
    ui_container_query: Query<(&UiContainer, &Node, &Transform)>,
    world_map: Res<HexMap>,
    egui_ctx: Res<EguiContext>,
) {
    let window = windows.get_primary().unwrap();
    if mouse_button_input.pressed(MouseButton::Left) {
        if let Some(cur_pos) = window.cursor_position() {
            // UI mouse occlusion
            if egui_ctx.ctx().is_pointer_over_area() {
                return;
            }
            for (_, node, transform) in ui_container_query.iter() {
                let nx = transform.translation.x;
                let ny = transform.translation.y;

                if cur_pos.x >= nx - node.size.x / 2.0 &&
                    cur_pos.y >= ny - node.size.y / 2.0 &&
                    cur_pos.x <= nx + node.size.x / 2.0 &&
                    cur_pos.y <= ny + node.size.y / 2.0 {
                        return;
                    }
            }
            for (_, camera_transform, ortho_proj, _) in camera_query.iter() {
                let world_pos = Vec2::new(
                    (cur_pos.x + ortho_proj.left) * camera_transform.scale.x + camera_transform.translation.x,
                    (cur_pos.y + ortho_proj.bottom) * camera_transform.scale.y + camera_transform.translation.y,
                );
                let coord = MapCoordinate::from_pixel_pos(world_pos);
                if let Some(entity) = world_map.0.get(&coord) {
                    for (last_hold_pressed_entity, _, held_coord) in hold_pressed_query.iter() {
                        if coord != *held_coord {
                            commands.entity(last_hold_pressed_entity).remove::<HoldPressed>();
                        }
                    }
                    commands.entity(**entity).insert(HoldPressed {});
                }
            }
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        for (last_hold_pressed_entity, _, _) in hold_pressed_query.iter() {
            commands.entity(last_hold_pressed_entity).remove::<HoldPressed>();
        }
    }
}

pub fn tile_select_system(
    mut commands: Commands,
    windows: ResMut<Windows>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    camera_query: Query<(&Camera, &Transform, &OrthographicProjection, &MapCamera)>,
    selected_query: Query<(Entity, &Selected)>,
    select_outline_query: Query<(Entity, &SelectOutline)>,
    ui_container_query: Query<(&UiContainer, &Node, &Transform)>,
    world_map: Res<HexMap>,
    texture_atlas_handle: Res<TileTextureAtlas>,
    egui_ctx: Res<EguiContext>,
) {
    let window = windows.get_primary().unwrap();
    for event in mouse_button_input_events.iter() {
        if event.button == MouseButton::Left && event.state == ElementState::Pressed {
            if let Some(cur_pos) = window.cursor_position() {
                // UI mouse occlusion
                if egui_ctx.ctx().is_pointer_over_area() {
                    return;
                }
                // for (_, node, transform) in ui_container_query.iter() {
                //     let nx = transform.translation.x;
                //     let ny = transform.translation.y;

                //     if cur_pos.x >= nx - node.size.x / 2.0 &&
                //         cur_pos.y >= ny - node.size.y / 2.0 &&
                //         cur_pos.x <= nx + node.size.x / 2.0 &&
                //         cur_pos.y <= ny + node.size.y / 2.0 {
                //             return;
                //         }
                // }
                for (_, camera_transform, ortho_proj, _) in camera_query.iter() {
                    let world_pos = Vec2::new(
                        (cur_pos.x + ortho_proj.left) * camera_transform.scale.x + camera_transform.translation.x,
                        (cur_pos.y + ortho_proj.bottom) * camera_transform.scale.y + camera_transform.translation.y,
                    );
                    let coord = MapCoordinate::from_pixel_pos(world_pos);
                    if let Some(entity) = world_map.0.get(&coord) {
                        for (last_selected_entity, _) in selected_query.iter() {
                            println!("remove selected");
                            commands.entity(last_selected_entity).remove::<Selected>();
                        }
                        for (select_outline, _) in select_outline_query.iter() {
                            println!("remove and despawn");
                            commands.entity(select_outline)
                                    .remove::<Sprite>()
                                    .despawn();
                        }
                        commands.entity(**entity).insert(Selected);
                        let sprite = TextureAtlasSprite::new(0);
                        let (selected_x, selected_y) = coord.pixel_pos();
                        commands
                            .spawn_bundle(SpriteSheetBundle {
                                texture_atlas: texture_atlas_handle.0.as_weak(),
                                sprite,
                                transform: Transform {
                                    translation: Vec3::new(selected_x, selected_y, 1.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(SelectOutline)
                            .insert(coord);
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

pub fn camera_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut q: Query<(&Camera, &mut Transform, &MapCamera)>,
    zoom_level: Res<ZoomLevel>,
    // mut q: Query<(&Camera, &mut Transform, &MapCamera)>
) {
    let mut translation: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    let camera_move_speed = 4.0 * zoom_level.0;
    let mut moved = false;
    if keyboard_input.pressed(KeyCode::Left) {
        translation.x -= camera_move_speed;
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        translation.x += camera_move_speed;
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        translation.y -= camera_move_speed;
        moved = true;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        translation.y += camera_move_speed;
        moved = true;
    }
    if moved {
        for (_, mut transform, _) in q.iter_mut() {
            transform.translation += translation;
        }
    }
}

pub fn info_box_change_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut info_box_mode: ResMut<InfoBoxMode>,
) {
    if keyboard_input.pressed(KeyCode::B) {
        *info_box_mode = InfoBoxMode::MapDrawingMode;
    }
    if keyboard_input.pressed(KeyCode::N) {
        *info_box_mode = InfoBoxMode::ProvinceInfoMode;
    }
}

pub fn change_speed_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut date_timer: ResMut<DateTimer>,
    mut game_speed: ResMut<GameSpeed>,
    mut game_paused: ResMut<GamePaused>,
) {
    if keyboard_input.just_pressed(KeyCode::LBracket) {
        game_speed.0 = game_speed.0.max(1) - 1;
    }
    if keyboard_input.just_pressed(KeyCode::RBracket) {
        game_speed.0 = game_speed.0.min(6) + 1;
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        game_paused.0 = !game_paused.0;
    }
    let duration = Duration::from_millis(2u64.pow(11 - game_speed.0 as u32));
    date_timer.0.set_duration(duration);
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system(change_speed_system.system())
            .add_system(camera_movement_system.system())
            .add_system(tile_select_system.system())
            .add_system(tile_hold_pressed_system.system())
            .add_system(info_box_change_system.system());
    }
}
