use bevy::prelude::*;

use crate::tag::DateDisplay;

#[derive(Debug, Clone, Copy)]
pub enum TimeEvent {
    Day,
    Month,
    Year,
}

#[derive(Debug, Clone, Copy)]
pub struct Date {
    pub day: usize,
    pub month: usize,
    pub year: usize,
}

impl Date {
    pub fn next_day(&mut self) -> Vec<TimeEvent> {
        let mut res = vec![TimeEvent::Day];
        self.day += 1;
        if self.day > 30 {
            self.month += 1;
            self.day = 1;
            res.push(TimeEvent::Month);
            if self.month > 12 {
                self.month = 1;
                self.year += 1;
                res.push(TimeEvent::Year);
            }
        }
        res
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

struct DateTimer(Timer);

fn time_system(
    mut date: ResMut<Date>,
    mut date_timer: ResMut<DateTimer>,
    time: Res<Time>,
    mut date_events: EventWriter<TimeEvent>,
    mut date_texts: Query<(&DateDisplay, &mut Text)>,
) {
    if date_timer.0.tick(time.delta()).just_finished() {
        let evts = date.next_day();
        date_events.send_batch(evts.into_iter());
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
                day: 1,
                month: 1,
                year: 1,
            })
            .insert_resource(DateTimer(Timer::from_seconds(2.0, true)))
            .add_event::<TimeEvent>()
            .add_system(time_system.system());
    }
}
