use core::panic;
use std::fmt::Write as _;
use std::io::Write;

// This module takes in the parsed haskell AST and outputs an Essence specification as raw text.
use crate::adt::{Adt, Func, Operand, Operation, Type};

pub fn generate_essence_output(adt: &Adt, funs: &[Func], verbose: bool) -> String {
    let mut essence_spec = String::new();

    essence_spec.push_str("language Essence 1.3\n\n");

    essence_spec.push_str(&adtext(adt));

    essence_spec.push_str("\n\n");
    essence_spec.push_str("such that\n\n");

    essence_spec.push_str(&funtext(adt, funs, verbose));

    if verbose {
        println!("--- Generated Essence Specification ---");
        println!("{essence_spec}");
    }

    // Write to file
    let path = "output.essence";
    let mut file = std::fs::File::create(path).expect("Could not create output file");
    file.write_all(essence_spec.as_bytes())
        .expect("Could not write to output file");

    path.to_string()
}

fn adtext(adt: &Adt) -> String {
    let mut str = String::new();
    let len = adt.constructors.len();

    for con in &adt.constructors {
        for (i, t) in con.types.iter().enumerate() {
            let _ = write!(str, "find {}_{} : ", con.prefix, i);

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

    let _ = write!(str, "find tag : int(1..{len})");

    str
}
fn funtext(adt: &Adt, funs: &[Func], verbose: bool) -> String {
    let mut str = String::new();

    let prefixes = adt
        .constructors
        .iter()
        .map(|c| c.prefix.clone())
        .collect::<Vec<String>>();
    for f in funs {
        // check the function's input constructor is in the Adt constructors
        assert!(
            prefixes.contains(&f.con.prefix),
            "Function input constructor {} not in Adt constructors {:?}",
            f.con.prefix,
            prefixes
        );
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
            Operation::ConstSelf | Operation::Eq(_, _) => "=",
            Operation::Gt(_, _) => ">",
            Operation::Lt(_, _) => "<",
            Operation::Neq(_, _) => "!=",
            Operation::Leq(_, _) => "<=",
            Operation::Geq(_, _) => ">=",
            Operation::Add(_, _) => "+",
        };
        let (l, r) = match &f.opp {
            Operation::Gt(l, r)
            | Operation::Lt(l, r)
            | Operation::Eq(l, r)
            | Operation::Neq(l, r)
            | Operation::Leq(l, r)
            | Operation::Geq(l, r)
            | Operation::Add(l, r) => (l, r),
            //dummy values
            Operation::ConstSelf => (&Operand::Lit(0), &Operand::Lit(0)),
        };
        if verbose {
            println!("{func_con:?}");
            println!("{f:?}");
        }
        // if the operation is ConstSelf then we only have to set the variable = to true
        if let Operation::ConstSelf = &f.opp {
            let _ = write!(
                str,
                "({}_{} = true /\\ tag = {})",
                f.con.prefix,
                // very bad, fix later
                0,
                prefixes.iter().position(|p| p == &f.con.prefix).unwrap() + 1
            );

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
        let _ = write!(str, " {str_op} ");
        // handle right operand
        str.push_str(&operand_str(r.clone(), f));
        str.push(')');

        let _ = write!(
            str,
            " /\\ tag = {}",
            prefixes.iter().position(|p| p == &f.con.prefix).unwrap() + 1
        );

        for np in &not_prefixes {
            let con = adt.constructors.iter().find(|c| &c.prefix == *np).unwrap();
            for (j, t) in con.types.iter().enumerate() {
                match t {
                    Type::Int => {
                        let _ = write!(str, " /\\ {}_{} = 1", con.prefix, j);
                    }
                    Type::Bool => {
                        let _ = write!(str, " /\\ {}_{} = false", con.prefix, j);
                    }
                }
            }
        }

        // if not the last function, add a newline
        if f != funs.last().unwrap() {
            str.push_str("\n \\/");
        }
    }

    str
}

fn infix_str(infix: Box<Operation>, func_input: &Func) -> String {
    match *infix {
        Operation::Gt(l, r) => format!(
            "({} > {})",
            operand_str(l, func_input),
            operand_str(r, func_input)
        ),
        Operation::Lt(l, r) => format!(
            "({} < {})",
            operand_str(l, func_input),
            operand_str(r, func_input)
        ),
        Operation::Eq(l, r) => format!(
            "({} = {})",
            operand_str(l, func_input),
            operand_str(r, func_input)
        ),
        Operation::Neq(l, r) => format!(
            "({} != {})",
            operand_str(l, func_input),
            operand_str(r, func_input)
        ),
        Operation::Leq(l, r) => format!(
            "({} <= {})",
            operand_str(l, func_input),
            operand_str(r, func_input)
        ),
        Operation::Geq(l, r) => format!(
            "({} >= {})",
            operand_str(l, func_input),
            operand_str(r, func_input)
        ),
        Operation::Add(l, r) => format!(
            "({} + {})",
            operand_str(l, func_input),
            operand_str(r, func_input)
        ),
        _ => panic!("Unsupported infix operation {:?}", infix),
    }
}

fn operand_str(op: Operand, func_input: &Func) -> String {
    match op {
        Operand::Lit(i) => format!("{i}"),
        Operand::Var(s) => {
            // get the index of the function input that matches the string
            let index = func_input.con.input.iter().position(|t| t == &s).unwrap();
            format!("{}_{}", func_input.con.prefix, index)
        }
        Operand::Infix(i) => infix_str(Box::new(*i), func_input),
    }
}
