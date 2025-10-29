use core::panic;

// This module takes in the parsed haskell AST and outputs an Essence specification as raw text.
use crate::adt::{Adt, Func, Operand, Operation, Type};

pub fn essence(adt: Adt, funs: Vec<Func>, verbose: bool) -> String {
    let mut essence_spec = String::new();

    essence_spec.push_str("language Essence 1.3\n\n");

    essence_spec.push_str(&adtext(adt.clone()));

    essence_spec.push_str("\n\n");
    essence_spec.push_str("such that\n\n");

    essence_spec.push_str(&funtext(adt.clone(), funs, verbose));

    essence_spec
}

fn adtext(adt: Adt) -> String {
    let mut str = String::new();
    let len = adt.constructors.len();

    for con in &adt.constructors {
        for (i, t) in con.types.iter().enumerate() {
            str.push_str(&format!("find {}_{} : ", con.prefix, i));
            match t {
                Type::Int => {
                    // For simplicity, we assume all integers are in the range 1 to 5
                    str.push_str("int(1..5)\n");
                }
                Type::Bool => {
                    str.push_str("bool\n");
                }
            }
        }
    }

    str.push_str(&format!("find tag : int(1..{})", len));

    str
}
fn funtext(adt: Adt, funs: Vec<Func>, verbose: bool) -> String {
    let mut str = String::new();

    let prefixes = adt
        .constructors
        .iter()
        .map(|c| c.prefix.clone())
        .collect::<Vec<String>>();
    for f in &funs {
        // check the function's input constructor is in the Adt constructors
        if !prefixes.contains(&f.con.prefix) {
            panic!(
                "Function input constructor {} not in Adt constructors {:?}",
                f.con.prefix, prefixes
            );
        }
        // get the list of constructors that are not the function's input constructor
        let not_prefixes = prefixes
            .iter()
            .filter(|p| *p != &f.con.prefix)
            .collect::<Vec<&String>>();

        // Find the constructor for the function
        let func_con = adt
            .constructors
            .iter()
            .find(|c| c.prefix == f.con.prefix)
            .unwrap();
        let str_op = match &f.opp {
            Operation::ConstSelf => "=",
            Operation::Gt(_, _) => ">",
            Operation::Lt(_, _) => "<",
            Operation::Eq(_, _) => "=",
            Operation::Neq(_, _) => "!=",
            Operation::Leq(_, _) => "<=",
            Operation::Geq(_, _) => ">=",
            Operation::Add(_, _) => "+",
        };
        let (l, r) = match &f.opp {
            Operation::Gt(l, r) => (l, r),
            Operation::Lt(l, r) => (l, r),
            Operation::Eq(l, r) => (l, r),
            Operation::Neq(l, r) => (l, r),
            Operation::Leq(l, r) => (l, r),
            Operation::Geq(l, r) => (l, r),
            Operation::Add(l, r) => (l, r),
            //dummy values
            Operation::ConstSelf => (&Operand::Lit(0), &Operand::Lit(0)),
        };
        if verbose {
            println!("{:?}", func_con);
            println!("{:?}", f);
        }
        // if the operation is ConstSelf then we only have to set the variable = to true
        if let Operation::ConstSelf = &f.opp {
            str.push_str(&format!(
                "({}_{} = true /\\ tag = {})",
                f.con.prefix,
                // very bad, fix later
                0,
                prefixes.iter().position(|p| p == &f.con.prefix).unwrap() + 1
            ));
            // if not the last function, add a newline
            if f != funs.last().unwrap() {
                str.push_str("\n \\/");
            }
            continue;
        }

        str.push('(');
        // handle left operand
        str.push_str(&operand_str(l.clone(), f));

        // add the operator
        str.push_str(&format!(" {} ", str_op));
        // handle right operand
        str.push_str(&operand_str(r.clone(), f));
        str.push(')');

        // get the index of the function input that matches the left string
        // get the index of the function input that matches the right string
        //let right_index = f.con.input.iter().position(|t| t == r).unwrap();
        // println!("Left index: {}, Right index: {}", left_index, right_index);
        //  str.push_str(&format!(
        //     "({}_{} <= {}_{})",
        //   func_con.prefix, left_index, func_con.prefix, right_index
        //));

        str.push_str(&format!(
            " /\\ tag = {}",
            prefixes.iter().position(|p| p == &f.con.prefix).unwrap() + 1
        ));
        for np in &not_prefixes {
            let con = adt.constructors.iter().find(|c| &c.prefix == *np).unwrap();
            for (j, t) in con.types.iter().enumerate() {
                match t {
                    Type::Int => {
                        str.push_str(&format!(" /\\ {}_{} = 1", con.prefix, j));
                    }
                    Type::Bool => {
                        str.push_str(&format!(" /\\ {}_{} = false", con.prefix, j));
                    }
                }
            }
            str.push(')');
        }

        // if not the last function, add a newline
        if f != funs.last().unwrap() {
            str.push_str("\n \\/");
        }
    }

    str
}

fn infix_str(infix : Box<Operation>, func_input: &Func) -> String {
    match *infix {
        Operation::Gt(l, r) => format!("({} > {})", operand_str(l, func_input), operand_str(r, func_input)),
        Operation::Lt(l, r) => format!("({} < {})", operand_str(l, func_input), operand_str(r, func_input)),
        Operation::Eq(l, r) => format!("({} = {})", operand_str(l, func_input), operand_str(r, func_input)),
        Operation::Neq(l, r) => format!("({} != {})", operand_str(l, func_input), operand_str(r, func_input)),
        Operation::Leq(l, r) => format!("({} <= {})", operand_str(l, func_input), operand_str(r, func_input)),
        Operation::Geq(l, r) => format!("({} >= {})", operand_str(l, func_input), operand_str(r, func_input)),
        Operation::Add(l, r) => format!("({} + {})", operand_str(l, func_input), operand_str(r, func_input)),
        _ => panic!("Unsupported infix operation {:?}", infix),
    }
}

fn operand_str(op: Operand, func_input: &Func) -> String {
    match op {
        Operand::Lit(i) => format!("{}", i),
        Operand::Var(s) => {
            // get the index of the function input that matches the string
            let index = func_input.con.input.iter().position(|t| t == &s).unwrap();
            format!("{}_{}", func_input.con.prefix, index)
        }
        Operand::Infix(i) => infix_str(Box::new(*i), func_input),
    }
}