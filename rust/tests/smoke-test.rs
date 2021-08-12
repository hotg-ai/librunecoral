use hotg_runecoral::{ElementType, Error, LoadError, Tensor, TensorDescriptor, TensorMut};
use std::{
    borrow::Cow,
    ffi::CStr
};

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
#[ignore = "https://github.com/hotg-ai/librunecoral/issues/7"]
fn create_inference_context_with_invalid_model() {
    let model = b"this is not a valid model";

    let _ = hotg_runecoral::create_inference_context(mimetype(), model, &[], &[])
        .unwrap();
}

#[test]
fn create_inference_context_with_incorrect_number_of_tensors() {
    let model = include_bytes!("sinemodel.tflite");

    let err = hotg_runecoral::create_inference_context(mimetype(), model, &[], &[])
        .unwrap_err();

    assert_eq!(err, Error::Load(LoadError::IncorrectArgumentSizes));
}

#[test]
fn run_inference_using_the_sine_model() {
    let model = include_bytes!("sinemodel.tflite");
    let descriptors = [TensorDescriptor {
        element_type: ElementType::Float32,
        shape: Cow::Borrowed(&[1, 1]),
    }];

    let mut ctx = hotg_runecoral::create_inference_context(mimetype(), model, &descriptors, &descriptors)
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
