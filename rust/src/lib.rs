//! Bindings for exposing the functionality of `librunecoral`
//!
//! # Example
//!
//! ```rust,no_run
//! # fn load_model() -> &'static [u8] { todo!() }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hotg_runecoral::{Tensor, TensorMut, InferenceContext};
//!
//! // load the model
//! let model: &[u8] = load_model();
//!
//! // set aside some arrays for our inputs and outputs
//! let input = [0.0_f32];
//! let mut output = [0.0_f32];
//!
//! // And create tensors which point to them
//! let input_tensor = Tensor::from_slice(&input, &[1]);
//! let output_tensor = TensorMut::from_slice(&mut output, &[1]);
//!
//! // load our inference backend
//! let mut ctx = InferenceContext::create_context(
//!     "application/tflite-context",
//!     model,
//!     &[input_tensor.descriptor()],
//!     &[output_tensor.descriptor()],
//!     AccelerationBackend::None,
//! )?;
//!
//! // Now we can run inference
//! ctx.infer(&[input_tensor], &mut [output_tensor])?;
//!
//! // and look at the results
//! println!("{:?} => {:?}", input, output);
//! # Ok(())
//! # }
//! ```

mod context;
pub mod ffi;
mod tensors;

pub use crate::{
    context::{InferenceContext, LoadError, AccelerationBackend},
    tensors::{ElementType, Tensor, TensorDescriptor, TensorElement, TensorMut},
};

use std::ffi::NulError;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Invalid string")]
    InvalidString(#[from] NulError),
    #[error("Unable to load the model")]
    Load(#[from] LoadError),
}
