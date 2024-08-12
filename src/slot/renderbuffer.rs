use std::num::NonZero;

use crate::{
    gl,
    renderbuffer::{self, Renderbuffer},
    slot::marker::{IsDefault, NotDefault, Unknown},
    GLEnum, NotSync, ThinGLObject,
};

pub struct Active<Kind>(std::marker::PhantomData<Kind>);

impl Active<NotDefault> {
    /// Define the format and size of a renderbuffer.
    ///
    /// Contents become undefined, even if the parameters are identical to a previous
    /// call.
    #[doc(alias = "glRenderbufferStorage")]
    pub fn storage(
        &mut self,
        internal_format: renderbuffer::InternalFormat,
        width: NonZero<u32>,
        height: NonZero<u32>,
    ) -> &mut Self {
        unsafe {
            gl::RenderbufferStorage(
                Renderbuffer::TARGET,
                internal_format.as_gl(),
                width.get().try_into().unwrap(),
                height.get().try_into().unwrap(),
            );
        }
        self
    }
    /// Define the format and size of a multisampled renderbuffer. Sample counts may
    /// be rounded up to the nearest supported value.
    ///
    /// a sample count of `1` is *not* equivalent to a non-multisampled storage.
    ///
    /// Contents become undefined, even if the parameters are identical to a previous
    /// call.
    #[doc(alias = "glRenderbufferStorageMultisample")]
    pub fn storage_multisample(
        &mut self,
        internal_format: renderbuffer::InternalFormatMultisample,
        width: NonZero<u32>,
        height: NonZero<u32>,
        samples: NonZero<u8>,
    ) -> &mut Self {
        unsafe {
            gl::RenderbufferStorageMultisample(
                Renderbuffer::TARGET,
                samples.get().into(),
                internal_format.as_gl(),
                width.get().try_into().unwrap(),
                height.get().try_into().unwrap(),
            );
        }
        self
    }
}

/// Slots for binding renderbuffers. Corresponds to texture `glRenderbuffer*` operations.
pub struct Slot(pub(crate) NotSync);
impl Slot {
    /// Bind a rendebufferbuffer to this slot.
    #[doc(alias = "glBindRenderbuffer")]
    pub fn bind(&mut self, buffer: &Renderbuffer) -> &mut Active<NotDefault> {
        unsafe {
            gl::BindRenderbuffer(Renderbuffer::TARGET, buffer.name().get());
        }
        super::zst_mut()
    }
    /// Make the slot empty.
    #[doc(alias = "glBindRenderbuffer")]
    pub fn unbind(&mut self) -> &mut Active<IsDefault> {
        unsafe {
            gl::BindRenderbuffer(Renderbuffer::TARGET, 0);
        }
        super::zst_mut()
    }
    /// Inherit the currently bound buffer - this may be no buffer at all.
    ///
    /// Most functionality is limited when the status of the buffer (`Default` or `NotDefault`) is not known.
    #[must_use]
    pub fn inherit(&self) -> &Active<Unknown> {
        super::zst_ref()
    }
    /// Inherit the currently bound buffer - this may be no buffer at all.
    ///
    /// Most functionality is limited when the status of the buffer (`Default` or `NotDefault`) is not known.
    #[must_use]
    pub fn inherit_mut(&mut self) -> &mut Active<Unknown> {
        super::zst_mut()
    }
    /// Delete renderbuffers. If any were bound to this slot, the slot becomes unbound.
    #[doc(alias = "glDeleteTextures")]
    pub fn delete<const N: usize>(&mut self, textures: [renderbuffer::Renderbuffer; N]) {
        unsafe { crate::gl_delete_with(gl::DeleteRenderbuffers, textures) }
    }
}
