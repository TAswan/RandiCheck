use serde::{Deserialize, Serialize};
use std::io::Write;
// This module takes in the parsed haskell AST and outputs an Essence specification as raw text.
use crate::adt::{Adt, Func, Operation, Type};

#[derive(Debug, Serialize, Deserialize)]
struct TeraFunc {
    input: String,
    nons: Vec<String>,
}

pub fn generate_essence_output(adt: &Adt, funs: &[Func], verbose: bool) -> String {
    let tera = tera::Tera::new("src/templates/*.tera").unwrap();
    let mut context = tera::Context::new();

    // the adt needs flipped because of stack nonsense
    

    context.insert("adt", &adt);
    context.insert(
        "funcs",
        &funs
            .iter()
            .map(|f| funtext(adt, f.clone(), verbose))
            .collect::<Vec<TeraFunc>>(),
    );

    let essence_spec = tera.render("essence.tera", &context).unwrap();

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

fn funtext(adt: &Adt, func: Func, verbose: bool) -> TeraFunc {
    let prefixes = adt
        .constructors
        .iter()
        .map(|c| c.prefix.clone())
        .collect::<Vec<String>>();

    // check the function's input constructor is in the Adt constructors
    assert!(
        prefixes.contains(&func.con.prefix),
        "Function input constructor {} not in Adt constructors {:?}",
        func.con.prefix,
        prefixes
    );

    let str_op = convert_variables(&func.opp, func.clone());

    let not_prefixes = prefixes
        .iter()
        .filter(|p| *p != &func.con.prefix)
        .collect::<Vec<&String>>();
    let mut nons = Vec::new();
    for np in &not_prefixes {
        let con = adt.constructors.iter().find(|c| &c.prefix == *np).unwrap();
        for (j, t) in con.types.iter().enumerate() {
            match t {
                Type::Int => {
                    nons.push(format!(" {}_{} = 1", con.prefix, j + 1));
                }
                Type::Bool => {
                    nons.push(format!(" {}_{} = false", con.prefix, j + 1));
                }
            }
        }
    }

    TeraFunc {
        input: str_op,
        nons,
    }
}

fn convert_variables(op: &Operation, func: Func) -> String {
    match op {
        Operation::Var(name) => {
            let index = func
                .con
                .input
                .iter()
                .position(|n| n == name)
                .expect("Variable name not found in function input");
            format!("{}_{}", func.con.prefix, index + 1)
        }
        Operation::Add(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} + {}", left, right)
        }
        Operation::Gt(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} > {}", left, right)
        }
        Operation::Lt(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} < {}", left, right)
        }
        Operation::Eq(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} = {}", left, right)
        }
        Operation::Neq(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} != {}", left, right)
        }
        Operation::Leq(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} <= {}", left, right)
        }
        Operation::Geq(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} >= {}", left, right)
        }
        Operation::Sub(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} - {}", left, right)
        }
        Operation::Mul(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} * {}", left, right)
        }
        Operation::And(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} && {}", left, right)
        }
        Operation::Or(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{} || {}", left, right)
        }
        Operation::Not(x) => {
            let val = convert_variables(x, func.clone());
            format!("not {}", val)
        }

        x => x.to_string(),
    }
}
