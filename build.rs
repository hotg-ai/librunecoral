use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use bindgen::Builder;

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/**
dist_dir is a directory containing the installation of librunecoral with a layout like:
$ tree dist
dist
├── include
│   └── runecoral.h
└── lib
    ├── android
    │   ├── arm
    │   │   └── librunecoral.a
    │   ├── aarch64
    │   │   └── librunecoral.a
    │   └── x86
    │       └── librunecoral.a
    └── linux
        ├── arm
        │   └── librunecoral.a
        ├── aarch64
        │   └── librunecoral.a
        └── x86_64
            └── librunecoral.a

When this directory is passed as an environment variable, we do not try to build the librunecoral and instead
use the precompiled libraries from this path.
*/
fn dist_dir() -> PathBuf {
    match std::env::var("RUNECORAL_DIST_DIR") {
        Ok(dir) => Path::new(&dir).to_path_buf(),
        _ => project_root()
            .join(std::env::var("OUT_DIR").unwrap())
            .join("dist"),
    }
}

fn bazel_cache_dir() -> PathBuf {
    project_root()
        .join(std::env::var("OUT_DIR").unwrap())
        .join("bazel-cache")
}

fn compilation_mode() -> String {
    String::from(match std::env::var("PROFILE").unwrap().as_str() {
        "release" => "opt",
        _ => "dbg"
    })
}

fn target_arch() -> String {
    std::env::var("CARGO_CFG_TARGET_ARCH").unwrap()
}

fn librunecoral_path() -> PathBuf {
    dist_dir()
        .join("lib")
        .join(std::env::var("CARGO_CFG_TARGET_OS").unwrap())
        .join(target_arch())
}

fn runecoral_h_path() -> PathBuf {
    dist_dir().join("include")
}

fn execute_cmd(mut cmd: Command) {
    let Output {
        stdout,
        stderr,
        status,
    } = cmd.output().unwrap();

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

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

fn make_runecoral_h() {
    fs::create_dir_all(runecoral_h_path()).unwrap();
    fs::copy(
        project_root().join("runecoral").join("runecoral.h"),
        runecoral_h_path().join("runecoral.h"),
    )
    .ok();
}

fn make_librunecoral(target_os: &str) {
    // Run the same build command from the README.
    let mut cmd = Command::new("make");

    cmd.current_dir(project_root());
    cmd.arg(format!("librunecoral-{}-{}", target_os, target_arch()))
        .arg(format!("PREFIX={}", std::env::var("OUT_DIR").unwrap()))
        .arg(format!("COMPILATION_MODE={}", compilation_mode()));

    cmd.arg(format!("BAZEL=bazel --batch --output_user_root={}", bazel_cache_dir().to_str().unwrap()));

    if cfg!(feature = "edgetpu_acceleration") {
        cmd.arg("EDGETPU_ACCELERATION=true");
    }
    if cfg!(feature = "gpu_acceleration") {
        cmd.arg("GPU_ACCELERATION=true");
    }

    execute_cmd(cmd)
}

fn make_librunecoral_windows() {
    // We are doing the job of make
    fs::create_dir_all(librunecoral_path()).unwrap();

    let mut cmd = Command::new("bazel");

    cmd.arg("--batch");

    cmd.arg("--output_user_root")
        .arg(bazel_cache_dir());

    cmd.arg("build")
        .arg("-c")
        .arg(compilation_mode())
        .arg("--config")
        .arg("windows")
        .arg("//runecoral:runecoral");

    if cfg!(feature = "edgetpu_acceleration") {
        cmd.arg("--define edgetpu_acceleration=true");
    }
    if cfg!(feature = "gpu_acceleration") {
        cmd.arg("--define gpu_acceleration=true");
    }

    cmd.current_dir(project_root());

    execute_cmd(cmd);

    if compilation_mode() == "dbg" {
        println!("cargo:rustc-link-lib=MSVCRTD");
    }

    fs::copy(
        project_root()
            .join("bazel-bin")
            .join("runecoral")
            .join("runecoral.lib"),
        librunecoral_path().join("runecoral.lib"),
    )
    .ok();
}

fn main() {
    let header_file = runecoral_h_path().join("runecoral.h");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if std::env::var("RUNECORAL_DIST_DIR").is_err() {
        match target_os.as_str() {
            "windows" => make_librunecoral_windows(),
            "linux" | "android" | "macos" | "ios" => make_librunecoral(&target_os),
            _ => panic!("Target OS not supported!"),
        };
        make_runecoral_h();
    }

    println!(
        "cargo:rustc-link-search={}",
        project_root()
            .join(librunecoral_path())
            .display()
            .to_string()
    );
    println!("cargo:rustc-link-lib=runecoral");
    if cfg!(feature = "gpu_acceleration") {
        println!("cargo:rustc-link-lib=EGL");
        println!("cargo:rustc-link-lib=GLESv2");
    }

    if target_os.as_str() == "linux" {
        println!("cargo:rustc-flags=-l dylib=stdc++");
    }

    if target_os.as_str() == "android" {
        println!("cargo:rustc-flags=-l dylib=c++");
    }

    if target_os.as_str() == "macos" || target_os.as_str() == "ios" {
        println!("cargo:rustc-flags=-l dylib=c++");
    }

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
