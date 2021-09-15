use bevy::{ecs::{schedule::{ParallelSystemDescriptor, SystemDescriptor}, system::ExclusiveSystem}, prelude::*};

use crate::prelude::DAY_LABEL;

#[derive(StageLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InitStage {
    LoadMap,
    LoadPops,
    LoadProvinces,
}

#[derive(StageLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DayStage {
    Init,
    Main,
}


pub trait DayStageBuilder {
    fn add_system_to_day<T>(&mut self, system: T) -> &mut AppBuilder where T: ParallelSystemDescriptorCoercion ;
}

impl DayStageBuilder for AppBuilder {
    fn add_system_to_day<T>(&mut self, system: T) -> &mut AppBuilder where T: ParallelSystemDescriptorCoercion {
        self.add_system_to_stage(DayStage::Main, system.label(DAY_LABEL))
    }
}
