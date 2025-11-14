use serde::{Deserialize, Serialize};
use std::io::Write;
// This module takes in the parsed haskell AST and outputs an Essence specification as raw text.
use crate::adt::{Adt, Func, Operation, Type};

#[derive(Debug, Serialize, Deserialize)]
struct TeraFunc {
    input: String,
    nons: Vec<String>,
}

pub fn generate_essence_output(
    adt: &Adt,
    funs: &[Func],
    verbose: bool,
    min: i32,
    max: i32,
) -> String {
    let tera = tera::Tera::new("src/templates/*.tera").unwrap();
    let mut context = tera::Context::new();

    // the adt needs flipped because of stack nonsense

    context.insert("adt", &adt);
    context.insert(
        "funcs",
        &funs
            .iter()
            .map(|f| funtext(adt, f.clone(), verbose, min))
            .collect::<Vec<TeraFunc>>(),
    );
    context.insert("min", &min);
    context.insert("max", &max);

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

fn funtext(adt: &Adt, func: Func, _verbose: bool, min: i32) -> TeraFunc {
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
                    nons.push(format!(" {}_{} = {min}", con.prefix, j + 1));
                }
                Type::Bool => {
                    nons.push(format!(" {}_{} = false", con.prefix, j + 1));
                }
                Type::Custom(_) => todo!(),
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
            format!("{left} + {right}")
        }
        Operation::Gt(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} > {right}")
        }
        Operation::Lt(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} < {right}")
        }
        Operation::Eq(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} = {right}")
        }
        Operation::Neq(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} != {right}")
        }
        Operation::Leq(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} <= {right}")
        }
        Operation::Geq(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} >= {right}")
        }
        Operation::Sub(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} - {right}")
        }
        Operation::Mul(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} * {right}")
        }
        Operation::And(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} && {right}")
        }
        Operation::Or(x, y) => {
            let left = convert_variables(x, func.clone());
            let right = convert_variables(y, func.clone());
            format!("{left} || {right}")
        }
        Operation::Not(x) => {
            let val = convert_variables(x, func.clone());
            format!("not {val}")
        }
        Operation::Apply(f, arg) => apply(f, arg, func),

        x => x.to_string(),
    }
}

fn apply(f: &Operation, arg: &Operation, func: Func) -> String {
    // get all functions in local_binds with prefix matching f
    let funcs = func
        .local_binds
        .iter()
        .filter(|fb| match f {
            Operation::Var(name) => fb.con.prefix == *name,
            _ => false,
        })
        .collect::<Vec<&Func>>();

    if funcs.is_empty() {
        panic!(
            "Function {} not found in local binds",
            match f {
                Operation::Var(name) => name.clone(),
                _ => "unknown".to_string(),
            }
        );
    }

    let mut apply_str = String::new();

    for (i, fb) in funcs.iter().enumerate() {
        let input = fb.con.input.first().expect("Function has no input");

        let arg_str = convert_variables(arg, func.clone());

        let replaced_op = replace_variable(&fb.opp, input, &arg_str, func.clone());

        apply_str.push_str(&format!("({})", replaced_op));

        if i < funcs.len() - 1 {
            apply_str.push_str(" \\/ ");
        }
    }

    apply_str
}

fn replace_variable(op: &Operation, var_name: &String, replacement: &String, func: Func) -> String {
    match op {
        Operation::Var(name) => {
            if name == var_name {
                replacement.clone()
            } else {
                let index = func
                    .con
                    .input
                    .iter()
                    .position(|n| n == name)
                    .expect("Variable name not found in function input");
                format!("{}_{}", func.con.prefix, index + 1)
            }
        }
        Operation::Add(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} + {right}")
        }
        Operation::Gt(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} > {right}")
        }
        Operation::Lt(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} < {right}")
        }
        Operation::Eq(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} = {right}")
        }
        Operation::Neq(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} != {right}")
        }
        Operation::Leq(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} <= {right}")
        }
        Operation::Geq(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} >= {right}")
        }
        Operation::Sub(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} - {right}")
        }
        Operation::Mul(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} * {right}")
        }
        Operation::And(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} && {right}")
        }
        Operation::Or(x, y) => {
            let left = replace_variable(x, var_name, replacement, func.clone());
            let right = replace_variable(y, var_name, replacement, func.clone());
            format!("{left} || {right}")
        }
        Operation::Not(x) => {
            let val = replace_variable(x, var_name, replacement, func.clone());
            format!("not {val}")
        }
        Operation::Apply(f, arg) => {
            let func_str = replace_variable(f, var_name, replacement, func.clone());
            let arg_str = replace_variable(arg, var_name, replacement, func.clone());
            format!("{}({})", func_str, arg_str)
        }
        x => x.to_string(),
    }
}
