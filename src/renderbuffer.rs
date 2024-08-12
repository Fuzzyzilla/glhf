//! Types and parameter enums for Renderbuffers.
use crate::{
    gl::{self, types::GLenum},
    NonZeroName,
};

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum InternalFormat {
    // Sized color formats
    R8 = gl::R8,
    R8ui = gl::R8UI,
    R8i = gl::R8I,
    R16ui = gl::R16UI,
    R16i = gl::R16I,
    R32ui = gl::R32UI,
    R32i = gl::R32I,
    Rg8 = gl::RG8,
    Rg8ui = gl::RG8UI,
    Rg8i = gl::RG8I,
    Rg16ui = gl::RG16UI,
    Rg16i = gl::RG16I,
    Rg32ui = gl::RG32UI,
    Rg32i = gl::RG32I,
    Rgb8 = gl::RGB8,
    Rgb565 = gl::RGB565,
    Rgba8 = gl::RGBA8,
    Srgb8Alpha8 = gl::SRGB8_ALPHA8,
    Rgb5A1 = gl::RGB5_A1,
    Rgba4 = gl::RGBA4,
    Rgb10A2 = gl::RGB10_A2,
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
    StencilIndex8 = gl::STENCIL_INDEX8,
}

// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for InternalFormat {}
impl InternalFormat {
    /// Get the "format" `GLenum` associated with this internal format.
    /// This describes the layout of pixel data in a buffer.
    ///
    /// This is *not* the same as `self.as_gl`
    #[must_use]
    pub fn format(&self) -> crate::texture::Format {
        use crate::texture::Format;
        match self {
            Self::R8 => Format::Red,
            Self::R8ui => Format::RedInteger,
            Self::R8i => Format::RedInteger,
            Self::R16ui => Format::RedInteger,
            Self::R16i => Format::RedInteger,
            Self::R32ui => Format::RedInteger,
            Self::R32i => Format::RedInteger,

            Self::Rg8 => Format::RG,
            Self::Rg8ui => Format::RGInteger,
            Self::Rg8i => Format::RGInteger,
            Self::Rg16ui => Format::RGInteger,
            Self::Rg16i => Format::RGInteger,
            Self::Rg32ui => Format::RGInteger,
            Self::Rg32i => Format::RGInteger,

            Self::Rgb8 => Format::RGB,
            Self::Rgb565 => Format::RGB,

            Self::Rgba8 => Format::RGBA,
            Self::Srgb8Alpha8 => Format::RGBA,
            Self::Rgb5A1 => Format::RGBA,
            Self::Rgba4 => Format::RGBA,
            Self::Rgb10A2 => Format::RGBA,
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
            Self::StencilIndex8 => Format::Stencil,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum InternalFormatMultisample {
    // Sized color formats
    R8 = gl::R8,
    Rg8 = gl::RG8,
    Rgb8 = gl::RGB8,
    Rgb565 = gl::RGB565,
    Rgba8 = gl::RGBA8,
    Srgb8Alpha8 = gl::SRGB8_ALPHA8,
    Rgb5A1 = gl::RGB5_A1,
    Rgba4 = gl::RGBA4,
    Rgb10A2 = gl::RGB10_A2,

    // Sized depth / stencil
    DepthComponent16 = gl::DEPTH_COMPONENT16,
    DepthComponent24 = gl::DEPTH_COMPONENT24,
    DepthComponent32f = gl::DEPTH_COMPONENT32F,
    Depth24Stencil8 = gl::DEPTH24_STENCIL8,
    Depth32fStencil8 = gl::DEPTH32F_STENCIL8,
    StencilIndex8 = gl::STENCIL_INDEX8,
}

// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for InternalFormatMultisample {}
impl InternalFormatMultisample {
    /// Get the "format" `GLenum` associated with this internal format.
    /// This describes the layout of pixel data in a buffer.
    ///
    /// This is *not* the same as `self.as_gl`
    #[must_use]
    pub fn format(&self) -> crate::texture::Format {
        use crate::texture::Format;
        match self {
            Self::R8 => Format::Red,

            Self::Rg8 => Format::RG,

            Self::Rgb8 => Format::RGB,
            Self::Rgb565 => Format::RGB,

            Self::Rgba8 => Format::RGBA,
            Self::Srgb8Alpha8 => Format::RGBA,
            Self::Rgb5A1 => Format::RGBA,
            Self::Rgba4 => Format::RGBA,
            Self::Rgb10A2 => Format::RGBA,

            Self::DepthComponent16 => Format::DepthComponent,
            Self::DepthComponent24 => Format::DepthComponent,
            Self::DepthComponent32f => Format::DepthComponent,
            Self::Depth24Stencil8 => Format::DepthStencil,
            Self::Depth32fStencil8 => Format::DepthStencil,
            Self::StencilIndex8 => Format::Stencil,
        }
    }
}

/// An application-owned renderbufferbuffer.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks resources"]
pub struct Renderbuffer(pub(crate) NonZeroName);
impl Renderbuffer {
    pub const TARGET: GLenum = gl::RENDERBUFFER;
}

impl crate::sealed::Sealed for Renderbuffer {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl crate::ThinGLObject for Renderbuffer {}
