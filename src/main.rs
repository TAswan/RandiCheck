mod adt;
mod essence;
mod parser;
mod parser_utils;
mod oxide_out;
mod rust_parser;
mod idris_parser;

use crate::parser::{collect_adts, collect_functions};
use crate::parser_utils::{parse_haskell, print_nodes};

fn main() {
    // printing the result of parsing the haskell file using tree sitter

    let input = std::env::args().nth(1).unwrap_or("tests/example.hs".into());

    let verbose = std::env::args().any(|arg| arg == "--verbose");

    let oxide_out = std::env::args().any(|arg| arg == "--oxide-out");

    println!("Input file: {}", input);

    let source_code = std::fs::read_to_string(&input).expect("Could not read file");

    let is_rust = input.ends_with(".rs");
    let is_idris = input.ends_with(".idr");
    let is_haskell = input.ends_with(".hs");

    let tree = if is_haskell {
        parse_haskell(input)
    } else if is_rust {
        rust_parser::parse_rust(input)
    } else if is_idris {
        idris_parser::parse_idris(input)
    } else {
        panic!("Unsupported file type. Please provide a .hs, .rs, or .idr file.");
    };
 
    

    // pretty print the tree
    if verbose {
        println!("--- Syntax Tree ---");
        print_nodes(&tree.root_node(), 0, &source_code, false);
        println!("--- Traversing the tree ---");
    }

    let adt = if is_rust {
        rust_parser::collect_rust_adts(&tree, &source_code, verbose)
    } else if is_haskell{
        collect_adts(&tree, &source_code, verbose)
    } else if is_idris {
        idris_parser::collect_idris_adts(&tree, &source_code, verbose)
    } else {
        panic!("Unsupported file type. Please provide a .hs, .rs, or .idr file.");
    };

    let funcs =  if is_rust {
        rust_parser::collect_rust_functions(&tree, &source_code, &adt.name, verbose)
    } else {
        collect_functions(&tree, &source_code, &adt.name, verbose)
    };
    
    if oxide_out {
        oxide_out::generate_oxide_output(&adt, &funcs, verbose);
        return;
    }
    let essence_spec = essence::essence(adt, funcs, verbose);
    println!("--- Essence Specification ---");
    println!("{}", essence_spec);
}
