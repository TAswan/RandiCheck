/// parses the solution output from Conjure

pub fn parse_essence_output(sol_path: &str, verbose: bool) -> Vec<(String, String)> {
    let solution = std::fs::read_to_string(sol_path).expect("Could not read solution file");

    if verbose {
        println!("Solution:\n{}", &solution);
    }

    // turn each line into a tuple of (variable, value) by splitting at the first word after "letting" and the word "be"
    let assignments: Vec<(String, String)> = solution
        .lines()
        .filter(|line| line.starts_with("letting"))
        .map(|line| {
            let parts: Vec<&str> = line["letting ".len()..].split(" be ").collect();
            (parts[0].trim().to_string(), parts[1].trim().to_string())
        })
        .collect();

    for (var, val) in &assignments {
        if verbose {
            println!("Parsed assignment: {} = {}", var, val);
        }
    }

    assignments
}
