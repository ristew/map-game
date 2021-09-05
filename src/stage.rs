use bevy::prelude::*;

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
