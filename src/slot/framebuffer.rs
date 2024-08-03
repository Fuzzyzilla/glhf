use crate::{
    framebuffer::{Attachment, Buffer, Complete, DefaultBuffer, Incomplete},
    gl,
    texture::Texture2D,
    GLEnum, GLenum, NotSync, ThinGLObject,
};

/// Returns true if the slice contains no duplicates.
/// Complexity is O(n^2), but has low overhead. Use for smol things!
fn is_all_unique<T: Eq>(slice: &[T]) -> bool {
    // https://stackoverflow.com/a/46766782 cuz I was too lazy
    (1..slice.len()).all(|i| !slice[i..].contains(&slice[i - 1]))
}

/// Marker for an active framebuffer which is known to be the default framebuffer.
#[derive(Debug)]
pub struct Default;
/// Marker for an active framebuffer is unknown whether it is the default or not.
#[derive(Debug)]
pub struct Unknown;
/// Marker for an active framebuffer which is known not to be the default framebuffer.
#[derive(Debug)]
pub struct NotDefault;

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

#[derive(Debug)]
pub struct Active<'slot, Slot, Kind>(
    std::marker::PhantomData<&'slot ()>,
    std::marker::PhantomData<(Kind, Slot)>,
);

impl<T: Target> Active<'_, T, NotDefault> {
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

impl Active<'_, Draw, NotDefault> {
    /// Direct fragment outputs into appropriate buffers.
    /// I.e., Fragment output 0 will go into the buffer defined by `buffers[0]`.
    /// If the slice is too short, remaining slots default to [`DrawBuffers::None`]
    ///
    /// Every element of `buffers` must be either none or a unique value.
    pub fn draw_buffers(&self, buffers: &[Buffer]) -> &Self {
        assert!(is_all_unique(buffers));
        // Cast safety: Fieldless repr(u32), can be safely reinterpreted as &[u32]
        unsafe { gl::DrawBuffers(buffers.len().try_into().unwrap(), buffers.as_ptr().cast()) }
        self
    }
}

impl Active<'_, Draw, Default> {
    /// Direct fragment outputs into appropriate buffers.
    /// I.e., Fragment output 0 will go into the buffer defined by `buffers[0]`.
    /// If the slice is too short, remaining slots default to [`DrawBuffers::None`]
    ///
    /// Every element of `buffers` must be either none or a unique value.
    pub fn draw_buffers(&self, buffers: &[DefaultBuffer]) -> &Self {
        assert!(is_all_unique(buffers));
        // Cast safety: Fieldless repr(u32), can be safely reinterpreted as &[u32]
        unsafe { gl::DrawBuffers(buffers.len().try_into().unwrap(), buffers.as_ptr().cast()) }
        self
    }
}

impl Active<'_, Read, NotDefault> {
    /// Set the source for pixel read operations.
    pub fn read_buffer(&self, buffer: Buffer) -> &Self {
        unsafe { gl::ReadBuffer(buffer.as_gl()) }
        self
    }
}

impl Active<'_, Draw, Default> {
    /// Set the source for pixel read operations.
    pub fn read_buffer(&self, buffer: DefaultBuffer) -> &Self {
        unsafe { gl::ReadBuffer(buffer.as_gl()) }
        self
    }
}

#[derive(Debug)]
#[must_use = "dropping a gl handle leaks memory"]
pub struct IncompleteError<'slot, Slot> {
    /// The activation token of the framebuffer. Even if it failed to pass completion,
    /// it is bound.
    pub active: Active<'slot, Slot, NotDefault>,
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
    /// * Not all Texture attachments are from the same target (i.e., some are Texture2D and some are TextureCube)
    //    ^^^^ Can we prevent this second one statically? perhaps...
    LayerTargets = gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS,
    // Dimensions is a defined error, which makes sense, but it is not mentioned in the GLES3.X spec?
}
impl IncompleteErrorKind {
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
    pub fn bind(&mut self, framebuffer: &Incomplete) -> Active<T, NotDefault> {
        unsafe {
            gl::BindFramebuffer(T::TARGET, framebuffer.0.get());
        }
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Check completeness of the given framebuffer, binding it in the process.
    ///
    /// On failure, the incomplete framebuffer is returned unchanged.
    // It is a limitation of my design that this requires a possibly redundant bind..
    pub fn try_complete(
        &mut self,
        framebuffer: Incomplete,
    ) -> Result<(Complete, Active<T, NotDefault>), IncompleteError<T>> {
        let active = self.bind(&framebuffer);
        let status = unsafe { gl::CheckFramebufferStatus(T::TARGET) };
        if status == gl::FRAMEBUFFER_COMPLETE {
            // Safety - we just checked, dummy!
            Ok((unsafe { framebuffer.into_complete_unchecked() }, active))
        } else {
            Err(IncompleteError {
                active,
                kind: IncompleteErrorKind::from_gl(status),
                framebuffer,
            })
        }
    }
    /// Bind the default framebuffer, 0, to this slot.
    pub fn bind_default(&mut self) -> Active<T, Default> {
        unsafe {
            gl::BindFramebuffer(T::TARGET, 0);
        }
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Inherit the currently bound framebuffer. This may be the default framebuffer.
    ///
    /// Some functionality is limited when the type of framebuffer (Default or NotDefault) is not known.
    pub fn get(&self) -> Active<T, Unknown> {
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
    pub fn bind(
        &mut self,
        framebuffer: &Incomplete,
    ) -> (Active<Read, NotDefault>, Active<Draw, NotDefault>) {
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
    pub fn bind_default(&mut self) -> (Active<Read, Default>, Active<Draw, Default>) {
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
    pub fn delete<const N: usize>(&mut self, framebuffers: [Incomplete; N]) {
        unsafe { crate::gl_delete_with(gl::DeleteFramebuffers, framebuffers) }
    }
}
