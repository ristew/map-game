use std::collections::HashMap;
use crate::prelude::*;
use crate::{map::MapCoordinate, pops::{PopRef}};
use bevy::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

pub enum ModifierName {

}

#[derive(Clone, Debug)]
pub struct Modifier<T> where T: GameRefQuery + Sized + Send + Sync {
    amount: f32,
    apply_to: T,
}

pub struct Modifiers<T> where T: GameRef {
    inner: HashMap<ModifierName, Modifier<T>>,
    cache: HashMap<ModifierName, f32>,
}

impl<T> Modifiers<T> where T: GameRef {
    pub fn get(&self, modifier: ModifierName) -> f32 {
        self.cache.get(&modifier).unwrap_or(0.0)
    }

    pub fn set(&mut self, modifier: ModifierName, amount: f32) -> f32 {
        self
            .inner
            .entry(&modifier)
            .or_default()
            .insert(&modifier, amount);

        let mut new_value = 0.0;
        for val in self.inner.get(&modifier).unwrap().values() {
            new_value += val.amount;
        }
    }
}
pub struct ModifierPlugin;

impl Plugin for ModifierPlugin {
    fn build(&self, app: &mut AppBuilder) {
    }
}
