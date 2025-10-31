#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone)]
pub struct Adt {
    pub name: String,
    pub constructors: Vec<Cons>,
}

#[derive(Debug, Clone)]
pub struct Cons {
    pub prefix: String,
    pub types: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pub con: FuncInput,
    pub opp: Operation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    ConstSelf,
    Gt(Operand, Operand),
    Lt(Operand, Operand),
    Eq(Operand, Operand),
    Neq(Operand, Operand),
    Leq(Operand, Operand),
    Geq(Operand, Operand),
    Add(Operand, Operand),
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::ConstSelf => write!(f, "self"),
            Operation::Gt(l, r) => write!(f, "{} > {}", l, r),
            Operation::Lt(l, r) => write!(f, "{} < {}", l, r),
            Operation::Eq(l, r) => write!(f, "{} == {}", l, r),
            Operation::Neq(l, r) => write!(f, "{} != {}", l, r),
            Operation::Leq(l, r) => write!(f, "{} <= {}", l, r),
            Operation::Geq(l, r) => write!(f, "{} >= {}", l, r),
            Operation::Add(l, r) => write!(f, "{} + {}", l, r),
        }
    }
}

impl Operation {
    pub fn left(&self) -> Option<&Operand> {
        match self {
            Operation::Gt(l, _) => Some(l),
            Operation::Lt(l, _) => Some(l),
            Operation::Eq(l, _) => Some(l),
            Operation::Neq(l, _) => Some(l),
            Operation::Leq(l, _) => Some(l),
            Operation::Geq(l, _) => Some(l),
            Operation::Add(l, _) => Some(l),
            Operation::ConstSelf => None,
        }
    }
    pub fn right(&self) -> Option<&Operand> {
        match self {
            Operation::Gt(_, r) => Some(r),
            Operation::Lt(_, r) => Some(r),
            Operation::Eq(_, r) => Some(r),
            Operation::Neq(_, r) => Some(r),
            Operation::Leq(_, r) => Some(r),
            Operation::Geq(_, r) => Some(r),
            Operation::Add(_, r) => Some(r),
            Operation::ConstSelf => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncInput {
    pub prefix: String,
    pub input: Vec<String>,
}
#[derive(Debug, Clone)]
pub enum Type {
    Bool,
    Int,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Lit(i32),
    Var(String),
    Infix(Box<Operation>),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Lit(i) => write!(f, "{}", i),
            Operand::Var(v) => write!(f, "{}", v),
            Operand::Infix(op) => write!(f, "({})", op),
        }
    }
}
