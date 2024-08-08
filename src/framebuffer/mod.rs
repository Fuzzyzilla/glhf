use super::{gl, NonZeroName};

/// Buffers available for reading and writing on user-created framebuffers.
#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum Buffer {
    None = gl::NONE,
    ColorAttachment0 = gl::COLOR_ATTACHMENT0,
    ColorAttachment1 = gl::COLOR_ATTACHMENT1,
    ColorAttachment2 = gl::COLOR_ATTACHMENT2,
    ColorAttachment3 = gl::COLOR_ATTACHMENT3,
    // This is the minimum requirement for GLES3.0.
    // Should we extend this? Maybe have a ColorAttachment(n) tuple variant?
    // If so, remember to fix ActiveDraw::<NotDefault>::draw_buffers cuz it assumes this is fieldless lol
}

// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for Buffer {}

/// An attachment point for binding a Texture or Renderbuffer to a framebuffer.
#[repr(u32)]
pub enum Attachment {
    Color0 = gl::COLOR_ATTACHMENT0,
    Color1 = gl::COLOR_ATTACHMENT1,
    Color2 = gl::COLOR_ATTACHMENT2,
    Color3 = gl::COLOR_ATTACHMENT3,
    Depth = gl::DEPTH_ATTACHMENT,
    Stencil = gl::STENCIL_ATTACHMENT,
    DepthStencil = gl::DEPTH_STENCIL_ATTACHMENT,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for Attachment {}

/// Buffers available for reading and writing on the Default framebuffer.
#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum DefaultBuffer {
    None = gl::NONE,
    Back = gl::BACK,
}

// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for DefaultBuffer {}

/// A framebuffer which has not been completeness checked yet.
///
/// To make [`Complete`], use [`crate::slot::framebuffer::Slot::try_complete`].
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks resources"]
#[derive(Debug)]
pub struct Incomplete(pub(crate) NonZeroName);
impl crate::sealed::Sealed for Incomplete {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl crate::ThinGLObject for Incomplete {}

/// A framebuffer that is known to be complete.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks resources"]
#[derive(Debug)]
pub struct Complete(pub(crate) NonZeroName);
impl crate::sealed::Sealed for Complete {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl crate::ThinGLObject for Complete {}

impl Incomplete {
    /// Make `self` into a completed framebuffer, without checking with the GL.
    /// If possible, use the checked methods available on
    /// [the framebuffer slots](`crate::slot::framebuffer::Slot::try_complete`).
    ///
    /// `Incomplete::from(complete).into_complete_unchecked()` is always valid.
    ///
    /// # Safety
    /// The framebuffer must be in a complete state, i.e. `glCheckFramebufferStatus` called with this framebuffer would
    /// return `GL_FRAMEBUFFER_COMPLETE`.
    pub unsafe fn into_complete_unchecked(self) -> Complete {
        Complete(self.0)
    }
}

// Discard knowledge that the framebuffer is complete.
impl From<Complete> for Incomplete {
    fn from(complete: Complete) -> Self {
        Self(complete.0)
    }
}
