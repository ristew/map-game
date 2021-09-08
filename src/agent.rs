use bevy::{ecs::system::{Command, CommandQueue}, prelude::*};

use crate::{PopRef, constant::DAY_LABEL, factor::Factored, pops::{Pop, PopFactor}, probability::individual_event, stage::DayStage, time::Date};


pub trait Agent {
    fn think(&self, world: &World);
}


impl Agent for PopRef {
    fn think(&self, world: &World) {
        if self.factor(world, PopFactor::MigrationDesire) > 1.0 {
            println!("try migrate {:?}", self);
        }
    }
}

pub struct PopThinkCommand(pub PopRef);

impl Command for PopThinkCommand {
    fn write(self: Box<Self>, world: &mut World) {
        self.0.think(&world);
    }
}

fn think_system(
    mut commands: Commands,
    date: Res<Date>,
    pop_q: Query<(Entity, &Pop)>,
) {
    if !date.is_month {
        return;
    }

    for (pop_ent, pop) in pop_q.iter() {
        if individual_event(0.1) {
            commands.add(PopThinkCommand(PopRef(pop_ent)));
        }
    }
}

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system_to_stage(DayStage::Main, think_system.system());
    }
}
