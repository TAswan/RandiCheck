use clap::Parser;
use randi_check::output::generate;
use randi_check::parse::parser;

mod adt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (default: tests/example.hs)
    #[arg(value_name = "INPUT", default_value = "tests/example.hs")]
    input: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Emit oxide output instead of essence
    #[arg(long = "oxide-out")]
    oxide_out: bool,
}

fn main() {
    let args = Args::parse();

    let input = args.input;
    let verbose = args.verbose;
    let oxide_out = args.oxide_out;

    let source_code = std::fs::read_to_string(&input).expect("Could not read file");

    let filetype = input.split('.').next_back().unwrap_or("");

    let (adt, funcs) = parser::parse(&source_code, filetype, verbose);

    let _spec = generate::output(adt, funcs, oxide_out, verbose);
}
