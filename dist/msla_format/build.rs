use std::process::Command;

#[cfg(not(unix))]
compile_error!("Generate script only supported on unix systems.");

fn main() {
    Command::new("./generate.sh").output().unwrap();
    println!("cargo:rerun-if-changed=script.sh");
}
