use crate::ffi;
use std::os::raw::c_int;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TensorDescriptor<'a> {
    pub element_type: ffi::RuneCoralElementType,
    pub dimensions: &'a [c_int],
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
