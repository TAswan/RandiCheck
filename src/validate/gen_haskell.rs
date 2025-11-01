// given the solutions from conjure and the parsed ADT and functions, write haskell code that validates the solutions
use std::fmt::Write as _;
use crate::adt::Adt;
use crate::adt::Func;


pub fn generate_haskell_validation(adt: Adt, funcs: Vec<Func>, assignments: &Vec<(String, String)>, verbose: bool) -> bool {
    let mut validation_code = String::new();

    // Generate code to reconstruct the ADT
    validation_code.push_str(gen_adt(&adt, verbose).as_str());
    validation_code.push_str("\n");

    // generate code for the predicate
    validation_code.push_str(gen_predicate( &adt, funcs, verbose).as_str());
    validation_code.push_str("\n");

    validation_code.push_str(&gen_value(adt, assignments, verbose));

    validation_code.push_str("\nmain :: IO ()\nmain = print(predicate value)\n");


    std::fs::write("validation.hs", validation_code).expect("Could not write validation file");

    // run the haskell code

    let cmd = std::process::Command::new("ghc")
        .arg("validation.hs")
        .output()
        .expect("Failed to execute Haskell validation code");

    if verbose {
        println!("Haskell compilation output: {}", String::from_utf8_lossy(&cmd.stdout));
        println!("Haskell compilation errors: {}", String::from_utf8_lossy(&cmd.stderr));
    }

    let cmd = std::process::Command::new("./validation")
        .output()
        .expect("Failed to execute Haskell validation code");

    if verbose {
        println!("Haskell validation output: {}", String::from_utf8_lossy(&cmd.stdout));
        println!("Haskell validation errors: {}", String::from_utf8_lossy(&cmd.stderr));
    }

    // check if the output contains "True"
    String::from_utf8_lossy(&cmd.stdout).contains("True")


    
}

fn gen_adt(adt: &Adt, verbose: bool) -> String {
    // generate haskell code for adt
    let mut adt_code = String::new();
    let _ = write!(adt_code, "data {} = \n", adt.name);
    for con in adt.constructors.clone() {
        let _ = write!(adt_code, "    {} ", &con.prefix);
        for t in &con.types {
            let _ = write!(adt_code, "{:?} ", t);
        }
        // if not the last constructor, add a pipe
        if con != *adt.constructors.last().unwrap() {
        adt_code.push_str("|\n");
        }
    }
    adt_code.push_str("\n");

    if verbose {
        println!("Generated Haskell ADT code:\n{}", &adt_code);
    }

    adt_code

}

fn gen_predicate(adt: &Adt, funcs: Vec<Func>, verbose: bool) -> String {
    // generate haskell code for predicates
    let mut pred_code = String::new();
    let _ = write!(pred_code, "predicate :: {} -> Bool\n", adt.name);

    for func in funcs {
        let _ = write!(pred_code, "predicate {} = ", func.con);
        if func.opp == crate::adt::Operation::ConstSelf {
            let _ = write!(pred_code, "{} \n", func.con.input[0]);
            continue;
        }
        let _ = write!(pred_code, "{} \n", func.opp);
    }
    if verbose {
        println!("Generated Haskell predicate code:\n{}", &pred_code);
    }

    pred_code
}

fn gen_value(adt: Adt, assignments: &Vec<(String, String)>, verbose: bool) -> String {
    // generate haskell code for values
    let mut value_code = String::new();
    
    let _ = write!(value_code, "value :: {}\n", adt.name);
    
    value_code.push_str("value = ");
    // find the value of the tag in the assignments
    let tag_value = assignments.iter().find(|(var, _)| var == "tag").map(|(_, val)| val.clone()).unwrap();
    let tag_value_int: i32 = tag_value.parse().unwrap();

    if verbose {
        println!("Generating Haskell value code for tag value: {}", tag_value_int);
    }
    // find the constructor corresponding to the tag value
    let constructor = &adt.constructors[(tag_value_int - 1) as usize];
    let _ = write!(value_code, "{} ", constructor.prefix);
  
    // find the value of each field in the assignments where the variable name begins with the constructor prefix 
    for (var, val) in assignments {
        if var.starts_with(&constructor.prefix) {
            if val == "true" {
                let _ = write!(value_code, "True ");
                continue;
            } else if val == "false" {
                let _ = write!(value_code, "False ");
                continue;
            }
            let _ = write!(value_code, "{} ", val);
        }
    }

    if verbose {
        println!("Generated Haskell value code:\n{}", &value_code);
    }


    value_code


}