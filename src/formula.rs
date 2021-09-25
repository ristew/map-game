use crate::{factor::Factor, gameref::GameRefQuery, prelude::*};


pub trait Factored: Send + Sync + Copy + Clone + Eq + Hash {
    type Subject;
}

pub enum FormulaInput {
    Formula(Formula),
    Factor(Q, FactorType),
    Constant(f32),
}

pub struct Formula {
    pub inputs: Vec<FormulaInput>,
}

pub struct FormulaSystem {

}
