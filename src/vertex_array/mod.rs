use super::{gl, NonZero, NonZeroName};

/// Determines the number of components to load, generally this should match the
/// dimensionality of the vertex shader input.
///
/// For non-packed formats, this determines the number of `ty` typed items to read.
/// For [packed](PackedIntegerAttribute) formats, this must be [`Components::Vec4`].
#[repr(i32)]
pub enum Components {
    Scalar = 1,
    Vec2 = 2,
    Vec3 = 3,
    Vec4 = 4,
}
impl From<Components> for i32 {
    fn from(value: Components) -> Self {
        value as _
    }
}

/// One integer per component.
#[repr(u32)]
pub enum IntegerAttribute {
    U8 = gl::UNSIGNED_BYTE,
    I8 = gl::BYTE,
    U16 = gl::UNSIGNED_SHORT,
    I16 = gl::SHORT,
    U32 = gl::UNSIGNED_INT,
    I32 = gl::INT,
}
impl IntegerAttribute {
    /// Get the align requirements for fetching this attribute.
    pub fn align_of(&self) -> usize {
        match self {
            Self::U8 => std::mem::align_of::<u8>(),
            Self::I8 => std::mem::align_of::<i8>(),
            Self::U16 => std::mem::align_of::<u16>(),
            Self::I16 => std::mem::align_of::<i16>(),
            Self::U32 => std::mem::align_of::<u32>(),
            Self::I32 => std::mem::align_of::<i32>(),
        }
    }
}

// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for IntegerAttribute {}

/// One float per component.
#[repr(u32)]
pub enum FloatingAttribute {
    F16 = gl::HALF_FLOAT,
    F32 = gl::FLOAT,
    /// Fixed point `16.16` format.
    // (Is this in the right place? This placement means that this cannot be used with
    // `normalize = true` - It is hard to find documentation or mention of this type anywhere!)
    Fixed16_16 = gl::FIXED,
}
impl FloatingAttribute {
    /// Get the align requirements for fetching this attribute.
    pub fn align_of(&self) -> usize {
        match self {
            Self::F16 => std::mem::align_of::<u16>(),
            Self::F32 => std::mem::align_of::<f32>(),
            Self::Fixed16_16 => std::mem::align_of::<u32>(),
        }
    }
}

// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for FloatingAttribute {}

/// A Single element representing four packed components.
#[repr(u32)]
pub enum PackedIntegerAttribute {
    /// LSB -> MSB, `[i10, i10, i10, i2]` packed signed integers.
    /// The fourth component, `w`, is 2 bits.
    IReverse2_10_10_10 = gl::INT_2_10_10_10_REV,
    /// LSB -> MSB, `[u10, u10, u10, u2]` packed unsigned integers.
    /// The fourth component, `w`, is 2 bits.
    UReverse2_10_10_10 = gl::UNSIGNED_INT_2_10_10_10_REV,
}
impl PackedIntegerAttribute {
    /// Get the align requirements for fetching this attribute.
    pub fn align_of(&self) -> usize {
        match self {
            Self::IReverse2_10_10_10 | Self::UReverse2_10_10_10 => std::mem::align_of::<u32>(),
        }
    }
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for PackedIntegerAttribute {}

/// Specifies the type and interpretation of component data.
pub enum AttributeType {
    /// Fetch as integers, access in shader as integers.
    Integer(IntegerAttribute),
    /// Fetch as integers, access in shader directly casted to floats.
    /// E.g. `3` becomes `3.0`.
    Scaled(IntegerAttribute),
    /// Fetch as packed integers, access in shader directly casted to floats.
    /// E.g. `3` becomes `3.0`.
    PackedScaled(PackedIntegerAttribute),
    /// Fetch as integers, access in shader as normalized floats.
    /// `[0, 1]` for unsigned integer formats and `[-1, 1]` for signed integer formats.
    Normalized(IntegerAttribute),
    /// Fetch as packed integers, access in shader as normalized floats.
    /// `[0, 1]` for unsigned integer formats and `[-1, 1]` for signed integer formats.
    PackedNormalized(PackedIntegerAttribute),
    /// Fetch as floats, access in shader as floats.
    Float(FloatingAttribute),
}
impl AttributeType {
    /// Get the align requirements for fetching this attribute.
    pub fn align_of(&self) -> usize {
        match self {
            AttributeType::Float(ty) => ty.align_of(),

            AttributeType::Scaled(ty)
            | AttributeType::Integer(ty)
            | AttributeType::Normalized(ty) => ty.align_of(),

            AttributeType::PackedScaled(ty) | AttributeType::PackedNormalized(ty) => ty.align_of(),
        }
    }
}

/// Arguments to `glVertexAttrib[I]Pointer`.
pub struct Attribute {
    /// The type of data to fetch from the array, as well as it's interpretation
    /// within the shader interface.
    pub ty: AttributeType,
    /// The number of components of the scalar/vector.
    pub components: Components,
    /// The spacing in bytes between consecutive attribute values.
    /// `None` assumes the size based on [`Self::ty`] and [`Self::components`].
    ///
    /// This must be aligned with [`AttributeType::align_of`]. If `None`, that is trivially true.
    // Does this need to be aligned? Prolly! IDK??!?
    pub stride: Option<NonZero<usize>>,
    /// Offset, in bytes, from the beginning of the buffer where the first component is located.
    ///
    /// This must be aligned with [`AttributeType::align_of`].
    pub offset: usize,
}

/// VAO.
/// A vertex array remembers the state of buffers bound at the same time as it,
/// and provides offsets, sizes, and types for the attributes fetched by the
/// vertex shader.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks memory"]
pub struct VertexArray(pub(crate) NonZeroName);

impl crate::sealed::Sealed for VertexArray {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl crate::ThinGLObject for VertexArray {}
