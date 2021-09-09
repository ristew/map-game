use bevy::{ecs::system::{Command, CommandQueue}, prelude::*};

use crate::{map::SpawnSettlementCommand, pops::PopLanguage, prelude::*, probability::logistic};


pub trait Agent {
    fn think(&self, world: &mut World) -> Vec<Box<dyn Command>>;
}


impl Agent for PopRef {
    fn think(&self, world: &mut World) -> Vec<Box<dyn Command>> {
        let migration_factor = self.factor(world, PopFactor::MigrationDesire);
        if migration_factor <= 1.0 {
            return Vec::new();
        }
        if !individual_event(logistic(migration_factor)) {
            return Vec::new()
        }
        // bad example
        println!("try migrate {:?} {:?}", self, self.get::<ProvinceRef>(world).get::<MapCoordinate>(world));
        self.get_mut::<Pop>(world).size -= 100;
        println!("move pops");
        vec![
            Box::new(SpawnSettlementCommand {
                province: *self.get::<ProvinceRef>(world),
                language: self.get::<PopLanguage>(world).language,
                culture: *self.get::<CultureRef>(world),
            })
        ]
    }
}

pub struct PopThinkCommand(pub PopRef);

impl Command for PopThinkCommand {
    fn write(self: Box<Self>, world: &mut World) {
        self.0.think(world);
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
