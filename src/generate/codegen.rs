use crate::adt::{Adt, Func};

#[must_use]
pub fn output(
    adt: &Adt,
    funcs: &[Func],
    oxide_out: bool,
    verbose: bool,
    min: i32,
    max: i32,
) -> String {
    if oxide_out {
        // crate::generate::oxide_out::generate_oxide_output(adt, funcs, verbose)
        unimplemented!("Oxide output generation is not implemented yet.");
    } else {
        crate::generate::essence::generate_essence_output(adt, funcs, verbose, min, max)
    }
}
