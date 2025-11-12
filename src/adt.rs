#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adt {
    pub name: String,
    pub constructors: Vec<Cons>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cons {
    pub prefix: String,
    pub types: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Func {
    pub con: FuncInput,
    pub opp: Operation,
}
impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.con, self.opp)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    BoolLit(bool),
    IntLit(i32),
    Var(String),
    And(Box<Operation>, Box<Operation>),
    Or(Box<Operation>, Box<Operation>),
    Not(Box<Operation>),
    Gt(Box<Operation>, Box<Operation>),
    Lt(Box<Operation>, Box<Operation>),
    Eq(Box<Operation>, Box<Operation>),
    Neq(Box<Operation>, Box<Operation>),
    Leq(Box<Operation>, Box<Operation>),
    Geq(Box<Operation>, Box<Operation>),
    Add(Box<Operation>, Box<Operation>),
    Sub(Box<Operation>, Box<Operation>),
    Mul(Box<Operation>, Box<Operation>),
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::BoolLit(b) => write!(f, "{}", b),
            Operation::IntLit(i) => write!(f, "{}", i),
            Operation::Var(name) => write!(f, "{}", name),
            Operation::Gt(l, r) => write!(f, "{l} > {r}"),
            Operation::Lt(l, r) => write!(f, "{l} < {r}"),
            Operation::Eq(l, r) => write!(f, "{l} == {r}"),
            Operation::Neq(l, r) => write!(f, "{l} != {r}"),
            Operation::Leq(l, r) => write!(f, "{l} <= {r}"),
            Operation::Geq(l, r) => write!(f, "{l} >= {r}"),
            Operation::Add(l, r) => write!(f, "{l} + {r}"),
            Operation::Sub(l, r) => write!(f, "{l} - {r}"),
            Operation::Mul(l, r) => write!(f, "{l} * {r}"),
            Operation::And(l, r) => write!(f, "({} /\\ {})", l, r),
            Operation::Or(l, r) => write!(f, "({} \\/ {})", l, r),
            Operation::Not(o) => write!(f, "!({})", o),
        }
    }
}
impl Operation {
    pub fn to_haskell(&self) -> String {
        match self {
            Operation::BoolLit(b) => match b {
                true => "True".to_string(),
                false => "False".to_string(),
            },
            Operation::IntLit(i) => format!("{}", i),
            Operation::Var(name) => format!("{}", name),
            Operation::Gt(l, r) => format!("{} > {}", l.to_haskell(), r.to_haskell()),
            Operation::Lt(l, r) => format!("{} < {}", l.to_haskell(), r.to_haskell()),
            Operation::Eq(l, r) => format!("{} == {}", l.to_haskell(), r.to_haskell()),
            Operation::Neq(l, r) => format!("{} /= {}", l.to_haskell(), r.to_haskell()),
            Operation::Leq(l, r) => format!("{} <= {}", l.to_haskell(), r.to_haskell()),
            Operation::Geq(l, r) => format!("{} >= {}", l.to_haskell(), r.to_haskell()),
            Operation::Add(l, r) => format!("{} + {}", l.to_haskell(), r.to_haskell()),
            Operation::Sub(l, r) => format!("{} - {}", l.to_haskell(), r.to_haskell()),
            Operation::Mul(l, r) => format!("{} * {}", l.to_haskell(), r.to_haskell()),
            Operation::And(l, r) => format!("{} && {}", l.to_haskell(), r.to_haskell()),
            Operation::Or(l, r) => format!("{} || {}", l.to_haskell(), r.to_haskell()),
            Operation::Not(o) => format!("not {}", o.to_haskell()),
        }
    }


    pub fn is_infix(&self) -> bool {
        matches!(
            self,
            Operation::Gt(_, _)
                | Operation::Lt(_, _)
                | Operation::Eq(_, _)
                | Operation::Neq(_, _)
                | Operation::Leq(_, _)
                | Operation::Geq(_, _)
                | Operation::Add(_, _)
                | Operation::Sub(_, _)
                | Operation::Mul(_, _)
                | Operation::And(_, _)
                | Operation::Or(_, _)
        )
    }

    pub fn left(&self) -> Option<&Operation> {
        match self {
            Operation::Gt(l, _) => Some(l),
            Operation::Lt(l, _) => Some(l),
            Operation::Eq(l, _) => Some(l),
            Operation::Neq(l, _) => Some(l),
            Operation::Leq(l, _) => Some(l),
            Operation::Geq(l, _) => Some(l),
            Operation::Add(l, _) => Some(l),
            Operation::Sub(l, _) => Some(l),
            Operation::Mul(l, _) => Some(l),
            Operation::And(l, _) => Some(l),
            Operation::Or(l, _) => Some(l),
            _ => None,
        }
    }
    pub fn right(&self) -> Option<&Operation> {
        match self {
            Operation::Gt(_, r) => Some(r),
            Operation::Lt(_, r) => Some(r),
            Operation::Eq(_, r) => Some(r),
            Operation::Neq(_, r) => Some(r),
            Operation::Leq(_, r) => Some(r),
            Operation::Geq(_, r) => Some(r),
            Operation::Add(_, r) => Some(r),
            Operation::Sub(_, r) => Some(r),
            Operation::Mul(_, r) => Some(r),
            Operation::And(_, r) => Some(r),
            Operation::Or(_, r) => Some(r),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FuncInput {
    pub prefix: String,
    pub input: Vec<String>,
}
impl fmt::Display for FuncInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {})", self.prefix, self.input.join(" "))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    Bool,
    Int,
}
