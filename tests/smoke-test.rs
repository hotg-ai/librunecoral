use hotg_runecoral::{
    ElementType, Error, InferenceContext, LoadError, Tensor, TensorDescriptor, TensorMut,
};
use std::{borrow::Cow, ffi::CStr};

/// If this isn't provided, the library will be compiled from scratch using the
/// `docker` image.

fn mimetype() -> &'static str {
    unsafe {
        CStr::from_ptr(hotg_runecoral::ffi::RUNE_CORAL_MIME_TYPE__TFLITE)
            .to_str()
            .unwrap()
    }
}

#[test]
fn create_inference_context_with_invalid_model() {
    let model = b"this is not a valid model";

    let result = hotg_runecoral::InferenceContext::create_context(
        mimetype(),
        model,
        hotg_runecoral::AccelerationBackend::NONE,
    );

    assert_eq!(result.unwrap_err(), Error::Load(LoadError::InternalError));
}

#[test]
fn create_inference_context() {
    let model = include_bytes!("sinemodel.tflite");

    let descriptors = vec![TensorDescriptor {
        element_type: ElementType::Float32,
        shape: Cow::Borrowed(&[1, 1]),
    }];

    let context = hotg_runecoral::InferenceContext::create_context(
        mimetype(),
        model,
        hotg_runecoral::AccelerationBackend::NONE,
    )
    .unwrap();

    assert_eq!(context.inputs(), descriptors);
    assert_eq!(context.outputs(), descriptors);
    assert_eq!(context.opcount(), 3);
}

#[test]
fn run_inference_using_the_sine_model() {
    let model = include_bytes!("sinemodel.tflite");

    let mut ctx = hotg_runecoral::InferenceContext::create_context(
        mimetype(),
        model,
        hotg_runecoral::AccelerationBackend::NONE,
    )
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

#[test]
fn query_available_hardware_backends() {
    let backends = AccelerationBackend::currently_available();
    );
    println!(
        "test query_available_hardware_backends: supports gpu acceleration: {}",
        (backends & hotg_runecoral::AccelerationBackend::GPU
            == hotg_runecoral::AccelerationBackend::GPU)
    );
}
