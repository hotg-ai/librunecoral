use crate::ffi;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TensorDescriptor<'a> {
    pub element_type: ElementType,
    pub dimensions: &'a [ffi::size_t],
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[non_exhaustive]
pub enum ElementType {
    NoType = ffi::RuneCoralElementType__NoType as isize,
    Float32 = ffi::RuneCoralElementType__Float32 as isize,
    Int32 = ffi::RuneCoralElementType__Int32 as isize,
    UInt8 = ffi::RuneCoralElementType__UInt8 as isize,
    Int64 = ffi::RuneCoralElementType__Int64 as isize,
    String = ffi::RuneCoralElementType__String as isize,
    Bool = ffi::RuneCoralElementType__Bool as isize,
    Int16 = ffi::RuneCoralElementType__Int16 as isize,
    Complex64 = ffi::RuneCoralElementType__Complex64 as isize,
    Int8 = ffi::RuneCoralElementType__Int8 as isize,
    Float16 = ffi::RuneCoralElementType__Float16 as isize,
    Float64 = ffi::RuneCoralElementType__Float64 as isize,
    Complex128 = ffi::RuneCoralElementType__Complex128 as isize,
}

pub trait TensorElement: Sized {
    const ELEMENT_TYPE: ElementType;

    fn byte_buffer(slice: &[Self]) -> &[u8];
    fn byte_buffer_mut(slice: &mut [Self]) -> &mut [u8];
}

macro_rules! impl_tensor_element {
    ($($type:ty => $variant:expr,)* $(,)?) => {
        $(
            impl TensorElement  for $type {
                const ELEMENT_TYPE: ElementType = $variant;

                fn byte_buffer(slice: &[Self]) -> &[u8] {
                    let len = std::mem::size_of_val(slice);
                    unsafe {
                        std::slice::from_raw_parts(slice.as_ptr() as *const u8, len)
                    }
                }

                fn byte_buffer_mut(slice: &mut [Self]) -> &mut [u8] {
                    let len = std::mem::size_of_val(slice);
                    unsafe {
                        std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, len)
                    }
                }
            }
        )*
    };
}

impl_tensor_element! {
    u8 => ElementType::UInt8,
    i16 => ElementType::Int16,
    i32 => ElementType::Int32,
    i64 => ElementType::Int64,
    f32 => ElementType::Float32,
    f64 => ElementType::Float64,
}

impl From<ElementType> for ffi::RuneCoralElementType {
    fn from(e: ElementType) -> ffi::RuneCoralElementType {
        e as ffi::RuneCoralElementType
    }
}

/// An immutable reference to a tensor's backing buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct Tensor<'a> {
    pub element_type: ElementType,
    pub buffer: &'a [u8],
    pub shape: Vec<ffi::size_t>,
}

impl<'a> Tensor<'a> {
    pub(crate) fn as_coral_tensor(&self) -> ffi::RuneCoralTensor {
        ffi::RuneCoralTensor {
            type_: self.element_type.into(),
            data: self.buffer.as_ptr() as *mut _,
            shape: self.shape.as_ptr(),
            rank: self.shape.len() as ffi::size_t,
        }
    }

    pub fn from_slice<E: TensorElement>(slice: &'a [E], dimensions: &[usize]) -> Self {
        Tensor {
            element_type: E::ELEMENT_TYPE,
            buffer: E::byte_buffer(slice),
            shape: dimensions.iter().map(|&d| d as ffi::size_t).collect(),
        }
    }
}

/// A mutable reference to a tensor's backing buffer.
#[derive(Debug, PartialEq)]
pub struct TensorMut<'a> {
    pub element_type: ElementType,
    pub buffer: &'a mut [u8],
    pub shape: Vec<ffi::size_t>,
}

impl<'a> TensorMut<'a> {
    pub(crate) fn as_coral_tensor(&mut self) -> ffi::RuneCoralTensor {
        ffi::RuneCoralTensor {
            type_: self.element_type.into(),
            data: self.buffer.as_mut_ptr() as *mut _,
            shape: self.shape.as_ptr(),
            rank: self.shape.len() as ffi::size_t,
        }
    }

    pub fn from_slice<E: TensorElement>(slice: &'a mut [E], dimensions: &[usize]) -> Self {
        TensorMut {
            element_type: E::ELEMENT_TYPE,
            buffer: E::byte_buffer_mut(slice),
            shape: dimensions.iter().map(|&d| d as ffi::size_t).collect(),
        }
    }
}
