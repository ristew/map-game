use bevy::{core::FixedTimestep, ecs::{component::Component, system::Command, world::EntityRef, system::SystemParam}, prelude::*};
use rand::{Rng, distributions::Slice, prelude::SliceRandom, random, thread_rng};
use rand_distr::Uniform;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::fmt::Debug;
use crate::{formula::{FactorSubject, FormulaId, FormulaSystem}, pops::GoodType, prelude::*};

pub enum FactorEffectLabel {

}

pub enum FactorEffect {
    Bonus(f32),
    BaseFactor(f32),
    TotalFactor(f32),
}


//TODO: split out into PopFactor eg like FactorRef
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FactorType {
    SettlementPopulation,
    SettlementCarryingCapacity,
    SettlementPressure,

    PopDemand(GoodType),
    PopPressure,
}

pub enum FactorDecay {
    Linear(f32),
    Exponential(f32),
    None,
}



#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FactorRef {
    Pop(PopRef),
    Language(LanguageRef),
    Polity(PolityRef),
    Province(ProvinceRef),
    Culture(CultureRef),
    Settlement(SettlementRef),
}

pub type FST = (FactorRef, FactorType);

impl FactorSubject for FST {

}

impl From<PopRef> for FactorRef {
    fn from(target: PopRef) -> Self {
        Self::Pop(target)
    }
}

pub enum Factor {
    Constant(f32),
    Decay(f32, FactorDecay),
    Formula(FormulaId),
}

impl Factor {
    pub fn decay(&mut self) -> f32 {
        match self {
            &mut Factor::Decay(amount, decay) => {
                let this_decay = match decay {
                    FactorDecay::Linear(n) => n,
                    FactorDecay::Exponential(n) => amount * n,
                    FactorDecay::None => 0.0,
                };
                amount = (amount - this_decay).max(0.0);
                this_decay
            },
            _ => 0.0,
        }
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

pub struct AddFactorCommand {
    target: FST,
    amt: f32,
}

impl Command for AddFactorCommand {
    fn write(self: Box<Self>, world: &mut World) {
        println!("set factor {:?} {}", self.target, self.amt);
        world.get_resource::<FormulaSystem<FST>>().map(|factor_system| factor_system.add_factor(&self.target, self.amt));
    }
}

pub struct FactorPlugin;

impl Plugin for FactorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            ;
    }
}
