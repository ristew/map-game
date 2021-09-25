use bevy::{core::FixedTimestep, ecs::{component::Component, system::Command, world::EntityRef, system::SystemParam}, prelude::*};
use rand::{Rng, distributions::Slice, prelude::SliceRandom, random, thread_rng};
use rand_distr::Uniform;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::fmt::Debug;
use crate::{formula::FactorSubject, pops::GoodType, prelude::*};

pub enum FactorEffectLabel {

}

pub enum FactorEffect {
    Bonus(f32),
    BaseFactor(f32),
    TotalFactor(f32),
}

#[macro_export]
macro_rules! const_factor {
    ( $fname:ident ) => {
        pub const $fname: FactorType = FactorType("$fname");
    }
}


#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum FactorType {
    SettlementCarryingCapacity,
    PopDemand(GoodType),
    PopPressure,
}

pub enum FactorDecay {
    Linear(f32),
    Exponential(f32),
    None,
}



#[derive(Copy, Clone, Eq, Hash, Debug)]
pub enum FactorRef {
    Pop(PopRef),
    Language(LanguageRef),
    Polity(PolityRef),
    Province(ProvinceRef),
    Culture(CultureRef),
    Settlement(SettlementRef),
}

impl FactorSubject for FactorRef {

}

impl From<PopRef> for FactorRef {
    fn from(target: PopRef) -> Self {
        Self::Pop(target)
    }
}

pub struct Factor {
    ftype: FactorType,
    amount: f32,
    target: f32,
    effects: HashMap<&'static str, FactorEffect>,
    decay: FactorDecay,
}

impl Factor {
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

    pub fn base(ftype: FactorType) -> Self {
        Self {
            ftype,
            amount: 0.0,
            target: 0.0,
            decay: ftype.base_decay(),
            effects: HashMap::new(),
        }
    }
}

pub struct Factors {
    pub inner: HashMap<FactorType, Factor>,
}


impl Factors {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new()
        }
    }

    pub fn decay(&mut self) {
        for factor in self.inner.values_mut() {
            factor.decay();
        }
    }

    pub fn add(&mut self, ftype: FactorType, amt: f32) -> f32 {
        if !self.inner.contains_key(&ftype) {
            self.inner.insert(ftype, Factor {
                ftype,
                amount: 0.0,
                target: 0.0,
                decay: ftype.base_decay(),
                effects: HashMap::new(),
            });
        }

        let f = self.inner.get_mut(&ftype).unwrap();
        f.amount += amt;
        f
    }

    pub fn factor(&self, ftype: FactorType) -> f32 {
        self.inner.get(&ftype).map(|f| f.amount).unwrap_or(0.0)
    }

    pub fn clear(&mut self, ftype: FactorType) -> f32 {
        self.inner.remove(&ftype).map(|f| f.amount).unwrap_or(0.0)
    }
}

pub trait EntityManager<R> where R: GameRef {
    fn get_component<T>(&self, ent: R) -> &T where T: Component;
    fn get_factor(&self, entity: R, factor: FactorType) -> f32;
}

pub trait EntityManagerType {
    type Ref: GameRef;
}

pub trait Factored {
    fn factor(&self, world: &World, factor: FactorType) -> f32;
    fn add_factor(&self, world: &mut World, factor: FactorType, amt: f32) -> f32;
}

impl Factored for PopRef {
    fn factor(&self, world: &World, factor: FactorType) -> f32 {
        world.get::<Factors>(self.0).map(|f| f.factor(factor)).unwrap_or(factor.default_amount())
    }

    fn add_factor(&self, world: &mut World, factor: FactorType, amt: f32) -> f32 {
        world.get_mut::<Factors>(self.0).unwrap().add(factor, amt);
        self.factor(world, factor)
    }
}

fn decay_factors_system<T> (
    mut factors: Query<&mut Factors>,
    date: Res<CurrentDate>,
) where T: GameRef {
    if date.is_month {
        for mut fs in factors.iter_mut() {
            fs.decay();
        }
    }
}

pub struct AddFactorCommand<T> where T: Factored + Sized {
    target: T,
    factor: FactorType,
    amt: f32,
}

impl<T> Command for AddFactorCommand<T> where T: Factored + Sized + Debug + Send + Sync + 'static {
    fn write(self: Box<Self>, world: &mut World) {
        println!("set factor {:?} {:?} {}", self.target, self.factor, self.amt);
        self.target.add_factor(world, self.factor, self.amt);
    }
}

pub struct FactorPlugin;

impl Plugin for FactorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            ;
    }
}
