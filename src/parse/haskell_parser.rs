use tree_sitter::{Tree, TreeCursor};

use crate::adt::{Adt, Cons, Func, FuncInput, Operation, Type};
use crate::parse::parser_utils::{
    print_node, print_nodes, traverse_and_capture, traverse_and_capture_from_node,
};

pub fn collect_haskell_adts(tree: &Tree, source_code: &str, verbose: bool) -> Adt {
    let adt_nodes = traverse_and_capture(tree, "data_type");

    assert!(
        (adt_nodes.len() == 1),
        "Expected exactly one Adt in the example file"
    );

    let node = &adt_nodes[0];
    if verbose {
        print!("Adt: ");
        let adt_str = &source_code[node.start_byte()..node.end_byte()];
        println!("{adt_str}\n");
        println!("{}\n", node.to_sexp());
    }

    // get the name of the Adt
    let mut child_cursor = node.walk();
    let mut name_str = String::new();
    if child_cursor.goto_first_child() {
        loop {
            let child = child_cursor.node();
            if child.kind() == "name" {
                name_str = source_code[child.start_byte()..child.end_byte()].to_string();
            }
            if !child_cursor.goto_next_sibling() {
                break;
            }
        }
    }

    // get the constructors of the Adt
    let constructors = traverse_and_capture_from_node(*node, "data_constructor");

    let mut cons_vec: Vec<Cons> = Vec::new();

    for constructor in constructors {
        let prefix_node = traverse_and_capture_from_node(constructor, "constructor");
        assert!(
            (prefix_node.len() == 1),
            "Expected exactly one prefix in the constructor"
        );
        let prefix_str =
            &source_code[prefix_node[0].start_byte()..prefix_node[0].end_byte()].to_string();

        let type_nodes = traverse_and_capture_from_node(constructor, "name");
        let mut type_vec: Vec<Type> = Vec::new();
        for type_node in type_nodes {
            let type_str = &source_code[type_node.start_byte()..type_node.end_byte()];
            let ty = match type_str {
                "Bool" => Type::Bool,
                "Int" => Type::Int,
                _ => panic!("Unknown type: {type_str}"),
            };
            type_vec.push(ty);
        }
        type_vec.reverse();
        let cons = Cons {
            prefix: prefix_str.to_string(),
            types: type_vec,
        };

        cons_vec.push(cons);
    }

    let adt = Adt {
        name: name_str.to_string(),
        constructors: cons_vec,
    };
    if verbose {
        println!("Adt: {adt:?}");
    }
    adt
}

pub fn collect_haskell_functions(
    tree: &Tree,
    source_code: &str,
    name: &str,
    verbose: bool,
) -> Vec<Func> {
    /* for the purposes of this initial version, I assume that every
    function begins with a signature node, and then every sibling
    function node after this is a part of it. we also only have
    one function right now anyway so not much point being super specific
    */
    let sigs = traverse_and_capture(tree, "signature");
    let mut funcs: Vec<Func> = Vec::new();

    for sig in sigs {
        // check if the type in the signature matches the name

        let names = traverse_and_capture_from_node(sig, "name");
        assert!(
            (names.len() == 2),
            "Expected exactly two names in the signature"
        );
        let output_type = &source_code[names[0].start_byte()..names[0].end_byte()];
        let input_name = &source_code[names[1].start_byte()..names[1].end_byte()];
        assert!((input_name == name), "input doesn't match Adt name");
        assert!((output_type == "Bool"), "Expected output type to be Bool");

        // get the sibling nodes that are of kind function
        if verbose {
            print_nodes(&sig, 0, source_code, false);
        }

        let mut sibling_cursor = tree.walk();
        sibling_cursor.goto_first_child();
        sibling_cursor.goto_first_child();
        while sibling_cursor.node() != sig {
            sibling_cursor.goto_next_sibling();
        }

        let mut functions: Vec<tree_sitter::Node> = Vec::new();
        if sibling_cursor.goto_next_sibling() {
            loop {
                let sibling = sibling_cursor.node();
                if sibling.kind() == "function" {
                    functions.push(sibling);
                } else {
                    break;
                }
                if !sibling_cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        let mut input_cons: Vec<FuncInput> = Vec::new();
        for function in functions {
            let patterns = traverse_and_capture_from_node(function, "patterns");

            assert!(
                (patterns.len() == 1),
                "Expected exactly one patterns node in the function. Found {}",
                patterns.len()
            );

            let pattern = &patterns[0];

            let constructors = traverse_and_capture_from_node(*pattern, "constructor");
            assert!(
                (constructors.len() == 1),
                "Expected exactly one constructor in the pattern"
            );
            let constr_prefix =
                &source_code[constructors[0].start_byte()..constructors[0].end_byte()];

            let mut input_strs = Vec::new();
            let inputs = traverse_and_capture_from_node(*pattern, "variable");
            for input in &inputs {
                if verbose {
                    println!(
                        "Input: {}",
                        &source_code[input.start_byte()..input.end_byte()]
                    );
                }
                let input_str = &source_code[input.start_byte()..input.end_byte()];
                input_strs.push(input_str);
            }

            input_strs.reverse(); // because they are captured in reverse order

            let con = FuncInput {
                prefix: constr_prefix.to_string(),
                input: input_strs.iter().map(|s| (*s).to_string()).collect(),
            };
            input_cons.push(con.clone());
            let operations = traverse_and_capture_from_node(function, "match");
            assert!(
                (operations.len() == 1),
                "Expected exactly one match in the function"
            );

            if verbose {
                println!("Operations nodes: {}", operations.len());
                println!("Operations: {operations:?}");
                println!(
                    "Operations source: {}",
                    &source_code[operations[0].start_byte()..operations[0].end_byte()]
                );
            }

            let mut child_cursor = operations[0].walk();
            child_cursor.goto_first_child();
            child_cursor.goto_next_sibling(); // skip "match"

            let operation = parse_operation(&mut child_cursor, source_code, verbose);

            let func = Func {
                con: con.clone(),
                opp: operation,
            };
            funcs.push(func.clone());
        }
    }
    if verbose {
        println!("All functions: {funcs:?}");
    }
    funcs
}

fn parse_operation(cursor: &mut TreeCursor<'_>, source_code: &str, verbose: bool) -> Operation {
    let child = cursor.node();
    if verbose {
        println!("Parsing operation node: {}", child.kind());
        print_node(&child, source_code);
    }

    match child.kind() {
        "infix" => {
            let mut left = None;
            let mut right = None;
            let mut op = None;
            if cursor.goto_first_child() {
                left = Some(parse_operation(&mut cursor.clone(), source_code, verbose));
                cursor.goto_next_sibling();
                op = Some(&source_code[cursor.node().start_byte()..cursor.node().end_byte()]);
                cursor.goto_next_sibling();
                right = Some(parse_operation(&mut cursor.clone(), source_code, verbose));
            }

            if let (Some(left_op), Some(right_op), Some(operator)) = (left, right, op) {
                match operator {
                    ">" => Operation::Gt(Box::new(left_op), Box::new(right_op)),
                    "<" => Operation::Lt(Box::new(left_op), Box::new(right_op)),
                    "==" => Operation::Eq(Box::new(left_op), Box::new(right_op)),
                    "/=" => Operation::Neq(Box::new(left_op), Box::new(right_op)),
                    "<=" => Operation::Leq(Box::new(left_op), Box::new(right_op)),
                    ">=" => Operation::Geq(Box::new(left_op), Box::new(right_op)),
                    "+" => Operation::Add(Box::new(left_op), Box::new(right_op)),
                    "-" => Operation::Sub(Box::new(left_op), Box::new(right_op)),
                    "*" => Operation::Mul(Box::new(left_op), Box::new(right_op)),
                    "&&" => Operation::And(Box::new(left_op), Box::new(right_op)),
                    "||" => Operation::Or(Box::new(left_op), Box::new(right_op)),
                    _ => panic!("Unknown operator: {operator}"),
                }
            } else {
                panic!("Incomplete infix operation");
            }
        }

        "literal" => {
            let val = &source_code[child.start_byte()..child.end_byte()];
            Operation::IntLit(val.parse::<i32>().unwrap())
        }
        "variable" | "name" => {
            let name = &source_code[child.start_byte()..child.end_byte()];
            if verbose {
                println!("Variable/Name node:");
                print_nodes(&child, 0, source_code, false);
                println!("Name: {}", name);
            }
            if name == "True" {
                Operation::BoolLit(true)
            } else if name == "False" {
                Operation::BoolLit(false)
            } else {
                Operation::Var(name.to_string())
            }
        }
        "parens" => {
            cursor.goto_first_child(); // go to '('
            cursor.goto_next_sibling(); // go to inner expression
            parse_operation(cursor, source_code, verbose)
        }
        "constructor" => {
            let constr_name = &source_code[child.start_byte()..child.end_byte()];
            if constr_name == "True" {
                Operation::BoolLit(true)
            } else if constr_name == "False" {
                Operation::BoolLit(false)
            } else {
                panic!("Unknown constructor: {constr_name}");
            }
        }

        _ => panic!(
            "Unknown operand kind: {}, {:?}",
            child.kind(),
            print_nodes(&child, 0, source_code, false)
        ),
    }
}
