use std::collections::HashMap;
use crate::{map::MapCoordinate, pops::{PopRef}};
use bevy::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, EnumIter)]
pub enum ModifierType {
    HarvestSize,
    MigrationPush,
    MigrationPull,
}

#[derive(Clone, Debug)]
pub enum ModifierEffect {
    Additional(f64),
    Factor(f64),
}

#[derive(Clone, Debug)]
pub struct Modifier {
    mod_effect: ModifierEffect,
}

#[derive(Default)]
pub struct ModifierList(Vec<Modifier>);

impl ModifierList {
    pub fn apply(&self, mut base: f64) -> f64 {
        let mut addtl = 0.0;
        for modifier in self.0.iter() {
            match modifier.mod_effect {
                ModifierEffect::Additional(a) => addtl += a,
                ModifierEffect::Factor(f) => base *= f,
            }
        }
        base + addtl
    }
}

#[derive(Default)]
pub struct Modifiers {
    global: ModifierList,
    // country: HashMap<CountryRef, ModifierList>,
    province: HashMap<MapCoordinate, ModifierList>,
    culture: HashMap<CultureRef, ModifierList>,
    religion: HashMap<ReligionRef, ModifierList>,
    pop: HashMap<Entity, ModifierList>,
}


impl Modifiers {
    pub fn modifiers_pop(&self, pop_entity: Entity, base_pop: &BasePop, coord: &MapCoordinate) -> ModifierList {
        let mut modifiers = Vec::new();
        macro_rules! add_modifiers {
            ( $itr:expr ) => {
                for modifier in $itr.iter() {
                    modifiers.push(modifier.clone());
                }
            }
        }
        add_modifiers!(self.global.0);
        add_modifiers!(self.province.get(&coord).map(|ml| &ml.0).unwrap_or(&Vec::new()));
        add_modifiers!(self.culture.get(&base_pop.culture).map(|ml| &ml.0).unwrap_or(&Vec::new()));
        add_modifiers!(self.religion.get(&base_pop.religion).map(|ml| &ml.0).unwrap_or(&Vec::new()));
        add_modifiers!(self.pop.get(&pop_entity).map(|ml| &ml.0).unwrap_or(&Vec::new()));
        ModifierList(modifiers)
    }
}

#[derive(Default)]
pub struct ModifierStorage(pub HashMap<ModifierType, Modifiers>);

impl ModifierStorage {
    pub fn modifiers_pop(&self, modifier_type: ModifierType, pop_entity: Entity, base_pop: &BasePop, coord: &MapCoordinate) -> ModifierList {
        if let Some(modifiers) = self.0.get(&modifier_type) {
            modifiers.modifiers_pop(pop_entity, base_pop, coord)
        } else {
            ModifierList(Vec::new())
        }
    }
}

pub fn setup_modifiers(
    mut modifier_storage: ResMut<ModifierStorage>,
) {
    for modifier_type in ModifierType::iter() {
        let modifiers = Modifiers::default();
        modifier_storage.0.insert(modifier_type, modifiers);
    }
}


pub fn get_modifiers(
) {

}

pub struct ModifierPlugin;

impl Plugin for ModifierPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .init_resource::<ModifierStorage>()
            .add_startup_system(setup_modifiers.system());
    }
}
