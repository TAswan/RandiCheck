use crate::adt::{Adt, Func};

#[must_use]
pub fn output(adt: &Adt, funcs: &[Func], oxide_out: bool, verbose: bool) -> String {
    if oxide_out {
        crate::generate::oxide_out::generate_oxide_output(adt, funcs, verbose)
    } else {
        crate::generate::essence::generate_essence_output(adt, funcs, verbose)
    }
}
