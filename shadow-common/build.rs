use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn get_git_version() -> String {
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let git_hash = String::from_utf8(
        Command::new("git")
            .args(["describe", "--always"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    format!("{} - {}", version, git_hash)
}

fn write_version() {
    let version = get_git_version();
    File::create(Path::new(&env::var("OUT_DIR").unwrap()).join("VERSION"))
        .unwrap()
        .write_all(version.trim().as_bytes())
        .unwrap();
}

fn main() {
    write_version();
}
