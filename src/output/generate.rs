use std::fs::File;

use crate::adt::{Adt, Func};

pub fn output(adt: Adt, funcs: Vec<Func>, oxide_out: bool, verbose: bool) -> File {
    if oxide_out {
        crate::output::oxide_out::generate_oxide_output(adt, funcs, verbose)
    } else {
        crate::output::essence::generate_essence_output(adt, funcs, verbose)
    }
}
