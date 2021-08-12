use crate::{ffi, Error, InferenceContext, TensorDescriptor};
use std::{
    ffi::{CString},
    mem::MaybeUninit,
    ptr::{self, NonNull}
};

pub fn create_inference_context(
    mimetype: &str,
    model: &[u8],
    inputs: &[TensorDescriptor<'_>],
    outputs: &[TensorDescriptor<'_>],
) -> Result<InferenceContext, Error> {
    let mimetype = CString::new(mimetype)?;
    let mut inference_context = MaybeUninit::uninit();

    let inputs = dummy_tensors(inputs);
    let outputs = dummy_tensors(outputs);

    // Safety: We've ensured our inputs are sane by construction (i.e. Rust
    // doesn't let you create a null slice and all enums are exhaustive)
    // and our `inputs` and `outputs` tensor vector can't outlive the
    // `inputs` and `outputs` function arguments.
    unsafe {
        let ret = ffi::create_inference_context(
            mimetype.as_ptr(),
            model.as_ptr().cast(),
            model.len() as ffi::size_t,
            inputs.as_ptr(),
            inputs.len() as ffi::size_t,
            outputs.as_ptr(),
            outputs.len() as ffi::size_t,
            inference_context.as_mut_ptr(),
        );
        check_load_result(ret)?;

        let inference_context = inference_context.assume_init();

        Ok(InferenceContext::new(
            NonNull::new(inference_context).expect("Should be initialized")
        ))
    }
}

fn check_load_result(return_code: ffi::RuneCoralLoadResult) -> Result<(), LoadError> {
    match return_code {
        ffi::RuneCoralLoadResult__Ok => Ok(()),
        ffi::RuneCoralLoadResult__IncorrectMimeType => Err(LoadError::IncorrectMimeType),
        ffi::RuneCoralLoadResult__IncorrectArgumentSizes => Err(LoadError::IncorrectArgumentSizes),
        ffi::RuneCoralLoadResult__IncorrectArgumentTypes => Err(LoadError::IncorrectArgumentTypes),
        ffi::RuneCoralLoadResult__InternalError => Err(LoadError::InternalError),
        _ => Err(LoadError::Other { return_code }),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, thiserror::Error)]
pub enum LoadError {
    #[error("Incorrect mimetype")]
    IncorrectMimeType,
    #[error("Incorrect argument types")]
    IncorrectArgumentTypes,
    #[error("Incorrect argument sizes")]
    IncorrectArgumentSizes,
    #[error("Internal error")]
    InternalError,
    #[error("Unknown error {}", return_code)]
    Other {
        return_code: ffi::RuneCoralLoadResult,
    },
}

fn dummy_tensors(inputs: &[TensorDescriptor<'_>]) -> Vec<ffi::RuneCoralTensor> {
    let mut tensors = Vec::new();

    for input in inputs {
        let tensor = ffi::RuneCoralTensor {
            type_: input.element_type as ffi::RuneCoralElementType,
            data: ptr::null_mut(),
            shape: input.shape.as_ptr(),
            rank: input.shape.len() as _,
        };
        tensors.push(tensor);
    }

    tensors
}