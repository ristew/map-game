use std::{hash::Hash};
use bevy::{ecs::component::Component, prelude::*};
use crate::prelude::*;

pub trait GameRef {
    type Factor: FactorType + Copy + Eq + Hash + Send + Sync + 'static;

    fn entity(&self) -> Entity;

    fn get<'a, T>(&self, world: &'a World) -> &'a T where T: Component {
        self.try_get(world).unwrap()
    }

    fn try_get<'a, T>(&self, world: &'a World) -> Option<&'a T> where T: Component {
        world.get::<T>(self.entity())
    }

    fn get_mut<'a, T>(&self, world: &'a mut World) -> Mut<'a, T> where T: Component {
        self.try_get_mut(world).unwrap()
    }

    fn try_get_mut<'a, T>(&self, world: &'a mut World) -> Option<Mut<'a, T>> where T: Component {
        world.get_mut::<T>(self.entity())
    }

    fn get_factor(&self, world: &World, factor: Self::Factor) -> f32 {
        world.get::<Factors<Self::Factor>>(self.entity()).unwrap().factor(factor)
    }

    fn clear_factor(&self, world: &mut World, factor: Self::Factor) -> f32 {
        world.get_mut::<Factors<Self::Factor>>(self.entity()).unwrap().clear(factor)
    }
}
