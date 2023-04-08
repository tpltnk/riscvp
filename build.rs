use std::{
    fs,
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

use cc::Build;

use std::env;

const OUT_DIRX: &str = env!("OUT_DIRX");
const IN_DIR: &str = env!("IN_DIR");
const INC_DIR: &str = env!("INC_DIR");

fn main() -> ExitCode {
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
        .for_each(|maybe_entry| {
            let entry = maybe_entry.unwrap();
            let mut build = Build::new();
            if build.get_compiler().is_like_clang() {
                println!("cargo:warning=clang detected");
                build.flag("-E").flag("-P");
            }
            if build.get_compiler().is_like_gnu() {
                println!("cargo:warning=gcc detected");
                build.flag("-E").flag("-P");
            }
            if build.get_compiler().is_like_msvc() {
                println!("cargo:warning=msvc detected");
                build.flag("/E").flag("/P");
            }
            let compiler = build
                .include(INC_DIR)
                .define("RISCVP_AUTHORS", env!("CARGO_PKG_AUTHORS"))
                .get_compiler();
            fs::write(
                PathBuf::from(OUT_DIRX).join(entry.path().file_name().unwrap()),
                compiler
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
        });

    add_watchers();
    if cfg!(target_os = "windows") {
        println!("cargo:warning=qemu on windows not supported yet");
        ExitCode::FAILURE
    } else {
        build_qemu();
        ExitCode::SUCCESS
    }
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

fn build_qemu() {
    if !PathBuf::from("build").exists() {
        fs::create_dir("build").unwrap();
        env::set_current_dir("build").unwrap();
        println!("cargo:warning=configuring qemu");
        Command::new("../qemu/configure")
            .arg("--target-list=riscv64-softmmu")
            // .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("Failed to execute process");
        println!("cargo:warning=building qemu");
        Command::new("make")
            .arg("-j")
            .arg(format!("{}", num_cpus::get() / 2))
            // .stdout(Stdio::inherit())
            .stderr(Stdio::inherit()) // stderr available at target/<profile>/build/riscvp-<hash>/stderr
            .status()
            .expect("Failed to execute process");
    }
}
