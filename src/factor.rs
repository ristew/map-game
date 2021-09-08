use bevy::{core::FixedTimestep, ecs::{component::Component, system::Command, world::EntityRef, system::SystemParam}, prelude::*};
use rand::{Rng, distributions::Slice, prelude::SliceRandom, random, thread_rng};
use rand_distr::Uniform;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use crate::{constant::{DAY_LABEL, DAY_TIMESTEP}, map::*, province::{Province, ProvinceMap, ProvinceRef}, pops::{GameRef}};

pub enum FactorEffectLabel {

}

pub enum FactorEffect {
    Bonus(f32),
    BaseFactor(f32),
    TotalFactor(f32),
}

pub trait FactorType {
    fn base_decay(&self) -> FactorDecay;
}

pub enum FactorDecay {
    Linear(f32),
    Exponential(f32),
    None,
}

pub struct Factor<T> where T: FactorType {
    ftype: T,
    amount: f32,
    target: f32,
    effects: HashMap<&'static str, FactorEffect>,
    decay: FactorDecay,
}

impl<T> Factor<T> where T: FactorType + Eq + Hash + Copy {
    pub fn decay(&mut self) -> f32 {
        let this_decay = match self.decay {
            FactorDecay::Linear(n) => n,
            FactorDecay::Exponential(n) => (self.amount - self.target) * n,
            FactorDecay::None => 0.0,
        };
        self.amount = (self.amount - this_decay).max(self.target);
        this_decay
    }

    pub fn add(&mut self, amt: f32) {
        self.amount += amt;
    }

    pub fn base(ftype: T) -> Self {
        Self {
            ftype,
            amount: 0.0,
            target: 0.0,
            decay: ftype.base_decay(),
            effects: HashMap::new(),
        }
    }
}

pub struct Factors<T> where T: FactorType + Eq + Hash {
    pub inner: HashMap<T, Factor<T>>,
}

impl<T> Factors<T> where T: FactorType + Eq + Hash {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new()
        }
    }
}


impl<T> Factors<T> where T: FactorType + Eq + Hash + Copy {
    pub fn decay(&mut self) {
        for factor in self.inner.values_mut() {
            factor.decay();
        }
    }

    pub fn add(&mut self, ftype: T, amt: f32) {
        if !self.inner.contains_key(&ftype) {
            self.inner.insert(ftype, Factor {
                ftype,
                amount: 0.0,
                target: 0.0,
                decay: ftype.base_decay(),
                effects: HashMap::new(),
            });
        }

        self.inner.get_mut(&ftype).unwrap().amount += amt;
    }

    pub fn factor(&self, ftype: T) -> f32 {
        self.inner.get(&ftype).map(|f| f.amount).unwrap_or(0.0)
    }
}

pub trait EntityManager<R> where R: GameRef {
    fn get_component<T>(&self, ent: R) -> &T where T: Component;
    fn get_factor(&self, entity: R, factor: R::Factor) -> f32;
}

pub trait EntityManagerType {
    type Ref: GameRef;
}
