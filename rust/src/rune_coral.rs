use crate::{ffi, Error, InferenceContext, TensorDescriptor};
use std::{
    ffi::{CString, OsStr},
    mem::MaybeUninit,
    ptr::{self, NonNull},
    rc::Rc,
};

/// A safe wrapper around `librunecoral` which has been dynamically loaded at
/// runtime.
pub struct RuneCoral {
    inner: Rc<ffi::RuneCoral>,
}

impl RuneCoral {
    pub fn load(path: impl AsRef<OsStr>) -> Result<Self, libloading::Error> {
        unsafe {
            let inner = ffi::RuneCoral::new(path)?;
            Ok(RuneCoral {
                inner: Rc::new(inner),
            })
        }
    }

    pub fn create_inference_context(
        &self,
        mimetype: &str,
        model: &[u8],
        inputs: &[TensorDescriptor],
        outputs: &[TensorDescriptor],
    ) -> Result<InferenceContext, Error> {
        let mimetype = CString::new(mimetype)?;
        let mut inference_context = MaybeUninit::uninit();

        let inputs = dummy_tensors(inputs);
        let outputs = dummy_tensors(outputs);

        let inference_context = unsafe {
            self.inner.create_inference_context(
                mimetype.as_ptr(),
                model.as_ptr().cast(),
                model.len() as ffi::size_t,
                inputs.as_ptr(),
                inputs.len() as ffi::size_t,
                outputs.as_ptr(),
                outputs.len() as ffi::size_t,
                inference_context.as_mut_ptr(),
            );

            inference_context.assume_init()
        };

        Ok(InferenceContext::new(
            NonNull::new(inference_context).expect("Should be initialized"),
            Rc::clone(&self.inner),
        ))
    }
}

fn dummy_tensors(inputs: &[TensorDescriptor<'_>]) -> Vec<ffi::RuneCoralTensor> {
    let mut tensors = Vec::new();

    for input in inputs {
        let tensor = ffi::RuneCoralTensor {
            type_: input.element_type as ffi::RuneCoralElementType,
            data: ptr::null_mut(),
            shape: input.dimensions.as_ptr(),
            rank: input.dimensions.len() as _,
        };
        tensors.push(tensor);
    }

    tensors
}
