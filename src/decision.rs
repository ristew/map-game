use crate::prelude::*;
use crate::agent::ValueAgent;
use crate::settlement::Settlement;
use bevy::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

pub struct CalledToArmsEvent {
    caller: PolityRef,
    callee: PolityRef,
    enemy: PolityRef,
}

// Some of our countrymen went hungry
pub struct PopStarvedEvent {
    pop: PopRef,
    amount: usize,
}

#[derive(EnumIter)]
pub enum PopStarvedChoice {
    SendFullRelief,
    SendSomeHelp,
    Ignore,
}

impl GameEvent for PopStarvedEvent {
    type Choice = PopStarvedChoice;

    fn description(&self, world: &World) -> String {
        let pop = self.pop.accessor(world);
        format!(
            "{} people in {} starved!",
            self.amount,
            pop.get_ref::<SettlementRef>().get::<Settlement>().name,
        )
    }

    fn choices(&self) -> Vec<PopStarvedChoice> {
        PopStarvedChoice::iter().collect::<Vec<_>>()
    }

    fn weigh_choice(&self, agent: &ValueAgent, choice: PopStarvedChoice) -> f32 {
        match choice {
            PopStarvedChoice::SendFullRelief => todo!(),
            PopStarvedChoice::SendSomeHelp => todo!(),
            PopStarvedChoice::Ignore => todo!(),
        }
    }
}

pub struct GameEventMeta {
    // added to final uncertainty to pass AI threshold
    importance: f32,
}

pub trait GameEvent {
    type Choice;
    fn description(&self, world: &World) -> String;
    fn choices(&self) -> Vec<Self::Choice>;
    fn weigh_choice(&self, agent: &ValueAgent, choice: Self::Choice) -> f32;
}
