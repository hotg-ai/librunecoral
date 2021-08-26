use std::{path::{Path, PathBuf}, process::{Command, Output, Stdio}};

use bindgen::Builder;

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn librunecoral_path() -> PathBuf  {
    project_root()
        .join("dist")
        .join("lib")
        .join(std::env::var("CARGO_CFG_TARGET_OS").unwrap())
        .join(std::env::var("CARGO_CFG_TARGET_ARCH").unwrap())
}

fn make_librunecoral(target_os: &str) {
    // Run the same build command from the README.
    let target_arch = match std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_ref() {
        "aarch64" => "arm64".to_string(),
        _ => std::env::var("CARGO_CFG_TARGET_ARCH").unwrap()
    };

    let mut cmd = Command::new("make");

    cmd.current_dir(project_root());
    cmd.arg(format!("librunecoral-{}-{}", target_os, target_arch));

    cmd.stdin(Stdio::null())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let Output {
        stdout,
        stderr,
        status,
    } = cmd.output().unwrap();

    if !status.success() {
        let stdout = String::from_utf8_lossy(&stdout);
        if !stdout.trim().is_empty() {
            println!("Stdout:");
            println!("{}", stdout);
        }
        let stderr = String::from_utf8_lossy(&stderr);
        if !stderr.trim().is_empty() {
            println!("Stderr:");
            println!("{}", stderr);
        }
        panic!("{:?} failed", cmd);
    }
}

fn main() {
    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    let header_file = manifest_dir.join("runecoral.h");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    match  target_os.as_str() {
        "linux" => make_librunecoral(&target_os),
        "android" => make_librunecoral(&target_os),
        _ => panic!("Target OS not supported!")
    };

    println!("cargo:rustc-link-search={}", project_root().join(librunecoral_path()).display().to_string());
    println!("cargo:rustc-link-lib=runecoral");
    println!("cargo:rustc-flags=-l dylib=stdc++ -lEGL -lGLESv2");

    let bindings = Builder::default()
        .header(header_file.display().to_string())
        .derive_debug(true)
        .derive_copy(true)
        .derive_default(true)
        .prepend_enum_name(false)
        .rustfmt_bindings(true)
        .generate()
        .unwrap();

    let out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    let dest = out_dir.join("bindings.rs");

    bindings.write_to_file(dest).unwrap();
}
