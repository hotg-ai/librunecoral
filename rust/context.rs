use crate::{ffi, Error, Tensor, TensorDescriptor, TensorMut};
use bitflags::bitflags;
use std::{
    convert::TryInto,
    ffi::CString,
    fmt::{self, Debug, Formatter},
    mem::MaybeUninit,
    ptr::NonNull,
};

/// A backend which can run inference on a model.
pub struct InferenceContext {
    ctx: NonNull<ffi::RuneCoralContext>,
}

impl InferenceContext {
    /// Create a new [`InferenceContext`].
    ///
    /// # Safety
    ///
    /// This takes ownership of the `ctx` pointer and will deallocate it on
    /// drop.
    pub(crate) unsafe fn new(ctx: NonNull<ffi::RuneCoralContext>) -> Self {
        InferenceContext { ctx }
    }

    pub fn infer(
        &mut self,
        inputs: &[Tensor<'_>],
        outputs: &mut [TensorMut<'_>],
    ) -> Result<(), InferError> {
        // Safety: We are effectively casting a &T to a *mut T here. This is
        // okay, but only as long as the infer() function doesn't mutate the
        // input tensors in any way (casting from *mut T to &mut T would still
        // be UB, though).
        unsafe {
            let inputs: Vec<_> = inputs.iter().map(|t| t.as_coral_tensor()).collect();
            let mut outputs: Vec<_> = outputs.iter_mut().map(|t| t.as_coral_tensor()).collect();

            let ret = ffi::infer(
                self.ctx.as_ptr(),
                inputs.as_ptr() as *mut _,
                inputs.len() as ffi::size_t,
                outputs.as_mut_ptr(),
                outputs.len() as ffi::size_t,
            );

            check_inference_error(ret)
        }
    }

    pub fn create_context(
        mimetype: &str,
        model: &[u8],
        acceleration_backend: AccelerationBackend,
    ) -> Result<InferenceContext, Error> {
        let mimetype = CString::new(mimetype)?;
        let mut inference_context = MaybeUninit::uninit();

        // Safety: We've ensured our inputs are sane by construction (i.e. Rust
        // doesn't let you create a null slice and all enums are exhaustive)
        // and our `inputs` and `outputs` tensor vector can't outlive the
        // `inputs` and `outputs` function arguments.
        unsafe {
            let ret = ffi::create_inference_context(
                mimetype.as_ptr(),
                model.as_ptr().cast(),
                model.len() as ffi::size_t,
                (acceleration_backend.bits() as i32).try_into().unwrap(),
                inference_context.as_mut_ptr(),
            );

            check_load_result(ret)?;

            let inference_context = inference_context.assume_init();

            Ok(InferenceContext::new(
                NonNull::new(inference_context).expect("Should be initialized"),
            ))
        }
    }

    pub fn opcount(&self) -> u64 {
        unsafe { ffi::inference_opcount(self.ctx.as_ptr()) }
    }

    pub fn inputs(&self) -> Vec<TensorDescriptor> {
        unsafe {
            let mut input_tensors = MaybeUninit::uninit();
            let input_count = ffi::inference_inputs(self.ctx.as_ptr(), input_tensors.as_mut_ptr());

            (0..input_count)
                .map(|i| {
                    TensorDescriptor::from_rune_coral_tensor(
                        &*((*input_tensors.as_ptr()).offset(i as isize)),
                    )
                })
                .collect()
        }
    }

    pub fn outputs(&self) -> Vec<TensorDescriptor> {
        unsafe {
            let mut outputs = MaybeUninit::uninit();
            let output_count = ffi::inference_outputs(self.ctx.as_ptr(), outputs.as_mut_ptr());
            (0..output_count)
                .map(|i| {
                    TensorDescriptor::from_rune_coral_tensor(
                        &*((*outputs.as_ptr()).offset(i as isize)),
                    )
                })
                .collect()
        }
    }
}

impl Debug for InferenceContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InferenceContext").finish_non_exhaustive()
    }
}

impl Drop for InferenceContext {
    fn drop(&mut self) {
        unsafe {
            ffi::destroy_inference_context(self.ctx.as_ptr());
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, thiserror::Error)]
pub enum LoadError {
    #[error("Incorrect mimetype")]
    IncorrectMimeType,
    #[error("Internal error")]
    InternalError,
    #[error("Unknown error {}", return_code)]
    Other {
        return_code: ffi::RuneCoralLoadResult,
    },
}

fn check_load_result(return_code: ffi::RuneCoralLoadResult) -> Result<(), LoadError> {
    match return_code {
        ffi::RuneCoralLoadResult__Ok => Ok(()),
        ffi::RuneCoralLoadResult__IncorrectMimeType => Err(LoadError::IncorrectMimeType),
        ffi::RuneCoralLoadResult__InternalError => Err(LoadError::InternalError),
        _ => Err(LoadError::Other { return_code }),
    }
}

// Safety: There shouldn't be any thread-specific state, so it's okay to move
// the inference context to another thread.
//
// The inference context is very much **not** thread-safe though, so we can't
// implement Sync.
unsafe impl Send for InferenceContext {}

fn check_inference_error(return_code: ffi::RuneCoralInferenceResult) -> Result<(), InferError> {
    match return_code {
        ffi::RuneCoralInferenceResult__Ok => Ok(()),
        ffi::RuneCoralInferenceResult__Error => Err(InferError::InterpreterError),
        ffi::RuneCoralInferenceResult__DelegateError => Err(InferError::DelegateError),
        ffi::RuneCoralInferenceResult__ApplicationError => Err(InferError::ApplicationError),
        _ => Err(InferError::Other { return_code }),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, thiserror::Error)]
pub enum InferError {
    /// Generally referring to an error in the runtime (i.e. interpreter).
    #[error("The TensorFlow Lite interpreter encountered an error")]
    InterpreterError,
    /// Generally referring to an error from a TfLiteDelegate itself.
    #[error("A delegate returned an error")]
    DelegateError,
    // Generally referring to an error in applying a delegate due to
    // incompatibility between runtime and delegate, e.g., this error is returned
    // when trying to apply a TfLite delegate onto a model graph that's already
    // immutable.
    #[error("Invalid model graph or incompatibility between runtime and delegates")]
    ApplicationError,
    #[error("Unknown inference error {}", return_code)]
    Other {
        return_code: ffi::RuneCoralInferenceResult,
    },
}

bitflags! {
    pub struct AccelerationBackend: u32 {
        const NONE = ffi::RuneCoralAccelerationBackend__None as u32;
        const EDGETPU = ffi::RuneCoralAccelerationBackend__Edgetpu as u32;
        const GPU = ffi::RuneCoralAccelerationBackend__Gpu as u32;
    }
}

impl AccelerationBackend {
    /// Get all [`AccelerationBackend`]s that are available on this device.
    pub fn currently_available() -> Self {
        unsafe {
            AccelerationBackend::from_bits(ffi::availableAccelerationBackends() as u32).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn inference_context_is_only_send() {
        static_assertions::assert_impl_all!(InferenceContext: Send);
        static_assertions::assert_not_impl_any!(InferenceContext: Sync);
        // but we can wrap it in a mutex!
        static_assertions::assert_impl_all!(Mutex<InferenceContext>: Send, Sync);
    }
}
