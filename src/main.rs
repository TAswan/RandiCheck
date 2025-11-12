use clap::Parser;
use randi_check::generate::codegen;
use randi_check::parse::parser;
use randi_check::solve::solve_conjure::solve_conjure;
use randi_check::validate::parse_solution::parse_essence_output;

mod adt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (default: tests/example.hs)
    #[arg(value_name = "INPUT", default_value = "input_files/example.hs")]
    input: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Emit oxide output instead of essence
    #[arg(long = "oxide-out")]
    oxide_out: bool,

    ///Create a new haskell file randomly
    #[arg(short, long)]
    generate: bool,
}

fn main() {
    let args = Args::parse();

    let input = args.input;
    let verbose = args.verbose;
    let oxide_out = args.oxide_out;
    let generate = args.generate;

    if generate {
        randi_check::randomGeneration::new_haskell::generate_haskell_random(6, verbose);
        return;
    }

    let source_code = std::fs::read_to_string(&input).expect("Could not read file");

    let filetype = input.split('.').next_back().unwrap_or("");

    let (adt, funcs) = parser::parse(&source_code, filetype, verbose);

    let spec = codegen::output(&adt, &funcs, oxide_out, verbose);

    solve_conjure(spec, verbose);

    let assignments = parse_essence_output("output.solution", verbose);

    let valid = randi_check::validate::gen_haskell::generate_haskell_validation(
        adt,
        funcs,
        &assignments,
        verbose,
    );

    if valid {
        println!("Validation succeeded: The solution satisfies the predicates.");
    } else {
        println!("Validation failed: The solution does not satisfy the predicates.");
    }
}
