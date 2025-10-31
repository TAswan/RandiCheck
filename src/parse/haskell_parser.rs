use tree_sitter::Tree;

use crate::adt::{Adt, Cons, Func, FuncInput, Operand, Operation, Type};
use crate::parse::parser_utils::{
    print_node, print_nodes, traverse_and_capture, traverse_and_capture_from_node,
};

pub fn collect_haskell_adts(tree: &Tree, source_code: &str, verbose: bool) -> Adt {
    let adt_nodes = traverse_and_capture(tree, "data_type");

    if adt_nodes.len() != 1 {
        panic!("Expected exactly one Adt in the example file");
    }

    let node = &adt_nodes[0];
    if verbose {
        print!("Adt: ");
        let adt_str = &source_code[node.start_byte()..node.end_byte()];
        println!("{}\n", adt_str);
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
        if prefix_node.len() != 1 {
            panic!("Expected exactly one prefix in the constructor");
        }
        let prefix_str =
            &source_code[prefix_node[0].start_byte()..prefix_node[0].end_byte()].to_string();

        let type_nodes = traverse_and_capture_from_node(constructor, "name");
        let mut type_vec: Vec<Type> = Vec::new();
        for type_node in type_nodes {
            let type_str = &source_code[type_node.start_byte()..type_node.end_byte()];
            let ty = match type_str {
                "Bool" => Type::Bool,
                "Int" => Type::Int,
                _ => panic!("Unknown type: {}", type_str),
            };
            type_vec.push(ty);
        }

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
        println!("Adt: {:?}", adt);
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
        if names.len() != 2 {
            panic!("Expected exactly two names in the signature");
        }
        let output_type = &source_code[names[0].start_byte()..names[0].end_byte()];
        let input_name = &source_code[names[1].start_byte()..names[1].end_byte()];
        if input_name != name {
            panic!("input doesn't match Adt name");
        }
        if output_type != "Bool" {
            panic!("Expected output type to be Bool");
        }

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
            let constructors = traverse_and_capture_from_node(function, "constructor");
            if constructors.len() != 1 {
                panic!("expected only one constructor")
            }
            let constr_prefix =
                &source_code[constructors[0].start_byte()..constructors[0].end_byte()];

            let patterns = traverse_and_capture_from_node(function, "patterns");

            let mut input_strs = Vec::new();
            for pattern in patterns {
                let inputs = traverse_and_capture_from_node(pattern, "variable");
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

                input_strs.reverse();
            }
            let con = FuncInput {
                prefix: constr_prefix.to_string(),
                input: input_strs.iter().map(|s| s.to_string()).collect(),
            };
            input_cons.push(con.clone());
            let operations = traverse_and_capture_from_node(function, "match");
            if operations.len() != 1 {
                panic!("Expected exactly one match in the function");
            }

            if verbose {
                println!("Operations nodes: {}", operations.len());
                println!("Operations: {:?}", operations);
                println!(
                    "Operations source: {}",
                    &source_code[operations[0].start_byte()..operations[0].end_byte()]
                );
            }

            // annoyingly tree sitter represents infix expressions as nested nodes, but just left to right
            // rather than respecting operator precedence

            let operation = parse_operation(operations[0], source_code, verbose);

            let func = Func {
                con: con.clone(),
                opp: operation,
            };
            funcs.push(func.clone());
        }
    }
    if verbose {
        println!("All functions: {:?}", funcs);
    }
    funcs
}

fn parse_operation(node: tree_sitter::Node, source_code: &str, verbose: bool) -> Operation {
    let mut child_cursor = node.walk();
    child_cursor.goto_first_child();
    child_cursor.goto_next_sibling(); // skip "match"
    let child = child_cursor.node();
    if verbose {
        println!("Parsing operation node: {}", child.kind());
        print_node(&child, source_code);
    }
    if child.kind() == "infix" {
        let infix = parse_infix(child, source_code);
        if verbose {
            println!("Parsed infix operation: {:?}", infix);
        }

        let mut is_infix = false;
        // check if left and right operands are also infix in case we have to fix precedence
        if let (Some(left), Some(right)) = (infix.left(), infix.right()) {
            if let Operand::Infix(left_infix) = left {
                if verbose {
                    println!("Left operand is an infix: {:?}", left_infix);
                }
                is_infix = true;
                // Here you would implement precedence handling if needed
            }
            if let Operand::Infix(right_infix) = right {
                if verbose {
                    println!("Right operand is an infix: {:?}", right_infix);
                }
                is_infix = true;
                // Here you would implement precedence handling if needed
            }
        }

        if is_infix {
            return precedence_swap(infix.clone());
        }

        return infix;
    }

    if child_cursor.goto_next_sibling() {
        print_node(&child_cursor.node(), source_code);
        panic!("Unexpected additional nodes in operation");
    }

    Operation::ConstSelf
}

fn parse_operand(node: tree_sitter::Node, source_code: &str) -> Operand {
    match node.kind() {
        "literal" => {
            let val = &source_code[node.start_byte()..node.end_byte()];
            Operand::Lit(val.parse::<i32>().unwrap())
        }
        "variable" | "name" => {
            let name = &source_code[node.start_byte()..node.end_byte()];
            Operand::Var(name.to_string())
        }
        _ => panic!("Unknown operand kind: {}", node.kind()),
    }
}

fn parse_infix(node: tree_sitter::Node, source_code: &str) -> Operation {
    let mut child_cursor = node.walk();
    let mut left = None;
    let mut right = None;
    let mut op = None;
    if child_cursor.goto_first_child() {
        loop {
            let child = child_cursor.node();
            match child.kind() {
                "infix" => {
                    let infix = parse_infix(child, source_code);
                    if left.is_none() {
                        left = Some(Operand::Infix(Box::new(infix)));
                    } else {
                        right = Some(Operand::Infix(Box::new(infix)));
                    }
                }
                "operator" => {
                    let operator = &source_code[child.start_byte()..child.end_byte()];
                    op = Some(operator.to_string());
                }
                _ => {
                    let operand = parse_operand(child, source_code);
                    if left.is_none() {
                        left = Some(operand);
                    } else {
                        right = Some(operand);
                    }
                }
            }
            if !child_cursor.goto_next_sibling() {
                break;
            }
        }
    }
    if let (Some(left_op), Some(right_op), Some(operator)) = (left, right, op) {
        match operator.as_str() {
            ">" => Operation::Gt(left_op, right_op),
            "<" => Operation::Lt(left_op, right_op),
            "==" => Operation::Eq(left_op, right_op),
            "/=" => Operation::Neq(left_op, right_op),
            "<=" => Operation::Leq(left_op, right_op),
            ">=" => Operation::Geq(left_op, right_op),
            "+" => Operation::Add(left_op, right_op),
            _ => panic!("Unknown operator: {}", operator),
        }
    } else {
        panic!("Incomplete infix operation");
    }
}

/*
currently because tree sitter parses infix expressions left to right without
respecting operator precedence, we may have to swap some nodes around to get
the correct precedence.

i think the only case we have to worry about is when the right operand
is itself an infix operation with higher precedence than the current one.

*/
fn precedence_swap(infix: Operation) -> Operation {
    let (left_op, right_op) = match &infix {
        Operation::Gt(l, r)
        | Operation::Lt(l, r)
        | Operation::Eq(l, r)
        | Operation::Neq(l, r)
        | Operation::Leq(l, r)
        | Operation::Geq(l, r)
        | Operation::Add(l, r) => (l.clone(), r.clone()),
        _ => return infix,
    };
    if let Operand::Infix(boxed_right_infix) = right_op {
        let right_precedence = precedence((*boxed_right_infix).clone());
        let current_precedence = precedence(infix.clone());
        if right_precedence > current_precedence {
            // we need to swap
            // extract the left and right operands of the right infix
            let (right_left_op, right_right_op) = match &*boxed_right_infix {
                Operation::Gt(l, r)
                | Operation::Lt(l, r)
                | Operation::Eq(l, r)
                | Operation::Neq(l, r)
                | Operation::Leq(l, r)
                | Operation::Geq(l, r)
                | Operation::Add(l, r) => (l.clone(), r.clone()),
                _ => return infix,
            };

            // create new infix for the current operation with left_op and right_left_op
            let new_current_infix = match &infix {
                Operation::Gt(_, _) => Operation::Gt(left_op, right_left_op),
                Operation::Lt(_, _) => Operation::Lt(left_op, right_left_op),
                Operation::Eq(_, _) => Operation::Eq(left_op, right_left_op),
                Operation::Neq(_, _) => Operation::Neq(left_op, right_left_op),
                Operation::Leq(_, _) => Operation::Leq(left_op, right_left_op),
                Operation::Geq(_, _) => Operation::Geq(left_op, right_left_op),
                Operation::Add(_, _) => Operation::Add(left_op, right_left_op),
                _ => return infix,
            };

            // now create new infix for the right operation with new_current_infix and right_right_op
            let new_infix = match &*boxed_right_infix {
                Operation::Gt(_, _) => {
                    Operation::Gt(Operand::Infix(Box::new(new_current_infix)), right_right_op)
                }
                Operation::Lt(_, _) => {
                    Operation::Lt(Operand::Infix(Box::new(new_current_infix)), right_right_op)
                }
                Operation::Eq(_, _) => {
                    Operation::Eq(Operand::Infix(Box::new(new_current_infix)), right_right_op)
                }
                Operation::Neq(_, _) => {
                    Operation::Neq(Operand::Infix(Box::new(new_current_infix)), right_right_op)
                }
                Operation::Leq(_, _) => {
                    Operation::Leq(Operand::Infix(Box::new(new_current_infix)), right_right_op)
                }
                Operation::Geq(_, _) => {
                    Operation::Geq(Operand::Infix(Box::new(new_current_infix)), right_right_op)
                }
                Operation::Add(_, _) => {
                    Operation::Add(Operand::Infix(Box::new(new_current_infix)), right_right_op)
                }
                _ => return infix,
            };
            return new_infix;
        }
    }
    infix
}

fn precedence(op: Operation) -> u8 {
    match op {
        Operation::Add(_, _) => 1,
        Operation::Gt(_, _)
        | Operation::Lt(_, _)
        | Operation::Eq(_, _)
        | Operation::Neq(_, _)
        | Operation::Leq(_, _)
        | Operation::Geq(_, _) => 2,
        _ => 0,
    }
}
