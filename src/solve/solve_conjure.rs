use std::process::{Command, Stdio};


pub fn solve_conjure(essence_file: String, verbose : bool) {
    
       

    let conjure_output = Command::new("conjure")
        .arg("solve")
        .arg(essence_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute conjure");

    if verbose {
        println!("Conjure stdout:\n{}", String::from_utf8_lossy(&conjure_output.stdout));
        eprintln!("Conjure stderr:\n{}", String::from_utf8_lossy(&conjure_output.stderr));
    }

    if !conjure_output.status.success() {
        panic!(
            "Conjure failed with status {}",
            conjure_output.status.code().unwrap_or(-1)
        );
    }
}

