use bevy::{core::FixedTimesteps, ecs::{schedule::ShouldRun, system::Command}, prelude::*};

use crate::{constant::DAY_TIMESTEP, stage::DayStage, tag::DateDisplay};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeEvent {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Date {
    pub day: usize,
    pub month: usize,
    pub year: usize,
    pub abs_day: usize,
    pub is_day: bool,
    pub is_week: bool,
    pub is_month: bool,
    pub is_year: bool,
}

impl Date {
    pub fn next_day(&mut self) {
        self.is_day = true;
        self.day += 1;
        self.abs_day += 1;
        if self.abs_day % 7 == 0 {
            self.is_week = true;
        }
        if self.day > 30 {
            self.month += 1;
            self.day = 1;
            self.is_month = true;
            if self.month > 12 {
                self.month = 1;
                self.year += 1;
                self.is_year = true;
            }
        }
    }

    pub fn days_after_doy(&self, day_of_year: DayOfYear) -> usize {
        let mut res = (self.month as isize - day_of_year.month as isize) * 30 + (self.day as isize - day_of_year.day as isize);
        if res < 0 {
            res += 360;
            if res < 0 {
                panic!("failed to diff dates: {:?} {:?}", self, day_of_year);
            } else {
                res as usize
            }
        } else {
            res as usize
        }
    }

    pub fn is_day_of_year(&self, day_of_year: DayOfYear) -> bool {
        self.days_after_doy(day_of_year) == 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DayOfYear {
    pub day: usize,
    pub month: usize,
}

pub struct DatesPerSecond(pub f32);
pub struct GameSpeed(pub usize);
pub struct GamePaused(pub bool);

fn time_system(
    mut frame: Local<usize>,
    mut date: ResMut<Date>,
    time: Res<Time>,
    game_speed: Res<GameSpeed>,
    mut date_texts: Query<(&DateDisplay, &mut Text)>,
    game_paused: Res<GamePaused>,
) {
    *frame = *frame + 1;
    date.is_day = false;
    date.is_week = false;
    date.is_month = false;
    date.is_year = false;
    println!("{}", *frame);
    if !game_paused.0 && *frame % 2usize.pow(11 - game_speed.0 as u32) == 0 {
        date.next_day();
        println!("next day! {:?}", *date);


        for (_, mut text) in date_texts.iter_mut() {
            text.sections[0].value = format!("year {}, {}/{}", date.year, date.month, date.day);
        }
    // } else {
    //     println!("not next day! {:?}", *date);
    }
}

pub fn day_run_criteria_system(
    day: Res<Date>,
) -> ShouldRun {
    println!("day_run_criteria_system?? {:?}", *day);
    if day.is_day {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .insert_resource(Date {
                abs_day: 1,
                day: 1,
                month: 1,
                year: 1,
                ..Default::default()
            })
            .insert_resource(GameSpeed(5))
            .insert_resource(GamePaused(false))
            .add_event::<TimeEvent>()
            .add_system_to_stage(DayStage::Main, time_system.system());
    }
}
