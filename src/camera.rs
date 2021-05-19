use std::cmp::max;

use bevy::{input::mouse::MouseWheel, prelude::*, render::{camera::{Camera, OrthographicProjection, CameraProjection, ActiveCameras},
             draw::OutsideFrustum}, sprite::SpriteSettings};

use crate::{map::MapCoordinate, tag::MapCamera};
use crate::constant::*;
use crate::tag::*;

pub fn camera_view_check(
    camera_query: Query<(&Camera, &Transform, &OrthographicProjection, &MapCamera), Changed<Transform>>,
    mut visible_query: Query<(&Transform, &mut Visible, &MapCoordinate)>,
) {
    for (_, camera_transform, ortho_proj, _) in &mut camera_query.iter() {
        let left_bound = ortho_proj.left * camera_transform.scale.x + camera_transform.translation.x - TILE_SIZE_X;
        let right_bound = ortho_proj.right * camera_transform.scale.x + camera_transform.translation.x + TILE_SIZE_X;
        let bottom_bound = ortho_proj.bottom * camera_transform.scale.y + camera_transform.translation.y - TILE_SIZE_Y;
        let top_bound = ortho_proj.top * camera_transform.scale.y + camera_transform.translation.y + TILE_SIZE_Y;
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

pub struct ZoomLevel(pub f32);

pub fn change_zoom_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut zoom_level: ResMut<ZoomLevel>,
    mut camera_query: Query<(&Camera, &mut OrthographicProjection, &mut Transform, &MapCamera)>,
) {
    for event in mouse_wheel_events.iter() {
        //zoom_level.0 = clamp(zoom_level.0 + event.y * 0.1, 0.5, 2.0);
        zoom_level.0 = 0.5f32.max(zoom_level.0 - event.y * 0.1);
        for (_, mut projection, mut transform, _) in camera_query.iter_mut() {
            projection.scale = zoom_level.0;
            transform.scale = Vec2::new(zoom_level.0, zoom_level.0).extend(1.0);
        }
    }
}

fn setup_cameras_system(
    mut commands: Commands,
) {
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(UiCamera);
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MapCamera);
}


pub fn active_cameras_system(
    mut active_cameras: ResMut<ActiveCameras>,
    ui_camera: Query<Entity, With<UiCamera>>,
) {
    for mut active_camera in active_cameras.iter_mut() {
        for ui_cam in ui_camera.iter() {
            if active_camera.entity == Some(ui_cam) {
                println!("set camera not active?");
                active_camera.entity = None;
            }
        }
    }
}


pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .insert_resource(ZoomLevel(1.0))
            .add_startup_system(setup_cameras_system.system())
            .add_system(camera_view_check.system())
            .add_system(change_zoom_system.system());
    }
}
