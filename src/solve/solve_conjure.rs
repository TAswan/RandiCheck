use std::process::{Command, Stdio};

/// Solve the given Essence specification using Conjure
/// # Panics
/// Panics if Conjure fails to execute or returns a non-zero status.
pub fn solve_conjure(essence_file: String, verbose: bool) {
    let conjure_output = Command::new("conjure")
        .arg("solve")
        .arg(essence_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute conjure");

    if verbose {
        println!(
            "Conjure stdout:\n{}",
            String::from_utf8_lossy(&conjure_output.stdout)
        );
        eprintln!(
            "Conjure stderr:\n{}",
            String::from_utf8_lossy(&conjure_output.stderr)
        );
    }

    assert!(
        conjure_output.status.success(),
        "Conjure failed with status {}",
        conjure_output.status.code().unwrap_or(-1)
    );
}
