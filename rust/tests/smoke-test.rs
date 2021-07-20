use once_cell::sync::Lazy;
use runecoral::{ElementType, Error, LoadError, RuneCoral, Tensor, TensorDescriptor, TensorMut};
use std::{
    ffi::CStr,
    io,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::SystemTime,
};

const RUNECORAL_BUILD_IMAGE: &str = "tinyverseml/runecoral-cross-debian-stretch";
static LIBRUNECORAL: Lazy<PathBuf> = Lazy::new(librunecoral);

fn mimetype() -> &'static str {
    CStr::from_bytes_with_nul(runecoral::ffi::RUNE_CORAL_MIME_TYPE__TFLITE)
        .unwrap()
        .to_str()
        .unwrap()
}

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

#[test]
fn run_inference_using_the_sine_model() {
    let rune_coral = RuneCoral::load(&*LIBRUNECORAL).unwrap();
    let model = include_bytes!("sinemodel.tflite");
    let descriptors = [TensorDescriptor {
        element_type: ElementType::Float32,
        dimensions: &[1, 1],
    }];

    let mut ctx = rune_coral
        .create_inference_context(mimetype(), model, &descriptors, &descriptors)
        .unwrap();

    let input = [0.5_f32];
    let mut output = [0_f32];

    ctx.infer(
        &[Tensor::from_slice(&input, &[1])],
        &mut [TensorMut::from_slice(&mut output, &[1])],
    )
    .unwrap();

    assert_eq!(round(output[0]), round(0.4540305));
}

fn round(n: f32) -> f32 {
    (n * 10000.0).round() / 10000.0
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn librunecoral() -> PathBuf {
    // Let the user override which `librunecoral.so` we load.
    if let Ok(path) = std::env::var("LIBRUNECORAL_SO") {
        let path = PathBuf::from(path);
        assert!(path.exists(), "Nothing found at \"{}\"", path.display());
        return path;
    }

    let project_root = project_root();
    let bin = project_root.join("bazel-bin/runecoral/librunecoral.so");

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
