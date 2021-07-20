mod context;
pub mod ffi;
mod rune_coral;
mod tensors;

pub use crate::{
    context::InferenceContext,
    rune_coral::{LoadError, RuneCoral},
    tensors::{ElementType, TensorDescriptor},
};

use std::ffi::NulError;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Invalid string")]
    InvalidString(#[from] NulError),
    #[error("Unable to load the model")]
    Load(#[from] LoadError),
}
