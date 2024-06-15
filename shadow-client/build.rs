use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

fn get_git_version() -> String {
    let version = env::var("CARGO_PKG_VERSION").unwrap();

    let child = Command::new("git").args(["describe", "--always"]).output();
    match child {
        Ok(child) => {
            let buf = String::from_utf8(child.stdout).expect("failed to read stdout");
            version + "-" + &buf
        }
        Err(err) => {
            eprintln!("`git describe` err: {}", err);
            version
        }
    }
}

fn write_version() {
    let version = get_git_version();
    File::create(Path::new(&env::var("OUT_DIR").unwrap()).join("VERSION"))
        .unwrap()
        .write_all(version.trim().as_bytes())
        .unwrap();
}

fn get_cert() -> String {
    let mut content = String::new();
    let path = Path::new(
        &String::from_utf8(
            Command::new(env!("CARGO"))
                .arg("locate-project")
                .arg("--workspace")
                .arg("--message-format=plain")
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap(),
    )
    .parent()
    .unwrap()
    .join("certs")
    .join("shadow_ca.crt");

    File::open(path)
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();

    content
}

fn write_cert() {
    let cert = get_cert();

    File::create(Path::new(&env::var("OUT_DIR").unwrap()).join("CERT"))
        .unwrap()
        .write_all(cert.trim().as_bytes())
        .unwrap();
}

fn main() {
    write_version();

    write_cert();
}
