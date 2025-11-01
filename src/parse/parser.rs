// this file works as a wrapper for the 3 languages, handling input from the main file and forwarding to the expected parser.
use crate::parse::haskell_parser;
use crate::parse::idris_parser;
use crate::parse::rust_parser;

#[must_use]
/// # Panics
/// Panics if the file type is not supported, ie not a haskell, rust or idris file.
pub fn parse(
    source_code: &str,
    file_type: &str,
    verbose: bool,
) -> (crate::adt::Adt, Vec<crate::adt::Func>) {
    let mut parser = tree_sitter::Parser::new();

    let language = match file_type {
        "hs" => tree_sitter_haskell::LANGUAGE,
        "rs" => tree_sitter_rust::LANGUAGE,
        "idr" => tree_sitter_idris::LANGUAGE,
        _ => panic!("Unsupported file type: {file_type}"),
    };

    parser
        .set_language(&language.into())
        .expect("Error loading parser");

    let tree = parser
        .parse(source_code, None)
        .expect("Error parsing source code");

    if verbose {
        println!("--- Syntax Tree ---");
        crate::parse::parser_utils::print_nodes(&tree.root_node(), 0, source_code, false);
        println!("--- Traversing the tree ---");
    }

    let adt = match file_type {
        "hs" => haskell_parser::collect_haskell_adts(&tree, source_code, verbose),
        "rs" => rust_parser::collect_rust_adts(&tree, source_code, verbose),
        "idr" => idris_parser::collect_idris_adts(&tree, source_code, verbose),
        _ => panic!("Unsupported file type: {file_type}"),
    };

    let funcs = match file_type {
        "hs" => haskell_parser::collect_haskell_functions(&tree, source_code, &adt.name, verbose),
        "rs" => rust_parser::collect_rust_functions(&tree, source_code, &adt.name, verbose),
        "idr" => idris_parser::collect_idris_functions(&tree, source_code, &adt.name, verbose),
        _ => panic!("Unsupported file type: {file_type}"),
    };

    (adt, funcs)
}
