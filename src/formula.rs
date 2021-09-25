use std::collections::HashMap;

use crate::{factor::Factor, gameref::GameRefQuery, prelude::*};

pub trait FactorSubject {

}

pub enum FormulaInput<T> where T: FactorSubject {
    Formula(FormulaId),
    Factor(T, FactorType),
    Constant(f32),
}

pub enum FormulaOp<T> where T: FactorSubject {
    Add(FormulaInput<T>),
    Sub(FormulaInput<T>),
    Mul(FormulaInput<T>),
    Div(FormulaInput<T>),
    Group(Box<Vec<FormulaOp<T>>>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FormulaId(usize);

/// forth-like formula to be maintained in a graph
pub struct Formula<T> where T: FactorSubject {
    pub inputs: Vec<FormulaInput<T>>,
    pub ops: Vec<FormulaOp<T>>,
    pub cached: f32,
}



impl Formula {

}

pub struct FormulaSystem<T> where T: FactorSubject {
    factors: HashMap<T, Factors>,
    formulae: Vec<Formula>,
    input_map: HashMap<FormulaInput<T>, Vec<FormulaId>>,
}

impl<T> FormulaSystem<T> where T: FactorSubject {
    pub fn add_factor(&mut self, sub: T, ftype: FactorType, amount: f32) -> f32 {
        self
            .factors
            .entry(&sub)
            .or_default()
            .add(ftype, amount)
    }

    fn propogate_changes(&mut self, input: FormulaInput<T>) {
        for formula_id in self.input_map.get(&input).unwrap_or_default().iter() {
            let before = self.formula_value(formula_id);
            self.calc_formula(formula_id);
            let after = self.formula_value(formula_id);
        }
    }

    fn formula_value(&self, formula: FormulaId) -> f32 {
        self.formulae.get(formula.0).map(|f| f.cached).unwrap_or(0.0)
    }

    fn calc_formula(&mut self, formula: FormulaId) -> f32 {

    }
}
