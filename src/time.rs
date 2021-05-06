use bevy::prelude::*;

use crate::tag::DateDisplay;

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
}

#[derive(Debug, Clone, Copy)]
pub struct DayOfYear {
    pub day: usize,
    pub month: usize,
}

pub struct DateTimer(pub Timer);
pub struct GameSpeed(pub usize);
pub struct GamePaused(pub bool);

fn time_system(
    mut date: ResMut<Date>,
    mut date_timer: ResMut<DateTimer>,
    time: Res<Time>,
    mut date_texts: Query<(&DateDisplay, &mut Text)>,
    game_paused: Res<GamePaused>,
) {
    date.is_day = false;
    date.is_week = false;
    date.is_month = false;
    date.is_year = false;
    if !game_paused.0 && date_timer.0.tick(time.delta()).just_finished() {
        date.next_day();

        for (_, mut text) in date_texts.iter_mut() {
            text.sections[0].value = format!("year {}, {}/{}", date.year, date.month, date.day);
        }
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
            .insert_resource(DateTimer(Timer::from_seconds(0.02, true)))
            .insert_resource(GameSpeed(4))
            .insert_resource(GamePaused(false))
            .add_event::<TimeEvent>()
            .add_system(time_system.system());
    }
}
