use crate::{ffi, Tensor, TensorMut};
use std::{
    fmt::{self, Debug, Formatter},
    ptr::NonNull,
    rc::Rc,
};

/// A backend which can run inference on a model.
pub struct InferenceContext {
    ctx: NonNull<ffi::RuneCoralContext>,
    lib: Rc<ffi::RuneCoral>,
}

impl InferenceContext {
    /// Create a new [`InferenceContext`].
    ///
    /// # Safety
    ///
    /// This takes ownership of the `ctx` pointer and will deallocate it on
    /// drop.
    pub(crate) unsafe fn new(ctx: NonNull<ffi::RuneCoralContext>, lib: Rc<ffi::RuneCoral>) -> Self {
        InferenceContext { ctx, lib }
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

            let ret = self.lib.infer(
                self.ctx.as_ptr(),
                inputs.as_ptr() as *mut _,
                inputs.len() as ffi::size_t,
                outputs.as_mut_ptr(),
                outputs.len() as ffi::size_t,
            );

            check_inference_error(ret)
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
            self.lib.destroy_inference_context(self.ctx.as_ptr());
        }
    }
}

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
