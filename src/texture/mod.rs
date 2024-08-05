//! Owned textures and their properties.
use super::{gl, GLenum, NonZero, NonZeroName};

/* /// The size and dimensionality of an image.
enum Dimensionality {
    D2{
        width: NonZero<u32>,
        height: NonZero<u32>,
        /// Invariant: <= floor(log2(max(width, height))) + 1
        levels: NonZero<u32>,
    },
    D3 {
        width: NonZero<u32>,
        height: NonZero<u32>,
        depth: NonZero<u32>,
        /// Invariant: <= floor(log2(max(width, height, depth))) + 1
        levels: NonZero<u32>,
    },
    D2Array {
        width: NonZero<u32>,
        height: NonZero<u32>,
        layers: NonZero<u32>,
        /// Invariant: <= floor(log2(max(width, height))) + 1
        levels: NonZero<u32>,
    },
    Cube {
        /// Square size of a cubemap face
        size: NonZero<u32>,
        /// Invariant: <= floor(log2(max(width))) + 1
        levels: NonZero<u32>,
    },
}*/

/// # Safety
/// TARGET must be one of GL_TEXTURE_{2D, 3D, 2D_ARRAY, CUBE_MAP}
pub unsafe trait Dimensionality: crate::sealed::Sealed {
    const TARGET: GLenum;
}
pub struct D2;
impl crate::sealed::Sealed for D2 {}
unsafe impl Dimensionality for D2 {
    const TARGET: GLenum = gl::TEXTURE_2D;
}
pub struct D3;
impl crate::sealed::Sealed for D3 {}
unsafe impl Dimensionality for D3 {
    const TARGET: GLenum = gl::TEXTURE_3D;
}
pub struct D2Array;
impl crate::sealed::Sealed for D2Array {}
unsafe impl Dimensionality for D2Array {
    const TARGET: GLenum = gl::TEXTURE_2D_ARRAY;
}
pub struct Cube;
impl crate::sealed::Sealed for Cube {}
unsafe impl Dimensionality for Cube {
    const TARGET: GLenum = gl::TEXTURE_CUBE_MAP;
}

#[repr(u32)]
pub enum InternalFormat {
    // Unsized color formats, i.e. the GL is allowed to chose any size it pleases.
    RGB = gl::RGB,
    RGBA = gl::RGBA,
    LuminanceAlpha = gl::LUMINANCE_ALPHA,
    Luminance = gl::LUMINANCE,
    Alpha = gl::ALPHA,

    // Sized color formats
    R8 = gl::R8,
    R8Snorm = gl::R8_SNORM,
    R16f = gl::R16F,
    R32f = gl::R32F,
    R8ui = gl::R8UI,
    R8i = gl::R8I,
    R16ui = gl::R16UI,
    R16i = gl::R16I,
    R32ui = gl::R32UI,
    R32i = gl::R32I,
    Rg8 = gl::RG8,
    Rg8Snorm = gl::RG8_SNORM,
    Rg16f = gl::RG16F,
    Rg32f = gl::RG32F,
    Rg8ui = gl::RG8UI,
    Rg8i = gl::RG8I,
    Rg16ui = gl::RG16UI,
    Rg16i = gl::RG16I,
    Rg32ui = gl::RG32UI,
    Rg32i = gl::RG32I,
    Rgb8 = gl::RGB8,
    Srgb8 = gl::SRGB8,
    Rgb565 = gl::RGB565,
    Rgb8Snorm = gl::RGB8_SNORM,
    R11fG11fB10f = gl::R11F_G11F_B10F,
    Rgb9E5 = gl::RGB9_E5,
    Rgb16f = gl::RGB16F,
    Rgb32f = gl::RGB32F,
    Rgb8ui = gl::RGB8UI,
    Rgb8i = gl::RGB8I,
    Rgb16ui = gl::RGB16UI,
    Rgb16i = gl::RGB16I,
    Rgb32ui = gl::RGB32UI,
    Rgb32i = gl::RGB32I,
    Rgba8 = gl::RGBA8,
    Srgb8Alpha8 = gl::SRGB8_ALPHA8,
    Rgba8Snorm = gl::RGBA8_SNORM,
    Rgb5A1 = gl::RGB5_A1,
    Rgba4 = gl::RGBA4,
    Rgb10A2 = gl::RGB10_A2,
    Rgba16f = gl::RGBA16F,
    Rgba32f = gl::RGBA32F,
    Rgba8ui = gl::RGBA8UI,
    Rgba8i = gl::RGBA8I,
    Rgb10A2ui = gl::RGB10_A2UI,
    Rgba16ui = gl::RGBA16UI,
    Rgba16i = gl::RGBA16I,
    Rgba32i = gl::RGBA32I,
    Rgba32ui = gl::RGBA32UI,

    // Sized depth / stencil
    DepthComponent16 = gl::DEPTH_COMPONENT16,
    DepthComponent24 = gl::DEPTH_COMPONENT24,
    DepthComponent32f = gl::DEPTH_COMPONENT32F,
    Depth24Stencil8 = gl::DEPTH24_STENCIL8,
    Depth32fStencil8 = gl::DEPTH32F_STENCIL8,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for InternalFormat {}
impl InternalFormat {
    /// Get the "format" GLenum associated with this internal format.
    /// This describes the layout of pixel data in a buffer.
    ///
    /// This is *not* the same as `self.as_gl`
    pub fn format(&self) -> Format {
        match self {
            Self::RGB => Format::RGB,
            Self::RGBA => Format::RGBA,
            Self::LuminanceAlpha => Format::LuminanceAlpha,
            Self::Luminance => Format::Luminance,
            Self::Alpha => Format::Alpha,

            Self::R8 => Format::Red,
            Self::R8Snorm => Format::Red,
            Self::R16f => Format::Red,
            Self::R32f => Format::Red,
            Self::R8ui => Format::RedInteger,
            Self::R8i => Format::RedInteger,
            Self::R16ui => Format::RedInteger,
            Self::R16i => Format::RedInteger,
            Self::R32ui => Format::RedInteger,
            Self::R32i => Format::RedInteger,

            Self::Rg8 => Format::RG,
            Self::Rg8Snorm => Format::RG,
            Self::Rg16f => Format::RG,
            Self::Rg32f => Format::RG,
            Self::Rg8ui => Format::RGInteger,
            Self::Rg8i => Format::RGInteger,
            Self::Rg16ui => Format::RGInteger,
            Self::Rg16i => Format::RGInteger,
            Self::Rg32ui => Format::RGInteger,
            Self::Rg32i => Format::RGInteger,

            Self::Rgb8 => Format::RGB,
            Self::Srgb8 => Format::RGB,
            Self::Rgb565 => Format::RGB,
            Self::Rgb8Snorm => Format::RGB,
            Self::R11fG11fB10f => Format::RGB,
            Self::Rgb9E5 => Format::RGB,
            Self::Rgb16f => Format::RGB,
            Self::Rgb32f => Format::RGB,
            Self::Rgb8ui => Format::RGBInteger,
            Self::Rgb8i => Format::RGBInteger,
            Self::Rgb16ui => Format::RGBInteger,
            Self::Rgb16i => Format::RGBInteger,
            Self::Rgb32ui => Format::RGBInteger,
            Self::Rgb32i => Format::RGBInteger,

            Self::Rgba8 => Format::RGBA,
            Self::Srgb8Alpha8 => Format::RGBA,
            Self::Rgba8Snorm => Format::RGBA,
            Self::Rgb5A1 => Format::RGBA,
            Self::Rgba4 => Format::RGBA,
            Self::Rgb10A2 => Format::RGBA,
            Self::Rgba16f => Format::RGBA,
            Self::Rgba32f => Format::RGBA,
            Self::Rgba8ui => Format::RGBAInteger,
            Self::Rgba8i => Format::RGBAInteger,
            Self::Rgb10A2ui => Format::RGBAInteger,
            Self::Rgba16ui => Format::RGBAInteger,
            Self::Rgba16i => Format::RGBAInteger,
            Self::Rgba32i => Format::RGBAInteger,
            Self::Rgba32ui => Format::RGBAInteger,

            Self::DepthComponent16 => Format::DepthComponent,
            Self::DepthComponent24 => Format::DepthComponent,
            Self::DepthComponent32f => Format::DepthComponent,
            Self::Depth24Stencil8 => Format::DepthStencil,
            Self::Depth32fStencil8 => Format::DepthStencil,
        }
    }
}

#[repr(u32)]
pub enum Format {
    Alpha = gl::ALPHA,
    Luminance = gl::LUMINANCE,
    LuminanceAlpha = gl::LUMINANCE_ALPHA,

    Red = gl::RED,
    RedInteger = gl::RED_INTEGER,
    RG = gl::RG,
    RGInteger = gl::RG_INTEGER,
    RGB = gl::RGB,
    RGBInteger = gl::RGB_INTEGER,
    RGBA = gl::RGBA,
    RGBAInteger = gl::RGBA_INTEGER,

    DepthComponent = gl::DEPTH_COMPONENT,
    DepthStencil = gl::DEPTH_STENCIL,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for Format {}

#[repr(u32)]
pub enum ImageData<'data> {
    U8(&'data [u8]) = gl::UNSIGNED_BYTE,
    I8(&'data [i8]) = gl::BYTE,
    U16(&'data [u16]) = gl::UNSIGNED_SHORT,
    I16(&'data [i16]) = gl::SHORT,
    U32(&'data [u32]) = gl::UNSIGNED_INT,
    I32(&'data [i32]) = gl::INT,
    F16(&'data [u16]) = gl::HALF_FLOAT,
    F32(&'data [f32]) = gl::FLOAT,
    Packed5_6_5(&'data [u16]) = gl::UNSIGNED_SHORT_5_6_5,
    Packed4_4_4_4(&'data [u16]) = gl::UNSIGNED_SHORT_4_4_4_4,
    Packed5_5_5_1(&'data [u16]) = gl::UNSIGNED_SHORT_5_5_5_1,
    Reverse2_10_10_10(&'data [u32]) = gl::UNSIGNED_INT_2_10_10_10_REV,
    Reverse10F11F11F(&'data [u32]) = gl::UNSIGNED_INT_10F_11F_11F_REV,
    Reverse5_9_9_9(&'data [u32]) = gl::UNSIGNED_INT_5_9_9_9_REV,
    Packed24_8(&'data [u32]) = gl::UNSIGNED_INT_24_8,
    F32Reverse24_8(&'data [F32Reverse24_8]) = gl::FLOAT_32_UNSIGNED_INT_24_8_REV,
}

// A unique type is needed here (as opposed to u64) because the GL treats this as two individual values, so the
// endian would be all messed up if u64 was used.
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct F32Reverse24_8 {
    /// First component, floating point.
    pub float: f32,
    /// Second (8 *least* significant bits) and third (24 most significant bits) components.
    pub int: u32,
}

// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for ImageData<'_> {}

impl ImageData<'_> {
    pub fn compatible_with_internal_format(&self, format: InternalFormat) -> bool {
        // Implement big table seen at https://registry.khronos.org/OpenGL-Refpages/es3.0/
        match format {
            InternalFormat::RGB => matches!(self, Self::U8(_) | Self::Packed5_6_5(_)),
            InternalFormat::RGBA => matches!(
                self,
                Self::U8(_) | Self::Packed4_4_4_4(_) | Self::Packed5_5_5_1(_)
            ),
            InternalFormat::LuminanceAlpha => matches!(self, Self::U8(_)),
            InternalFormat::Luminance => matches!(self, Self::U8(_)),
            InternalFormat::Alpha => matches!(self, Self::U8(_)),

            InternalFormat::R8 => matches!(self, Self::U8(_)),
            InternalFormat::R8Snorm => matches!(self, Self::I8(_)),
            InternalFormat::R16f => matches!(self, Self::F16(_) | Self::F32(_)),
            InternalFormat::R32f => matches!(self, Self::F32(_)),
            InternalFormat::R8ui => matches!(self, Self::U8(_)),
            InternalFormat::R8i => matches!(self, Self::I8(_)),
            InternalFormat::R16ui => matches!(self, Self::U16(_)),
            InternalFormat::R16i => matches!(self, Self::I16(_)),
            InternalFormat::R32ui => matches!(self, Self::U32(_)),
            InternalFormat::R32i => matches!(self, Self::I32(_)),

            InternalFormat::Rg8 => matches!(self, Self::U8(_)),
            InternalFormat::Rg8Snorm => matches!(self, Self::I8(_)),
            InternalFormat::Rg16f => matches!(self, Self::F16(_) | Self::F32(_)),
            InternalFormat::Rg32f => matches!(self, Self::F32(_)),
            InternalFormat::Rg8ui => matches!(self, Self::U8(_)),
            InternalFormat::Rg8i => matches!(self, Self::I8(_)),
            InternalFormat::Rg16ui => matches!(self, Self::U16(_)),
            InternalFormat::Rg16i => matches!(self, Self::I16(_)),
            InternalFormat::Rg32ui => matches!(self, Self::U32(_)),
            InternalFormat::Rg32i => matches!(self, Self::I32(_)),

            InternalFormat::Rgb8 => matches!(self, Self::U8(_)),
            InternalFormat::Srgb8 => matches!(self, Self::U8(_)),
            InternalFormat::Rgb565 => matches!(self, Self::U8(_) | Self::Packed5_6_5(_)),
            InternalFormat::Rgb8Snorm => matches!(self, Self::I8(_)),
            InternalFormat::R11fG11fB10f => matches!(
                self,
                Self::F16(_) | Self::F32(_) | Self::Reverse10F11F11F(_)
            ),
            InternalFormat::Rgb9E5 => {
                matches!(self, Self::F16(_) | Self::F32(_) | Self::Reverse5_9_9_9(_))
            }
            InternalFormat::Rgb16f => matches!(self, Self::F16(_) | Self::F32(_)),
            InternalFormat::Rgb32f => matches!(self, Self::F32(_)),
            InternalFormat::Rgb8ui => matches!(self, Self::U8(_)),
            InternalFormat::Rgb8i => matches!(self, Self::I8(_)),
            InternalFormat::Rgb16ui => matches!(self, Self::U16(_)),
            InternalFormat::Rgb16i => matches!(self, Self::I16(_)),
            InternalFormat::Rgb32ui => matches!(self, Self::U32(_)),
            InternalFormat::Rgb32i => matches!(self, Self::I32(_)),

            InternalFormat::Rgba8 => matches!(self, Self::U8(_)),
            InternalFormat::Srgb8Alpha8 => matches!(self, Self::U8(_)),
            InternalFormat::Rgba8Snorm => matches!(self, Self::I8(_)),
            InternalFormat::Rgb5A1 => matches!(
                self,
                Self::U8(_) | Self::Packed5_5_5_1(_) | Self::Reverse2_10_10_10(_)
            ),
            InternalFormat::Rgba4 => matches!(self, Self::U8(_) | Self::Packed4_4_4_4(_)),
            InternalFormat::Rgb10A2 => matches!(self, Self::Reverse2_10_10_10(_)),
            InternalFormat::Rgba16f => matches!(self, Self::F16(_) | Self::F32(_)),
            InternalFormat::Rgba32f => matches!(self, Self::F32(_)),
            InternalFormat::Rgba8ui => matches!(self, Self::U8(_)),
            InternalFormat::Rgba8i => matches!(self, Self::I8(_)),
            InternalFormat::Rgb10A2ui => matches!(self, Self::Reverse2_10_10_10(_)),
            InternalFormat::Rgba16ui => matches!(self, Self::U16(_)),
            InternalFormat::Rgba16i => matches!(self, Self::I16(_)),
            InternalFormat::Rgba32i => matches!(self, Self::I32(_)),
            InternalFormat::Rgba32ui => matches!(self, Self::U32(_)),

            InternalFormat::DepthComponent16 => matches!(self, Self::U16(_) | Self::U32(_)),
            InternalFormat::DepthComponent24 => matches!(self, Self::U32(_)),
            InternalFormat::DepthComponent32f => matches!(self, Self::F32(_)),
            InternalFormat::Depth24Stencil8 => matches!(self, Self::Packed24_8(_)),
            InternalFormat::Depth32fStencil8 => matches!(self, Self::F32Reverse24_8(_)),
        }
    }
}

#[repr(u32)]
pub enum Swizzle {
    Red = gl::RED,
    Green = gl::GREEN,
    Blue = gl::BLUE,
    Alpha = gl::ALPHA,
    Zero = gl::ZERO,
    One = gl::ONE,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for Swizzle {}
pub enum Filter {
    Nearest,
    /// For Color images, enables linear filtering.
    /// For Depth images, enables Percentage-Closer Filtering
    Linear,
}
#[repr(u32)]
pub enum Wrap {
    ClampToEdge = gl::CLAMP_TO_EDGE,
    MirroredRepeat = gl::MIRRORED_REPEAT,
    Repeat = gl::REPEAT,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for Wrap {}

/// An application-owned texture. (i.e, *not* the default texture `0`)
///
/// The type parameter, `Dim`, represents the kind of initialization. E.g., binding a [`Stateless`]
/// texture to [`crate::slot::texture::Slot2D`] changes it into a `Texture<D2>`.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks memory"]
pub struct Texture<Dim: Dimensionality>(
    pub(crate) NonZeroName,
    pub(crate) std::marker::PhantomData<Dim>,
);
impl<Dim: Dimensionality> Texture<Dim> {
    pub const TARGET: GLenum = Dim::TARGET;
}

pub type Texture2D = Texture<D2>;
pub type Texture2DArray = Texture<D2Array>;
pub type Texture3D = Texture<D3>;
pub type TextureCube = Texture<Cube>;

/// An application-owned texture which does not currently have a dimensionality, properties,
/// nor datastore. Bind it to a texture target in order to initialize the GL-internal datastructures.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks memory"]
pub struct Stateless(pub(crate) NonZeroName);

impl crate::sealed::Sealed for Stateless {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl crate::ThinGLObject for Stateless {}
impl<Dim: Dimensionality> crate::sealed::Sealed for Texture<Dim> {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl<Dim: Dimensionality> crate::ThinGLObject for Texture<Dim> {}
