pub mod ui;
pub mod tag;
pub mod map;
pub mod constant;
pub mod pops;
pub mod probability;
pub mod save;
pub mod input;
pub mod camera;
pub mod time;
pub mod province;
pub mod stage;
pub mod modifier;

use modifier::ModifierPlugin;
use province::ProvincePlugin;
// fuck yo namespace
use ui::*;
use map::*;
use tag::*;
use constant::*;
use pops::*;
use save::*;
use input::*;
use camera::*;
use time::TimePlugin;
use stage::*;
use macroquad::prelude::*;

#[macroquad::main("map-game")]
async fn main() {
    loop {
        clear_background(WHITE);
        let delta = get_frame_time();
        draw_circle(50.0, 50.0, 50.0, BLACK);
        next_frame().await
    }
}
