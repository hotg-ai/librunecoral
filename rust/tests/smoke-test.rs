use once_cell::sync::Lazy;
use runecoral::RuneCoral;
use std::{
    ffi::CStr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

static LIBRUNECORAL: Lazy<PathBuf> = Lazy::new(librunecoral);

#[test]
fn load_librunecoral_from_disk() {
    let _ = RuneCoral::load(&*LIBRUNECORAL).unwrap();
}

#[test]
fn create_inference_context() {
    let rune_coral = RuneCoral::load(&*LIBRUNECORAL).unwrap();
    let mimetype = CStr::from_bytes_with_nul(runecoral::ffi::RUNE_CORAL_MIME_TYPE__TFLITE)
        .unwrap()
        .to_str()
        .unwrap();

    rune_coral
        .create_inference_context(mimetype, &[], &[], &[])
        .unwrap();
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn librunecoral() -> PathBuf {
    let project_root = project_root();
    let bin = project_root.join("bazel-bin/runecoral/librunecoral.so");

    if !bin.exists() {
        let home = std::env::var("HOME").unwrap();
        let user = std::env::var("USER").unwrap();
        let user_id = id("-u");
        let group_id = id("-g");
        let cpu = "k8";

        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("--rm")
            .arg("-it")
            .arg(format!(
                "--volume={}:{}",
                project_root.display(),
                project_root.display()
            ))
            .arg(format!("--volume={}:{}", home, home))
            .arg("--volume=/etc/group:/etc/group:ro")
            .arg("--volume=/etc/passwd:/etc/passwd:ro")
            .arg("--volume=/etc/localtime:/etc/localtime:ro")
            .arg(format!("--user={}:{}", user_id, group_id))
            .arg(format!("--env=HOME={}", home))
            .arg(format!("--env=USER={}", user))
            .arg(format!("--env=CPU={}", cpu))
            .arg(format!("--workdir={}", project_root.display()))
            .arg("runecoral-cross-debian-stretch")
            .arg("make");
    }

    bin
}

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
