use std::{hash::Hash, fmt::Debug};
use bevy::{ecs::component::Component, prelude::*};
use crate::prelude::*;

pub struct GameRefAccessor<'a, T> where T: GameRef {
    gref: T,
    world: &'a World,
}

impl<'a, T> GameRefAccessor<'a, T> where T: GameRef {
    pub fn new(gref: T, world: &'a World) -> Self {
        Self {
            gref,
            world,
        }
    }

    pub fn get<C>(&'a self) -> &'a C where C: Component {
        self.gref.get::<C>(self.world)
    }

    pub fn get_ref<C>(&'a self) -> GameRefAccessor<'a, C> where C: GameRef + Component {
        self.gref.get::<C>(self.world).accessor(self.world)
    }
}

pub trait GameRef: Copy + Clone + Debug {
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

    fn accessor<'a>(&self, world: &'a World) -> GameRefAccessor<'a, Self> {
        GameRefAccessor::new(*self, world)
    }
}
