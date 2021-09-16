use bevy::{ecs::system::{Command, CommandQueue}, prelude::*};

use crate::{map::SpawnSettlementCommand, pops::{PopLanguage, PopSeekMigrationCommand}, prelude::*, probability::logistic};

pub struct ValueAgent {

}

pub trait Agent {
    fn think(&self, world: &mut World) -> Vec<Box<dyn Command>>;
}


impl Agent for PopRef {
    fn think(&self, world: &mut World) -> Vec<Box<dyn Command>> {
        let migration_factor = self.factor(world, PopFactor::MigrationDesire);
        if migration_factor > 1.0 && individual_event(logistic(migration_factor)) {
            // bad example
            // println!("try migrate {:?} {:?}", self, self.get::<ProvinceRef>(world).get::<MapCoordinate>(world));
            // println!("move pops");
            vec![
                Box::new(PopSeekMigrationCommand {
                    pop: *self,
                    pressure: migration_factor,
                })
            ]
        }
        else {
            Vec::new()
        }
    }
}

pub struct PopThinkCommand(pub PopRef);

impl Command for PopThinkCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let cmds = self.0.think(world);
        for command in cmds.into_iter() {
            command.write(world);
        }
    }
}

fn think_system(
    mut commands: Commands,
    date: Res<CurrentDate>,
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
            .add_system_to_stage(DayStage::Main, think_system.system().label(DAY_LABEL));
    }
}
