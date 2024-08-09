use crate::{
    framebuffer::{Attachment, Buffer, Complete, DefaultBuffer, Incomplete},
    gl,
    slot::marker::{Defaultness, IsDefault, NotDefault, Unknown},
    texture::{Dimensionality, Texture2D},
    GLEnum, GLenum, NotSync, ThinGLObject,
};

/// Returns true if the slice contains no duplicates.
/// Complexity is O(n^2), but has low overhead. Use for smol things!
fn is_all_unique<T: Eq>(slice: &[T]) -> bool {
    // https://stackoverflow.com/a/46766782 cuz I was too lazy
    (1..slice.len()).all(|i| !slice[i..].contains(&slice[i - 1]))
}

/// Marker trait for the two framebuffer targets, [`Draw`] and [`Read`]
pub trait Target: crate::sealed::Sealed {
    const TARGET: GLenum;
}
/// Marker for `GL_DRAW_FRAMEBUFFER`
#[derive(Debug)]
pub struct Draw;
impl crate::sealed::Sealed for Draw {}
impl Target for Draw {
    const TARGET: GLenum = gl::DRAW_FRAMEBUFFER;
}

/// Marker for `GL_READ_FRAMEBUFFER`
#[derive(Debug)]
pub struct Read;
impl crate::sealed::Sealed for Read {}
impl Target for Read {
    const TARGET: GLenum = gl::READ_FRAMEBUFFER;
}

bitflags::bitflags! {
    /// Defines which aspects of a framebuffer to affect.
    #[repr(transparent)]
    pub struct AspectMask: gl::types::GLbitfield {
        /// All current color attachments (contextually defined by `Active::draw_buffers` or
        /// `Active::read_buffer`, depending on if this is used for a read or write operation)
        const COLOR = gl::COLOR_BUFFER_BIT;
        /// The depth attachment, if any.
        const DEPTH = gl::DEPTH_BUFFER_BIT;
        /// The stencil attachment, if any.
        const STENCIL = gl::STENCIL_BUFFER_BIT;
    }
}

/// Specifies a rectangle to blit.
/// It is not specified whether which corner `from` and `to` refer to,
/// as it is arbitrary as long as both `read` and `write` rectangles agree.
///
/// One rectangle may be the mirror of the other, which will cause the transferred
/// image to be flipped.
struct BlitRectangle {
    /// Lower bound, inclusive.
    from: [i32; 2],
    /// Upper bound, exclusive.
    to_exclusive: [i32; 2],
}
pub struct BlitInfo {
    read: BlitRectangle,
    write: BlitRectangle,
    /// If enlarging, what filter should be applied to the color planes?
    filter: crate::texture::Filter,
    /// Which aspects to copy?
    ///
    /// If this contains Depth or Stencil, [`Self::filter`] must be `Nearest`.
    mask: AspectMask,
}

/// Entry points for `glFramebuffer*`
#[derive(Debug)]
pub struct Active<'slot, Slot, Default: Defaultness, Completeness>(
    std::marker::PhantomData<&'slot ()>,
    std::marker::PhantomData<(Default, Slot, Completeness)>,
);

impl<T: Target> Active<'_, T, NotDefault, Incomplete> {
    #[doc(alias = "glFramebufferTexture2D")]
    pub fn texture_2d(&self, texture: &Texture2D, attachment: Attachment, mip_level: u32) -> &Self {
        unsafe {
            gl::FramebufferTexture2D(
                T::TARGET,
                attachment.as_gl(),
                Texture2D::TARGET,
                texture.name().into(),
                mip_level.try_into().unwrap(),
            );
        }
        self
    }
}

impl<AnyDefaultness: Defaultness> Active<'_, Draw, AnyDefaultness, Complete> {
    /// Blit data from the read buffer into this buffer.
    ///
    /// The read buffer's current color attachment ([`Active::read_buffer`]) is copied
    /// to each of this buffer's [`Active::draw_buffers`].
    ///
    /// # Safety
    /// If the read buffer and any of the draw buffers refer to the same resource and the source
    /// and destination rectangles overlap, behavior is undefined.
    #[doc(alias = "glBlitFramebuffer")]
    pub unsafe fn blit_from<OtherDefaultness: Defaultness>(
        &self,
        _from: &Active<Read, OtherDefaultness, Complete>,
        info: &BlitInfo,
    ) -> &Self {
        if info.mask.is_empty() {
            return self;
        }
        unsafe {
            gl::BlitFramebuffer(
                info.read.from[0],
                info.read.from[1],
                info.read.to_exclusive[0],
                info.read.to_exclusive[1],
                info.write.from[0],
                info.write.from[1],
                info.write.to_exclusive[0],
                info.write.to_exclusive[1],
                info.mask.bits(),
                match info.filter {
                    crate::texture::Filter::Linear => gl::LINEAR,
                    crate::texture::Filter::Nearest => gl::NEAREST,
                },
            );
        }

        self
    }
    /// Clear color, depth, and/or stencil buffers. Aspects not contained in the framebuffer are ignored.
    ///
    /// Affected color buffers are limited to those selected by [`Self::draw_buffers`].
    ///
    /// The clear values are inherited from the global values `ClearColor`, `ClearDepth`, and `ClearStencil`.
    #[doc(alias = "glClear")]
    pub fn clear(&self, mask: AspectMask) -> &Self {
        if mask.is_empty() {
            return self;
        }
        unsafe {
            gl::Clear(mask.bits());
        }
        self
    }
}
impl<AnyDefaultness: Defaultness> Active<'_, Read, AnyDefaultness, Complete> {
    /// Blit data from this buffer into the write buffer.
    ///
    /// This is the reverse of [Active<'_, Draw, OtherDefaultness, Complete>::blit_from],
    /// see that function for more information.
    #[allow(clippy::missing_safety_doc)]
    #[doc(alias = "glBlitFramebuffer")]
    pub unsafe fn blit_to<OtherDefaultness: Defaultness>(
        &self,
        other: &Active<'_, Draw, OtherDefaultness, Complete>,
        info: &BlitInfo,
    ) -> &Self {
        other.blit_from(self, info);
        self
    }
    /// Copy texels from the current [`Self::read_buffer`] to the given bound texture.
    ///
    /// Texels are taken from the read buffer starting at `source_offset`, and `size` texels
    /// are trasferred to the texture mip given by `level` starting at `destination_offset`.
    ///
    /// `[0, 0]` is defined to be the lower-left corner.
    ///
    /// # Safety
    /// If the source range extends beyond the extent of the current `read_buffer`, the values
    /// transferred from those texels are undefined. This is *not* immediate UB, but it would
    /// be UB for any read access to those values in the destination texture.
    #[doc(alias = "glCopyTexSubImage2D")]
    pub unsafe fn copy_subimage_to(
        &self,
        _to: &crate::slot::texture::Active<'_, crate::texture::D2>,
        level: u32,
        // Intentionally signed. It is not UB to read beyond the buffer, but it is UB to access those values read.
        // This may still be useful, idk X3
        source_offset: [i32; 2],
        destination_offset: [u32; 2],
        size: [u32; 2],
    ) -> &Self {
        unsafe {
            gl::CopyTexSubImage2D(
                crate::texture::D2::TARGET,
                level.try_into().unwrap(),
                destination_offset[0].try_into().unwrap(),
                destination_offset[1].try_into().unwrap(),
                source_offset[0],
                source_offset[1],
                size[0].try_into().unwrap(),
                size[1].try_into().unwrap(),
            );
        }
        self
    }
    /// Copy texels from the current [`Self::read_buffer`] to the given bound texture.
    ///
    /// Texels are taken from the read buffer starting at `source_offset`, and `size` texels
    /// are trasferred to the entire texture mip given by `level`.
    ///
    /// `[0, 0]` is defined to be the lower-left corner.
    ///
    /// # Safety
    /// If the source range extends beyond the extent of the current `read_buffer`, the values
    /// transferred from those texels are undefined. This is *not* immediate UB, but it would
    /// be UB for any read access to those values in the destination texture.
    #[doc(alias = "glCopyTexImage2D")]
    pub unsafe fn copy_image_to(
        &self,
        _to: &crate::slot::texture::Active<'_, crate::texture::D2>,
        level: u32,
        // Fixme: this actually only accepts a subset of this enum.
        internal_format: crate::texture::InternalFormat,
        // Intentionally signed. It is not UB to read beyond the buffer, but it is UB to access those values read.
        // This may still be useful, idk X3
        source_offset: [i32; 2],
        size: [u32; 2],
    ) -> &Self {
        unsafe {
            gl::CopyTexImage2D(
                crate::texture::D2::TARGET,
                level.try_into().unwrap(),
                internal_format.as_gl(),
                source_offset[0],
                source_offset[1],
                size[0].try_into().unwrap(),
                size[1].try_into().unwrap(),
                0,
            );
        }
        self
    }
}

impl<AnyCompleteness> Active<'_, Draw, NotDefault, AnyCompleteness> {
    /// Direct fragment outputs into appropriate buffers.
    /// I.e., Fragment output 0 will go into the buffer defined by `buffers[0]`.
    /// If the slice is too short, remaining slots default to [`Buffer::None`]
    ///
    /// # Panics
    /// Every element of `buffers` must be either none or a unique value.
    #[doc(alias = "glDrawBuffers")]
    pub fn draw_buffers(&self, buffers: &[Buffer]) -> &Self {
        assert!(is_all_unique(buffers));
        // Cast safety: Fieldless repr(u32), can be safely reinterpreted as &[u32]
        unsafe { gl::DrawBuffers(buffers.len().try_into().unwrap(), buffers.as_ptr().cast()) }
        self
    }
}

impl Active<'_, Draw, IsDefault, Complete> {
    /// Direct fragment outputs into appropriate buffers.
    /// I.e., Fragment output 0 will go into the buffer defined by `buffers[0]`.
    /// If the slice is too short, remaining slots default to [`DefaultBuffer::None`]
    ///
    /// # Panics
    /// Every element of `buffers` must be either none or a unique value.
    #[doc(alias = "glDrawBuffers")]
    pub fn draw_buffers(&self, buffers: &[DefaultBuffer]) -> &Self {
        assert!(is_all_unique(buffers));
        // Cast safety: Fieldless repr(u32), can be safely reinterpreted as &[u32]
        unsafe { gl::DrawBuffers(buffers.len().try_into().unwrap(), buffers.as_ptr().cast()) }
        self
    }
}

impl<AnyCompleteness> Active<'_, Read, NotDefault, AnyCompleteness> {
    /// Set the source for pixel read operations.
    #[doc(alias = "glReadBuffer")]
    pub fn read_buffer(&self, buffer: Buffer) -> &Self {
        unsafe { gl::ReadBuffer(buffer.as_gl()) }
        self
    }
}

impl Active<'_, Draw, IsDefault, Complete> {
    /// Set the source for pixel read operations.
    #[doc(alias = "glReadBuffer")]
    pub fn read_buffer(&self, buffer: DefaultBuffer) -> &Self {
        unsafe { gl::ReadBuffer(buffer.as_gl()) }
        self
    }
}

#[derive(Debug)]
#[must_use = "dropping a gl handle leaks resources"]
pub struct IncompleteError<'slot, Slot> {
    /// The activation token of the framebuffer. Even if it failed to pass completion,
    /// it is bound.
    pub active: Active<'slot, Slot, NotDefault, Incomplete>,
    /// Returns ownership of the framebuffer.
    pub framebuffer: Incomplete,
    pub kind: IncompleteErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IncompleteErrorKind {
    /// GL internal error, or unknown status.
    Unspecified = 0,
    // Not needed, type state prevents this :3
    // Undefined,
    /// One or more of the attachments are "framebuffer incomplete".
    Attachment = gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT,
    /// The framebuffer has no attachments.
    MissingAttachment = gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT,
    /// Either of:
    /// * Depth and Stencil attachments refer to different renderbuffers or textures.
    /// * An implementation-defined restriction was violated by the combination of internal formats.
    Unsupported = gl::FRAMEBUFFER_UNSUPPORTED,
    /// Sample counts are not all the same for every renderbuffer or texture.
    Multisample = gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE,
    /// Either of:
    /// * Some attachments are layered while others are not.
    /// * Not all Texture attachments are from the same target (i.e., some are `Texture2D` and some are `TextureCube`)
    //    ^^^^ Can we prevent this second one statically? perhaps...
    LayerTargets = gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS,
    // Dimensions is a defined error, which makes sense, but it is not mentioned in the GLES3.X spec?
}
impl IncompleteErrorKind {
    #[must_use]
    pub fn from_gl(gl: GLenum) -> Self {
        match gl {
            gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => Self::Attachment,
            gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => Self::MissingAttachment,
            gl::FRAMEBUFFER_UNSUPPORTED => Self::Unsupported,
            gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => Self::Multisample,
            gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => Self::LayerTargets,
            _ => Self::Unspecified,
        }
    }
}

pub struct Slot<T: Target>(pub(crate) NotSync, pub(crate) std::marker::PhantomData<T>);
impl<T: Target> Slot<T> {
    /// Bind a user-defined framebuffer to this slot.
    #[doc(alias = "glBindFramebuffer")]
    pub fn bind(&mut self, framebuffer: &Incomplete) -> Active<T, NotDefault, Incomplete> {
        unsafe {
            gl::BindFramebuffer(T::TARGET, framebuffer.0.get());
        }
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Bind a user-defined framebuffer to this slot.
    #[doc(alias = "glBindFramebuffer")]
    pub fn bind_complete(&mut self, framebuffer: &Complete) -> Active<T, NotDefault, Complete> {
        unsafe {
            gl::BindFramebuffer(T::TARGET, framebuffer.0.get());
        }
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Check completeness of the given framebuffer, binding it in the process.
    ///
    /// On failure, the incomplete framebuffer is returned unchanged.
    // It is a limitation of my design that this requires a possibly redundant bind..
    #[doc(alias = "glCheckFramebufferStatus")]
    pub fn try_complete(
        &mut self,
        framebuffer: Incomplete,
    ) -> Result<(Complete, Active<T, NotDefault, Complete>), IncompleteError<T>> {
        let active = self.bind(&framebuffer);
        let status = unsafe { gl::CheckFramebufferStatus(T::TARGET) };
        if status == gl::FRAMEBUFFER_COMPLETE {
            Ok((
                // Safety - we just checked, dummy!
                unsafe { framebuffer.into_complete_unchecked() },
                Active(std::marker::PhantomData, std::marker::PhantomData),
            ))
        } else {
            Err(IncompleteError {
                active,
                kind: IncompleteErrorKind::from_gl(status),
                framebuffer,
            })
        }
    }
    /// Bind the default framebuffer, 0, to this slot.
    #[doc(alias = "glBindFramebuffer")]
    pub fn bind_default(&mut self) -> Active<T, IsDefault, Complete> {
        unsafe {
            gl::BindFramebuffer(T::TARGET, 0);
        }
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Inherit the currently bound framebuffer. This may be the default framebuffer.
    ///
    /// Some functionality is limited when the type of framebuffer (`Default` or `NotDefault`) is not known.
    #[must_use]
    pub fn inherit(&self) -> Active<T, Unknown, Unknown> {
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
}

pub struct Slots {
    pub draw: Slot<Draw>,
    pub read: Slot<Read>,
}
impl Slots {
    /// Bind a framebuffer to both the read and the draw slots.
    ///
    /// Refer to the individual slots to bind individually.
    #[doc(alias = "glBindFramebuffer")]
    pub fn bind(
        &mut self,
        framebuffer: &Incomplete,
    ) -> (
        Active<Read, NotDefault, Incomplete>,
        Active<Draw, NotDefault, Incomplete>,
    ) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer.0.get());
        }
        (
            Active(std::marker::PhantomData, std::marker::PhantomData),
            Active(std::marker::PhantomData, std::marker::PhantomData),
        )
    }
    /// Bind a framebuffer to both the read and the draw slots.
    ///
    /// Refer to the individual slots to bind individually.
    #[doc(alias = "glBindFramebuffer")]
    pub fn bind_complete(
        &mut self,
        framebuffer: &Complete,
    ) -> (
        Active<Read, NotDefault, Complete>,
        Active<Draw, NotDefault, Complete>,
    ) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer.0.get());
        }
        (
            Active(std::marker::PhantomData, std::marker::PhantomData),
            Active(std::marker::PhantomData, std::marker::PhantomData),
        )
    }
    /// Bind the default framebuffer to both the read and the draw slots.
    ///
    /// Refer to the individual slots to bind individually.
    #[doc(alias = "glBindFramebuffer")]
    pub fn bind_default(
        &mut self,
    ) -> (
        Active<Read, IsDefault, Complete>,
        Active<Draw, IsDefault, Complete>,
    ) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        (
            Active(std::marker::PhantomData, std::marker::PhantomData),
            Active(std::marker::PhantomData, std::marker::PhantomData),
        )
    }
    /// Delete framebuffers. If any were bound to a slot, the slot becomes bound to the default framebuffer.
    ///
    /// To delete [`Complete`] framebuffers, downgrade them to incomplete using [`Into::into`].
    #[doc(alias = "glDeleteFramebuffers")]
    pub fn delete<const N: usize>(&mut self, framebuffers: [Incomplete; N]) {
        unsafe { crate::gl_delete_with(gl::DeleteFramebuffers, framebuffers) }
    }
}
