use tree_sitter::Tree;

use crate::adt::{Adt, Cons, Func, FuncInput, Operand, Operation, Type};
use crate::parse::parser_utils::{print_nodes, traverse_and_capture_from_node};

pub fn collect_rust_adts(tree: &Tree, source_code: &str, verbose: bool) -> Adt {
    let root = tree.root_node();

    let adt_nodes = traverse_and_capture_from_node(root, "enum_item");
    // let mut adts = Vec::new();
    if adt_nodes.len() != 1 {
        panic!(
            "Expected exactly one ADT in the source file, found {}",
            adt_nodes.len()
        );
    }

    let adt_node = &adt_nodes[0];

    if verbose {
        print_nodes(adt_node, 0, source_code, false);
    }

    let binding = traverse_and_capture_from_node(*adt_node, "type_identifier");
    let adt_name_node = binding.first().expect("Could not find ADT name node");

    let adt_name = source_code[adt_name_node.start_byte()..adt_name_node.end_byte()].to_string();
    if verbose {
        println!("ADT Name: {}", adt_name);
    }

    let constructor_nodes = traverse_and_capture_from_node(*adt_node, "enum_variant");

    let mut constructors: Vec<Cons> = Vec::new();

    for constructor_node in constructor_nodes {
        let binding = traverse_and_capture_from_node(constructor_node, "identifier");
        let constructor_name_node = binding
            .first()
            .expect("Could not find constructor name node");

        let constructor_name = source_code
            [constructor_name_node.start_byte()..constructor_name_node.end_byte()]
            .to_string();

        if verbose {
            println!("Constructor Name: {}", constructor_name);
        }

        let type_nodes = traverse_and_capture_from_node(constructor_node, "primitive_type");

        let mut types: Vec<Type> = Vec::new();
        for type_node in type_nodes {
            let type_str = source_code[type_node.start_byte()..type_node.end_byte()].to_string();
            let ty = match type_str.as_str() {
                "i32" => Type::Int,
                "bool" => Type::Bool,
                _ => panic!("Unsupported type: {}", type_str),
            };
            types.push(ty.clone());
            if verbose {
                println!("Constructor Type: {:?}", ty);
            }
        }
        let constructor = Cons {
            prefix: constructor_name,
            types,
        };
        constructors.push(constructor);
    }
    if verbose {
        println!("Constructors: {:?}", constructors);
    }
    Adt {
        name: adt_name,
        constructors,
    }
}

pub fn collect_rust_functions(
    tree: &Tree,
    source_code: &str,
    _adt_name: &str,
    verbose: bool,
) -> Vec<Func> {
    let functions = traverse_and_capture_from_node(tree.root_node(), "function_item");
    if functions.len() != 1 {
        panic!(
            "Expected exactly one function in the source file, found {}",
            functions.len()
        );
    }

    let function_node = &functions[0];

    if verbose {
        print_nodes(function_node, 0, source_code, false);
    }

    let mut funcs = Vec::new();
    //currently we only support one function per file with a match statement inside it
    let func_nodes = traverse_and_capture_from_node(*function_node, "match_arm");

    for func_node in func_nodes {
        // parse each match arm into a Func
        let mut cursor = func_node.walk();
        cursor.goto_first_child();
        let pattern_node = cursor.node();
        let mut identifiers = traverse_and_capture_from_node(pattern_node, "identifier");
        identifiers
            .pop()
            .expect("empty identifiers in function pattern");

        let constructor_name_node = identifiers
            .pop()
            .expect("Could not find function constructor name node");

        let constructor_name = source_code
            [constructor_name_node.start_byte()..constructor_name_node.end_byte()]
            .to_string();
        if verbose {
            println!("Function Constructor: {}", constructor_name);
        }

        let mut inputs = Vec::new();

        while let Some(input_node) = identifiers.pop() {
            let input_name =
                source_code[input_node.start_byte()..input_node.end_byte()].to_string();
            inputs.push(input_name);
        }
        inputs.reverse();
        if verbose {
            println!("Function Inputs: {:?}", inputs);
        }

        let func_input = FuncInput {
            prefix: constructor_name,
            input: inputs,
        };

        let mut cursor = func_node.walk();
        cursor.goto_first_child();
        loop {
            if cursor.node().kind() == "=>" {
                cursor.goto_next_sibling();
                break;
            }
            if !cursor.goto_next_sibling() {
                panic!("Could not find function expression node");
            }
        }

        let expr_node = cursor.node();
        let expr_text = source_code[expr_node.start_byte()..expr_node.end_byte()].to_string();
        if verbose {
            println!("Function Expression: {}", expr_text);
        }

        match expr_node.kind() {
            "identifier" => {
                let value = source_code[expr_node.start_byte()..expr_node.end_byte()].to_string();
                if verbose {
                    println!("Function is ConstSelf with value: {}", value);
                }
                // create Func with ConstSelf
                let func = Func {
                    con: func_input,
                    opp: Operation::ConstSelf,
                };
                funcs.push(func);
            }
            "binary_expression" => {
                let mut bin_cursor = expr_node.walk();
                bin_cursor.goto_first_child();
                let left_node = bin_cursor.node();
                bin_cursor.goto_next_sibling();
                let operator_node = bin_cursor.node();
                bin_cursor.goto_next_sibling();
                let right_node = bin_cursor.node();
                let left_value =
                    source_code[left_node.start_byte()..left_node.end_byte()].to_string();
                let right_value =
                    source_code[right_node.start_byte()..right_node.end_byte()].to_string();
                let operator_value =
                    source_code[operator_node.start_byte()..operator_node.end_byte()].to_string();
                if verbose {
                    println!(
                        "Left: {}, Operator: {}, Right: {}",
                        left_value, operator_value, right_value
                    );
                }
                let left_operand = Operand::Var(left_value);

                let right_operand = if let Ok(n) = right_value.parse::<i32>() {
                    Operand::Lit(n)
                } else {
                    Operand::Var(right_value)
                };

                let operation = match operator_value.as_str() {
                    "+" => Operation::Add(left_operand, right_operand),
                    ">" => Operation::Gt(left_operand, right_operand),
                    "<" => Operation::Lt(left_operand, right_operand),
                    "==" => Operation::Eq(left_operand, right_operand),
                    "!=" => Operation::Neq(left_operand, right_operand),
                    ">=" => Operation::Geq(left_operand, right_operand),
                    "<=" => Operation::Leq(left_operand, right_operand),
                    _ => panic!("Unsupported operator: {}", operator_value),
                };

                let func = Func {
                    con: func_input,
                    opp: operation,
                };
                funcs.push(func);
            }
            _ => {
                panic!("Unsupported function expression kind: {}", expr_node.kind());
            }
        }
    }

    funcs
}
