use std::{collections::HashMap, sync::Arc};
use std::fmt::Debug;
use std::hash::Hash;
use parking_lot::Mutex;

use bevy::core::AsBytes;

use crate::{factor::Factor, gameref::GameRefQuery, prelude::*};

pub trait FactorSubject: Clone + Eq + Hash + Debug {

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

pub struct FactorDescriptor {

}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FormulaInput<T> where T: FactorSubject {
    Formula(FormulaId),
    Factor(T, FactorType),
    Constant(F32Hash),
}

impl<T> FormulaInput<T> where T: FactorSubject {
    pub fn value(&self, system: &FormulaSystem<T>) -> f32 {
        match self {
            FormulaInput::Formula(f_id) => system.formula_value(*f_id),
            FormulaInput::Factor(r, ftype) => system.get_factor(r, *ftype),
            FormulaInput::Constant(n) => n.0,
        }
    }
}

pub enum FormulaOp<T> where T: FactorSubject {
    Add,
    Sub,
    Mul,
    Div,
    Input(FormulaInput<T>),
}

pub struct FormulaArgs(Vec<FormulaInput<T>>)

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FormulaId(usize);

/// forth-like formula to be maintained in a graph
/// vec![
///     FormulaOp<T>::Add(FormulaInput::Factor(pop_ref, FactorType::PopPressureBase))
/// ]
pub struct Formula<T> where T: FactorSubject {
    pub inputs: Vec<FormulaInput<T>>,
    pub ops: Vec<FormulaOp<T>>,
    pub inner_fn: Box<dyn Fn()>
}

pub struct FormulaValue {
    pub cached: f32,
    pub dirty: bool,
}


pub struct FormulaSystem<T> where T: FactorSubject {
    factors: HashMap<T, Factors>,
    formulae: Vec<Formula<T>>,
    input_map: HashMap<FormulaInput<T>, Vec<FormulaId>>,
    formula_values: Arc<Mutex<HashMap<FormulaId, FormulaValue>>>,
}

// TODO: don't propogate onto end nodes
impl<T> FormulaSystem<T> where T: FactorSubject {
    pub fn add_factor(&mut self, entity: &T, ftype: FactorType, amount: f32) -> f32 {
        if !self.factors.contains_key(entity) {
            self.factors.insert(entity.clone(), Factors::new());
        }
        self.factors.get_mut(entity).unwrap().add(ftype, amount)
    }

    pub fn get_factor(&self, entity: &T, ftype: FactorType) -> f32 {
        self.factors.get(entity).map(|factors| factors.factor(ftype)).unwrap_or(0.0)
    }

    pub fn get_formulae(&mut self, input: FormulaInput<T>) -> Vec<FormulaId> {
        self.input_map.get(&input).map(|fs| fs.clone()).unwrap_or_default()
    }

    fn propogate_changes(&mut self, input: FormulaInput<T>) {
        if !self.input_map.contains_key(&input) {
            return;
        }
        // a formula or factor changed
        for &formula_id in self.get_formulae(input).iter() {
            // look at all the formulae that factor in to other formulae to which is is an input and recalc
            if self.input_map.get(&FormulaInput::Formula(formula_id)).map(|v| v.len()).unwrap_or(0) > 0 {
                let before = self.formula_value(formula_id);
                self.calc_formula(formula_id);
                let after = self.formula_value(formula_id);
                if before != after {
                    self.propogate_changes(FormulaInput::Formula(formula_id));
                }
            } else {
                self.dirty_formula(formula_id);
            }
        }
    }

    fn formula_value(&self, formula_id: FormulaId) -> f32 {
        let mut formula_values = self.formula_values.lock();
        if let Some(val) = formula_values.get_mut(&formula_id) {
            if val.dirty {
                val.cached = self.calc_formula(formula_id);
                val.dirty = false;
            }
            val.cached
        } else {
            0.0
        }
    }

    fn fetch_inputs(&self, inputs: &Vec<FormulaInput<T>>) -> HashMap<FormulaInput<T>, f32> {
        let mut input_values = HashMap::new();
        for input in inputs.iter() {
            let input_value = match input {
                FormulaInput::Formula(f) => self.formula_value(*f),
                FormulaInput::Factor(t, ftype) => self.get_factor(t, *ftype),
                FormulaInput::Constant(n) => n.0,
            };
            input_values.insert(input.clone(), input_value);

        }
        input_values
    }

    fn dirty_formula(&self, formula_id: FormulaId) {
        let mut formula_values = self.formula_values.lock();
        if let Some(val) = formula_values.get_mut(&formula_id) {
            val.dirty = true;
        }
    }

    fn calc_formula(&self, formula_id: FormulaId) -> f32 {
        0.0
    }

    fn add_formula_input(&mut self, formula: FormulaId, input: FormulaInput<T>) {
        self.input_map.entry(input).or_default().push(formula);
    }

    pub fn add_formula(&mut self, formula: Formula<T>) -> FormulaId {
        let idx = self.formulae.len();
        let formula_id = FormulaId(idx);
        for input in formula.inputs.iter() {
        }
        self.formulae.push(formula);
        {
            let mut formula_values = self.formula_values.lock();
            formula_values.insert(formula_id, FormulaValue {
                cached: self.calc_formula(formula_id),
                dirty: false,
            });
        }
        formula_id
    }
}
