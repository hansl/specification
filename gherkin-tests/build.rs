use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("all.cddl");

    let output = Command::new("make")
        .args(&["compile-cddl"])
        .env("CDDL_OUT_PATH", &dest_path)
        .current_dir("..")
        .output()
        .unwrap();
    assert!(output.status.success(), "Make failed.");

    fs::write(&dest_path, output.stdout).unwrap();
    println!(
        "cargo:rerun-if-changed={}",
        env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
    );
    println!("cargo:rerun-if-changed={}", dest_path.to_string_lossy());
}
