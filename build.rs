use std::{env::var_os, fs, path::Path, process::Command};

fn main() {
    let commit_hash = String::from_utf8(
        Command::new("sh")
            .args(["-c", "git rev-parse --short HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let commit_hash = commit_hash.trim();

    let out_dir = var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("version.rs");
    fs::write(
        out_path,
        format!(r#"pub const COMMIT_HASH: &str = "{commit_hash}";"#),
    )
    .unwrap();
    println!("cargo::rerun-if-changed=build.rs");
}
