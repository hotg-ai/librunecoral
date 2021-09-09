use hotg_runecoral::{
    mimetype, AccelerationBackend, ElementType, Error, InferenceContext, LoadError, Tensor,
    TensorDescriptor, TensorMut,
};
use std::borrow::Cow;

#[test]
fn create_inference_context_with_invalid_model() {
    let model = b"this is not a valid model";

    let result = InferenceContext::create_context(mimetype(), model, AccelerationBackend::NONE);

    assert_eq!(result.unwrap_err(), Error::Load(LoadError::InternalError));
}

#[test]
fn create_inference_context() {
    let model = include_bytes!("sinemodel.tflite");

    let descriptors = vec![TensorDescriptor {
        element_type: ElementType::Float32,
        shape: Cow::Borrowed(&[1, 1]),
    }];

    let context =
        InferenceContext::create_context(mimetype(), model, AccelerationBackend::NONE).unwrap();

    assert_eq!(context.inputs().collect::<Vec<_>>(), descriptors);
    assert_eq!(context.outputs().collect::<Vec<_>>(), descriptors);
    assert_eq!(context.opcount(), 3);
}

#[test]
fn run_inference_using_the_sine_model() {
    let mut model = include_bytes!("sinemodel.tflite").to_vec();

    let mut ctx =
        InferenceContext::create_context(mimetype(), &model, AccelerationBackend::NONE).unwrap();

    // Note: If the inference context held a reference to our model, this would
    // trigger a use-after-free.
    model.fill(0xAA);
    drop(model);

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

    println!(
        "test query_available_hardware_backends: supports edgetpu acceleration: {}",
        backends.contains(AccelerationBackend::EDGETPU)
    );
    println!(
        "test query_available_hardware_backends: supports gpu acceleration: {}",
        backends.contains(AccelerationBackend::GPU),
    );
}
