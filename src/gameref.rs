use std::{hash::Hash, fmt::Debug};
use bevy::{ecs::component::Component, prelude::*};
use crate::{prelude::*, settlement::Settlement};

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

pub type PopAccessor<'a> = GameRefAccessor<'a, PopRef>;
pub type SettlementAccessor<'a> = GameRefAccessor<'a, SettlementRef>;

impl<'a> PopAccessor<'a> {
    pub fn size(&self) -> isize {
        self.get::<Pop>().size
    }

    pub fn settlement(&self) -> SettlementAccessor {
        self.get_ref::<SettlementRef>()
    }
}

impl<'a> SettlementAccessor<'a> {
    pub fn name(&self) -> &String {
        &self.get::<Settlement>().name
    }
}

pub trait GameRefQuery<T> where T: GameRef {
    fn get_refs(&self) -> Vec<T>;
}

pub trait GameRef: Copy + Clone + Debug + Send + Sync + Hash + Eq {
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

impl<T> GameRefQuery<T> for T where T: GameRef + Sized {
    fn get_refs(&self) -> Vec<T> {
        vec![*self]
    }
}
