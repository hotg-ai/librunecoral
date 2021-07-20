mod context;
mod ffi;
mod rune_coral;
mod tensors;

pub use crate::{
    context::InferenceContext,
    rune_coral::RuneCoral,
    tensors::{ElementType, TensorDescriptor},
};

use std::ffi::NulError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid string")]
    InvalidString(#[from] NulError),
}
