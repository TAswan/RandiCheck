// given the solutions from conjure and the parsed ADT and functions, write haskell code that validates the solutions
use crate::adt::Adt;
use crate::adt::Func;
use serde;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Write as _;
use tera::{Context, Tera};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Assignments {
    var: String,
    val: String,
}

//hack to make func easier for the template, but probably could swap to just display later
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FuncInput {
    input: String,
    opp: String,
}

#[must_use]
pub fn generate_haskell_validation(
    adt: Adt,
    funcs: Vec<Func>,
    assignments: &Vec<(String, String)>,
    verbose: bool,
) -> bool {
    let tera = Tera::new("src/templates/*.tera").unwrap();
    let mut context = Context::new();

    context.insert("adt", &adt);
    context.insert("funcs", &gen_predicate(funcs.clone()));
    context.insert("assignments", &gen_value(adt, assignments, verbose));

    let validation_code = tera.render("haskell.tera", &context).unwrap();

    if verbose {
        println!("Generated Haskell validation code:\n{}", &validation_code);
    }

    std::fs::write("validation.hs", validation_code).expect("Could not write validation file");

    let cmd = std::process::Command::new("ghc")
        .arg("validation.hs")
        .output()
        .expect("Failed to execute Haskell validation code");

    if verbose {
        println!(
            "Haskell compilation output: {}",
            String::from_utf8_lossy(&cmd.stdout)
        );
        println!(
            "Haskell compilation errors: {}",
            String::from_utf8_lossy(&cmd.stderr)
        );
    }

    let cmd = std::process::Command::new("./validation")
        .output()
        .expect("Failed to execute Haskell validation code");

    if verbose {
        println!(
            "Haskell validation output: {}",
            String::from_utf8_lossy(&cmd.stdout)
        );
        println!(
            "Haskell validation errors: {}",
            String::from_utf8_lossy(&cmd.stderr)
        );
    }

    // check if the output contains "True"
    String::from_utf8_lossy(&cmd.stdout).contains("True")
}

fn gen_predicate(funcs: Vec<Func>) -> Vec<FuncInput> {
    // generate haskell code for predicates
    let mut result = Vec::new();

    for func in funcs {
        let mut pred_code = String::new();

        let _ = writeln!(pred_code, "{} ", func.opp.to_haskell());

        result.push(FuncInput {
            input: func.con.to_string(),
            opp: pred_code,
        });
    }
    result
}

fn gen_value(adt: Adt, assignments: &Vec<(String, String)>, verbose: bool) -> Assignments {
    // generate haskell code for values
    let mut value_code = String::new();

    let _ = writeln!(value_code, "value :: {}", adt.name);

    value_code.push_str("value = ");
    // find the value of the tag in the assignments
    let tag_value = assignments
        .iter()
        .find(|(var, _)| var == "tag")
        .map(|(_, val)| val.clone())
        .unwrap();
    let tag_value_int: i32 = tag_value.parse().unwrap();

    if verbose {
        println!("Generating Haskell value code for tag value: {tag_value_int}");
    }
    // find the constructor corresponding to the tag value
    // reverse the adt because of previous stack shenanigans
    let adt = Adt {
        name: adt.name,
        constructors: adt.constructors.into_iter().rev().collect(),
    };

    let constructor = &adt.constructors[(tag_value_int - 1) as usize];
    let _ = write!(value_code, "{} ", constructor.prefix);

    let mut value = String::new();
    // find the value of each field in the assignments where the variable name begins with the constructor prefix
    for (var, val) in assignments {
        if var.starts_with(&constructor.prefix) {
            if val == "true" {
                let _ = write!(value_code, "True ");
                value.push_str("True ");
                continue;
            } else if val == "false" {
                let _ = write!(value_code, "False ");
                value.push_str("False ");
                continue;
            }
            let _ = write!(value_code, "{val} ");
        }
    }

    if verbose {
        println!("Generated Haskell value code:\n{}", &value_code);
    }

    Assignments {
        var: constructor.prefix.clone(),
        val: value.trim().to_string(),
    }
}
