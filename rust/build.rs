use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::SystemTime,
};

use bindgen::Builder;

const RUNECORAL_BUILD_IMAGE: &str = "tinyverseml/runecoral-cross-debian-stretch";
fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn last_touched(path: impl AsRef<Path>) -> Result<SystemTime, io::Error> {
    let path = path.as_ref();
    let mut modified = path.metadata()?.modified()?;

    if path.is_dir() {
        for entry in path.read_dir()? {
            let entry = entry?;
            modified = std::cmp::min(modified, last_touched(entry.path())?);
        }
    }

    Ok(modified)
}

/// Use the `id` command to get certain information about the current user.
fn id(flag: &str) -> u64 {
    let output = Command::new("id")
        .arg(flag)
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout.trim().parse().unwrap()
}

fn make_librunecoral() -> PathBuf {
    let project_root = project_root();
    let bin = project_root.join("bazel-bin/runecoral/librunecoral.a");

    if bin.exists()
        && last_touched(&bin).unwrap() >= last_touched(project_root.join("runecoral")).unwrap()
    {
        // no need to recompile.
        return bin;
    }

    // Run the same build command from the README.

    let home = std::env::var("HOME").unwrap();
    let user = std::env::var("USER").unwrap();
    let user_id = id("-u");
    let group_id = id("-g");

    // FIXME: Parameterize the target CPU architecture
    let cpu = "k8";

    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("--rm")
        .arg(format!(
            "--volume={}:{}",
            project_root.display(),
            project_root.display()
        ))
        .arg(format!("--volume={}:{}", home, home))
        .arg("--volume=/etc/group:/etc/group:ro")
        .arg("--volume=/etc/passwd:/etc/passwd:ro")
        .arg("--volume=/etc/localtime:/etc/localtime:ro")
        .arg("--init")
        .arg(format!("--user={}:{}", user_id, group_id))
        .arg(format!("--env=HOME={}", home))
        .arg(format!("--env=USER={}", user))
        .arg(format!("--env=CPU={}", cpu))
        .arg(format!("--workdir={}", project_root.display()))
        .arg(RUNECORAL_BUILD_IMAGE)
        .arg("bash")
        .arg("-c")
        .arg("make");

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

    bin
}

fn main() {
    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    let header_file = manifest_dir.join("runecoral.h");

    make_librunecoral();

    println!("cargo:rustc-link-search={}", project_root().join("bazel-bin/runecoral/").display().to_string());
    println!("cargo:rustc-link-lib=runecoral");
    println!("cargo:rustc-flags=-l dylib=stdc++");

    let bindings = Builder::default()
        .header(header_file.display().to_string())
        .dynamic_link_require_all(true)
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
