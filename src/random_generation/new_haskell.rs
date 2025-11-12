// This module contains functions for generating random haskell code based on our ADTs and operations using tera
use crate::adt::Adt;
use crate::adt::Cons;
use crate::adt::Func;
use crate::adt::FuncInput;
use crate::adt::Operation;

use rand::distr::SampleString;
use rand::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use tera::Context;
use tera::Tera;

#[derive(Debug, Serialize, Deserialize)]
struct TeraFunc {
    con: FuncInput,
    opp: String,
}

pub fn generate_haskell_random(max_depth: u32, verbose: bool) {
    let mut rng = rand::rng();

    let adt = generate_adt(&mut rng, verbose, max_depth);

    let func_depth = rng.random_range(1..=max_depth);
    let funcs = generate_func(&mut rng, &adt, func_depth, verbose);

    if verbose {
        println!("Generated Functions: {funcs:?}");
    }

    let tera = Tera::new("src/templates/*.tera").unwrap();
    let mut context = Context::new();

    context.insert("adt", &adt);

    let tera_funcs: Vec<TeraFunc> = funcs
        .iter()
        .map(|f| TeraFunc {
            con: f.con.clone(),
            opp: f.opp.to_haskell(),
        })
        .collect();

    context.insert("funcs", &tera_funcs);

    let code = tera.render("haskell_in.tera", &context).unwrap();

    if verbose {
        println!("{code}");
    }
}

fn generate_adt(rng: &mut impl Rng, verbose: bool, max_depth: u32) -> Adt {
    // generate a random ADT

    let random_name = format!("RandomADT{}", rng.random_range(1..1000));

    let num_constructors = rng.random_range(1..max_depth);
    let mut constructors = Vec::new();

    for i in 0..num_constructors {
        let prefix = format!("Constructor{}", i + 1);
        let num_types = rng.random_range(1..max_depth);
        let mut types = Vec::new();
        for _ in 0..num_types {
            types.push(match rng.random_range(0..2) {
                0 => crate::adt::Type::Int,
                1 => crate::adt::Type::Bool,
                _ => panic!("Unexpected random value"),
            });
        }
        constructors.push(crate::adt::Cons { prefix, types });
    }
    let adt = Adt {
        name: random_name,
        constructors,
    };

    if verbose {
        println!("Generated ADT: {adt:?}");
    }

    adt
}

fn generate_func(rng: &mut impl Rng, adt: &Adt, max_depth: u32, verbose: bool) -> Vec<Func> {
    let mut result = Vec::new();

    /*  let num_inputs = match rng.random_range(1..6) != 3 {
        true => adt.constructors.len() as u32,
        false => rng.random_range(1..=adt.constructors.len() as u32),
    };*/

    let num_inputs = adt.constructors.len();
    for i in 0..num_inputs {
        let constructor = adt.constructors[i].clone();

        let mut input_values = Vec::new();

        for (i, _) in constructor.types.iter().enumerate() {
            // push a random letter as variable (changing size based on index to avoid duplicates)
            input_values.push(
                rand::distr::Alphabetic
                    .sample_string(rng, i + 1)
                    .to_lowercase(),
            );
        }

        let con = crate::adt::FuncInput {
            prefix: constructor.prefix.clone(),
            input: input_values,
        };
        let opp = generate_operation(
            rng,
            verbose,
            &constructor,
            max_depth,
            con.clone(),
            crate::adt::Type::Bool,
        );

        let func = crate::adt::Func { con, opp };
        result.push(func);
    }
    result
}

fn generate_operation(
    rng: &mut impl Rng,
    verbose: bool,
    constructor: &Cons,
    max_depth: u32,
    input: FuncInput,
    return_type: crate::adt::Type,
) -> Operation {
    if verbose {
        println!("Generating operation with return type {return_type:?} at depth {max_depth}");
    }
    if max_depth == 0 {
        //find all variables in the input with the return type
        let mut candidates = Vec::new();
        for (i, t) in constructor.types.iter().enumerate() {
            if *t == return_type {
                candidates.push(input.input[i].clone());
            }
        }

        if !candidates.is_empty() && rng.random_bool(0.75) {
            let selected = candidates.choose(rng).unwrap();

            return Operation::Var(selected.to_string());
        }

        return match return_type {
            crate::adt::Type::Bool => Operation::BoolLit(rng.random_bool(0.5)),
            crate::adt::Type::Int => Operation::IntLit(rng.random_range(1..=100)),
        };
    }

    match return_type {
        crate::adt::Type::Bool => match rng.random_range(1..10) {
            1 => Operation::Neq(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Int,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Int,
                )),
            ),
            2 => Operation::Eq(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Int,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Int,
                )),
            ),
            3 => Operation::Geq(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Int,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Int,
                )),
            ),
            4 => Operation::Leq(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Int,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Int,
                )),
            ),
            5 => Operation::Lt(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Int,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Int,
                )),
            ),
            6 => Operation::Gt(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Int,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Int,
                )),
            ),
            7 => Operation::And(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Bool,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Bool,
                )),
            ),
            8 => Operation::Or(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Bool,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Bool,
                )),
            ),
            9 => Operation::Not(Box::new(generate_operation(
                rng,
                verbose,
                constructor,
                max_depth - 1,
                input.clone(),
                crate::adt::Type::Bool,
            ))),

            _ => panic!(),
        },
        crate::adt::Type::Int => match rng.random_range(1..4) {
            1 => Operation::Add(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Int,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Int,
                )),
            ),
            2 => Operation::Sub(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Int,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Int,
                )),
            ),
            3 => Operation::Mul(
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input.clone(),
                    crate::adt::Type::Int,
                )),
                Box::new(generate_operation(
                    rng,
                    verbose,
                    constructor,
                    max_depth - 1,
                    input,
                    crate::adt::Type::Int,
                )),
            ),
            _ => panic!(),
        },
    }
}
