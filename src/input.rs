use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
    input::{
        ElementState,
        mouse::MouseButtonInput,
    },
};
use crate::*;

pub fn tile_hold_pressed_system(
    mut commands: Commands,
    windows: ResMut<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &Transform, &OrthographicProjection, &MapCamera)>,
    hold_pressed_query: Query<(Entity, &HoldPressed, &MapCoordinate)>,
    ui_container_query: Query<(&UiContainer, &Node, &Transform)>,
    world_map: Res<TileMap>,
) {
    let window = windows.get_primary().unwrap();
    if mouse_button_input.pressed(MouseButton::Left) {
        if let Some(cur_pos) = window.cursor_position() {
            // UI mouse occlusion
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
                    cur_pos.x + ortho_proj.left + camera_transform.translation.x,
                    cur_pos.y + ortho_proj.bottom + camera_transform.translation.y,
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
    world_map: Res<TileMap>,
    texture_atlas_handle: Res<TileTextureAtlas>,
) {
    let window = windows.get_primary().unwrap();
    for event in mouse_button_input_events.iter() {
        if event.button == MouseButton::Left && event.state == ElementState::Pressed {
            if let Some(cur_pos) = window.cursor_position() {
                // UI mouse occlusion
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
                        cur_pos.x + ortho_proj.left + camera_transform.translation.x,
                        cur_pos.y + ortho_proj.bottom + camera_transform.translation.y,
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

