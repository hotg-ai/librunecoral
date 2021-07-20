use crate::ffi;
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
