use tree_sitter::Tree;

use crate::adt::{Adt, Cons};
use crate::parse::parser_utils::{print_node, print_nodes, traverse_and_capture_from_node};

pub fn collect_idris_adts(tree: &Tree, source_code: &str, verbose: bool) -> Adt {
    let root = tree.root_node();

    let adt_nodes = traverse_and_capture_from_node(root, "data");
    // let mut adts = Vec::new();
    if adt_nodes.len() != 2 {
        panic!(
            "Expected exactly one ADT in the source file, found {}",
            adt_nodes.len()
        );
    }

    let adt_node = &adt_nodes[0];

    if verbose {
        print_nodes(adt_node, 0, source_code, false);
    }

    let name = {
        let binding = traverse_and_capture_from_node(*adt_node, "data_name");
        let adt_name_node = binding.first().expect("Could not find ADT name node");

        source_code[adt_name_node.start_byte()..adt_name_node.end_byte()].to_string()
    };

    if verbose {
        println!("ADT Name: {}", name);
    }

    let cursor = &mut adt_node.walk();
    let mut constructors: Vec<Cons> = Vec::new();
    for child in adt_node.children(&mut cursor.clone()) {
        if child.kind() == "exp_name" {
            // process constructor
            print_node(&child, source_code);
            let prefix_name = &source_code[child.start_byte()..child.end_byte()];

            while cursor.goto_next_sibling() {}
            constructors.push(Cons {
                prefix: prefix_name.to_owned(),
                types: Vec::new(),
            });

            panic!("Not yet implemented");
        }
    }

    panic!("Not yet implemented");
}

pub fn collect_idris_functions(
    _tree: &Tree,
    _source_code: &str,
    _name: &str,
    _verbose: bool,
) -> Vec<crate::adt::Func> {
    panic!("Not yet implemented");
}
