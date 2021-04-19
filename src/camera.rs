use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
};
use crate::*;

pub fn camera_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut q: Query<(&Camera, &mut Transform, &MapCamera)>
) {
    let mut translation: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    let camera_move_speed = 4.0;
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

pub fn camera_view_check(
    camera_query: Query<(&Camera, &Transform, &OrthographicProjection, &MapCamera)>,
    mut visible_query: Query<(&Transform, &mut Visible, &MapCoordinate)>,
) {
    for (_, camera_transform, ortho_proj, _) in &mut camera_query.iter() {
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
