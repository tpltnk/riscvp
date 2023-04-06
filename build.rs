use std::{fs, path::PathBuf};

use cc::Build;

const OUT_DIRX: &str = env!("OUT_DIRX");
const IN_DIR: &str = env!("IN_DIR");
const INC_DIR: &str = env!("INC_DIR");

fn main() {
    if PathBuf::from(OUT_DIRX).exists() {
        fs::remove_dir_all(OUT_DIRX).expect("failed to remove directory");
    }
    fs::create_dir(OUT_DIRX).expect("failed to create directory");
    fs::read_dir(IN_DIR)
        .expect("Cannot read IN_DIR")
        .filter(|maybe_entry| {
            let entry = maybe_entry.as_ref().unwrap();
            (entry.path().is_file() || entry.path().is_symlink())
                && entry
                    .path()
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with("S")
        })
        .map(|maybe_entry| {
            let entry = maybe_entry.unwrap();
            fs::write(
                PathBuf::from(OUT_DIRX).join(entry.path().file_name().unwrap()),
                Build::new()
                    .flag(if cfg!(target_os = "windows") {
                        "/E"
                    } else {
                        "-E"
                    })
                    .flag(if cfg!(target_os = "windows") {
                        "/C"
                    } else {
                        "-C"
                    })
                    .include(INC_DIR)
                    .define("RISCVP_AUTHORS", env!("CARGO_PKG_AUTHORS"))
                    .get_compiler()
                    .to_command()
                    .arg(entry.path())
                    .output()
                    .expect("Failed to execute process")
                    .stdout,
            )
            .unwrap();
            println!(
                "cargo:rerun-if-changed={}",
                PathBuf::from(IN_DIR)
                    .join(entry.path())
                    .display()
                    .to_string()
            );
        })
        .for_each(drop);

    add_watchers();
}

fn add_watchers() {
    fs::read_dir(INC_DIR)
        .unwrap()
        .filter(|maybe_entry| {
            let entry = maybe_entry.as_ref().unwrap();
            (entry.path().is_file() || entry.path().is_symlink())
                && entry
                    .path()
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with("h")
        })
        .for_each(|maybe_entry| {
            let entry = maybe_entry.unwrap();
            println!("cargo:rerun-if-changed={}", entry.path().display());
        });

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_AUTHORS");
    println!("cargo:rerun-if-env-changed=OUT_DIRX");
    println!("cargo:rerun-if-env-changed=IN_DIR");
    println!("cargo:rerun-if-env-changed=INC_DIR");
    println!("cargo:rustc-link-arg=-nostartfiles");
    println!("cargo:OUT_DIRX={OUT_DIRX}");
}
