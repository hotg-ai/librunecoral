use crate::ffi;
use std::{ptr::NonNull, rc::Rc};

pub struct InferenceContext {
    ctx: NonNull<ffi::RuneCoralContext>,
    lib: Rc<ffi::RuneCoral>,
}

impl InferenceContext {
    pub(crate) fn new(ctx: NonNull<ffi::RuneCoralContext>, lib: Rc<ffi::RuneCoral>) -> Self {
        InferenceContext { ctx, lib }
    }
}

impl Drop for InferenceContext {
    fn drop(&mut self) {
        unsafe {
            let mut ptr = self.ctx.as_ptr();
            self.lib.destroy_inference_context(&mut ptr);
        }
    }
}
