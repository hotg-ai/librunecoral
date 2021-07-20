use once_cell::sync::Lazy;
use runecoral::{Error, LoadError, RuneCoral};
use std::{
    ffi::CStr,
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::SystemTime,
};

static LIBRUNECORAL: Lazy<PathBuf> = Lazy::new(librunecoral);

#[test]
fn load_librunecoral_from_disk() {
    let _ = RuneCoral::load(&*LIBRUNECORAL).unwrap();
}

#[test]
#[ignore = "https://github.com/hotg-ai/librunecoral/issues/7"]
fn create_inference_context_with_invalid_model() {
    let rune_coral = RuneCoral::load(&*LIBRUNECORAL).unwrap();
    let model = b"this is not a valid model";

    let _ = rune_coral
        .create_inference_context(mimetype(), model, &[], &[])
        .unwrap();
}

#[test]
fn create_inference_context_with_incorrect_number_of_tensors() {
    let rune_coral = RuneCoral::load(&*LIBRUNECORAL).unwrap();
    let model = include_bytes!("sinemodel.tflite");

    let err = rune_coral
        .create_inference_context(mimetype(), model, &[], &[])
        .unwrap_err();

    assert_eq!(err, Error::Load(LoadError::IncorrectArgumentSizes));
}

fn mimetype() -> &'static str {
    CStr::from_bytes_with_nul(runecoral::ffi::RUNE_CORAL_MIME_TYPE__TFLITE)
        .unwrap()
        .to_str()
        .unwrap()
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

    let previous_compilation_time = bin.metadata().unwrap().modified().unwrap();

    if previous_compilation_time >= last_touched(project_root.join("runecoral")).unwrap() {
        // no need to recompile.
        return bin;
    }

    // Run the same build command from

    let home = std::env::var("HOME").unwrap();
    let user = std::env::var("USER").unwrap();
    let user_id = id("-u");
    let group_id = id("-g");
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
        .arg(format!("--user={}:{}", user_id, group_id))
        .arg(format!("--env=HOME={}", home))
        .arg(format!("--env=USER={}", user))
        .arg(format!("--env=CPU={}", cpu))
        .arg(format!("--workdir={}", project_root.display()))
        .arg("runecoral-cross-debian-stretch")
        .arg("bash")
        .arg("-c")
        .arg("make && bazel shutdown");

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().unwrap();

    if !output.status.success() {
        panic!("{:?} failed with {:?}", cmd, output);
    }

    bin
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
