use std::{collections::HashMap, sync::Arc};
use std::fmt::Debug;
use std::hash::Hash;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};

use bevy::core::AsBytes;

use crate::factor::FactorRef;
use crate::{factor::Factor, gameref::GameRefQuery, prelude::*};

pub trait FactorSubject: Clone + Eq + Hash + Debug + Send + Sync {
}


#[derive(Copy, Clone, Debug)]
pub struct F32Hash(f32);
impl Hash for F32Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_bytes().hash(state);
    }
}

impl PartialEq for F32Hash {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for F32Hash {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Into<f32> for F32Hash {
    fn into(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FormulaId(usize);

pub enum FormulaFn {
    VecArgs(Arc<dyn Fn(Vec<f32>) -> f32 + Send + Sync>),
    OneArgs(Arc<dyn Fn(f32) -> f32 + Send + Sync>),
}

impl<F> From<F> for FormulaFn where F: Fn(f32) -> f32 + Send + Sync + 'static {
    fn from(f: F) -> Self {
        Self::OneArgs(Arc::new(f))
    }
}

impl FormulaFn {
    pub fn new<F, Args>(inner: F) -> Self where F: Fn(f32) -> f32 + Send + Sync + 'static {
        Self::OneArgs(Arc::new(inner))
    }

    pub fn run(&self, args: Vec<f32>) -> f32 {
        match self {
            FormulaFn::VecArgs(f) => (*f)(args),
            FormulaFn::OneArgs(f) => (*f)(args[0]),
        }
    }
}

pub struct Formula<T> where T: FactorSubject {
    pub inputs: Vec<T>,
    pub inner_fn: FormulaFn,
    pub subject: T,
}

impl<T> Formula<T> where T: FactorSubject {
    pub fn new<F>(inputs: Vec<T>, inner_fn: F, subject: T) -> Self where F: Into<FormulaFn> {
        Self {
            inputs,
            inner_fn: inner_fn.into(),
            subject,
        }
    }

    pub fn calc(&self, args: Vec<f32>) -> f32 {
        self.inner_fn.run(args)
    }
}

pub struct FormulaValue {
    pub cached: f32,
    pub dirty: bool,
}


pub struct FormulaSystem<T> where T: FactorSubject {
    factors: DashMap<T, Factor>,
    formulae: Vec<Formula<T>>,
    input_map: HashMap<T, Vec<FormulaId>>,
    formula_values: DashMap<FormulaId, FormulaValue>,
}

// TODO: don't propogate onto end nodes
impl<T> FormulaSystem<T> where T: FactorSubject {
    pub fn add_factor(&mut self, f: &T, amount: f32) {
        self.factors.get_mut(f).map(|factor| {
            match factor.value_mut() {
                Factor::Constant(n) => *n += amount,
                Factor::Decay(n, _) => *n += amount,
                _ => println!("tried to add to formula? {:?} + {}", f, amount),
            }
        });
        self.propogate_changes(f);
    }

    pub fn set_factor(&self, f: &T, amount: f32) {

    }

    pub fn get_factor(&self, f: &T) -> f32 {
        self.factors.get(f).map(|factor| {
            match factor.value() {
                Factor::Constant(n) => *n,
                Factor::Decay(n, _) => *n,
                Factor::Formula(formula_id) => self.formula_value(*formula_id),
            }
        }).unwrap_or(0.0)
    }

    pub fn get_formula(&self, f: &T) -> FormulaId {
        let factor = self.factors.get(f).unwrap();
        match factor.value() {
            Factor::Formula(formula_id) => *formula_id,
            _ => panic!("get formula on not formula {:?}", f),
        }
    }

    // retrieve formulae that change as a result of f changing
    pub fn get_formulae(&self, f: &T) -> Vec<FormulaId> {
        self
            .input_map
            .get(f)
            .map(|fs|
                 fs.iter()
                 .map(|fid| *fid)
                 .collect::<Vec<_>>()
            ).unwrap_or(Vec::new())
    }

    // given that f changed, update values of all descendant formulae
    fn propogate_changes(&mut self, f: &T) {
        for &formula_id in self.get_formulae(f).iter() {
            let formula = self.formulae[formula_id.0];
            // only really calc if there are more down the line, otherwise mark dirty
            if self.input_map.get(&formula.subject).map(|v| v.len()).unwrap_or(0) > 0 {
                let before = self.formula_value(formula_id);
                self.calc_formula(formula_id);
                let after = self.formula_value(formula_id);
                // only recalc if value actually changed (highly likely)
                if before != after {
                    self.propogate_changes(&formula.subject);
                }
            } else {
                self.dirty_formula(formula_id);
            }
        }
    }

    fn formula_value(&self, formula_id: FormulaId) -> f32 {
        {
            if let Some(val) = self.formula_values.get(&formula_id) {
                if !val.dirty {
                    return val.cached
                }
            } else {
                println!("BAD: Formula without value! {:?}", formula_id);
                return 0.0;
            }
        }
        {
            let val = self.formula_values.get_mut(&formula_id).unwrap();
            val.cached = self.calc_formula(formula_id);
            val.dirty = false;
            val.cached
        }
    }

    fn fetch_inputs(&self, inputs: &Vec<T>) -> Vec<f32> {
        let mut res = Vec::new();
        for input in inputs.iter() {
            res.push(self.get_factor(input));
        }
        res
    }

    fn dirty_formula(&self, formula_id: FormulaId) {
        if let Some(val) = self.formula_values.get_mut(&formula_id) {
            val.dirty = true;
        }
    }

    fn calc_formula(&self, formula_id: FormulaId) -> f32 {
        let formula = &self.formulae[formula_id.0];
        let value = formula.calc(self.fetch_inputs(&formula.inputs));
        value
    }

    fn add_input(&self, f: &T, ) {

    }

    pub fn add_formula(&mut self, formula: Formula<T>) -> FormulaId {
        let idx = self.formulae.len();
        let formula_id = FormulaId(idx);
        for input in formula.inputs.iter() {
            self.input_map.get_mut(input).map(|v| v.push(formula_id));
        }
        self.formulae.push(formula);
        self.formula_values.insert(formula_id, FormulaValue {
            cached: self.calc_formula(formula_id),
            dirty: false,
        });
        formula_id
    }
}
