//! Binding and manipulating Buffers.
use crate::{
    buffer::{usage, Buffer},
    gl,
    slot::marker::{IsDefault, NotDefault, Unknown},
    GLenum, NotSync, ThinGLObject,
};

/// Marker trait for the many buffer binding targets.
pub trait Target: crate::sealed::Sealed {
    const TARGET: GLenum;
}

macro_rules! target {
    (pub struct $marker:ident = $value:ident$(,$doc:literal)?) => {
        // This doc comment does not work with RA, but does at doc-build. weird.
        #[doc = "Marker for `"]
        #[doc = stringify!($value)]
        #[doc = "`."]
        $(#[doc = concat!(" ", $doc)])?
        #[derive(Debug)]
        pub struct $marker;
        impl crate::sealed::Sealed for $marker {}
        impl Target for $marker {
            const TARGET: GLenum = gl::$value;
        }
    };
}

target!(
    pub struct Array = ARRAY_BUFFER,
    "Source for arbitrary vertex data when attached to a [`VertexArray`](crate::vertex_array::VertexArray) attribute."
);
target!(
    pub struct CopyRead = COPY_READ_BUFFER,
    "Scratch buffer for copy operations without disturbing other bindings."
);
target!(
    pub struct CopyWrite = COPY_WRITE_BUFFER,
    "Scratch buffer for copy operations without disturbing other bindings."
);
target!(
    pub struct ElementArray = ELEMENT_ARRAY_BUFFER,
    "Source for vertex indices when executing a [`Draw::elements`](crate::draw::Draw::elements) operation."
);
target!(
    pub struct PixelPack = PIXEL_PACK_BUFFER,
    "Destination for image downloads."
);
target!(
    pub struct PixelUnpack = PIXEL_UNPACK_BUFFER,
    "Source for image uploads."
);
target!(
    pub struct TransformFeedback = TRANSFORM_FEEDBACK_BUFFER,
    "Destination for vertex shader output feedback."
);
target!(pub struct Uniform = UNIFORM_BUFFER);

/// Marker trait for the many buffer targets.
/// # Safety
/// `FLAGS` should must contain `MAP_READ_BIT` and optionally `MAP_WRITE_BIT`, and no others.
pub unsafe trait MapAccess: crate::sealed::Sealed {
    const FLAGS: gl::types::GLbitfield;
}
/// Marker type for a Read-only buffer guard.
pub struct Read;
impl crate::sealed::Sealed for Read {}
unsafe impl MapAccess for Read {
    const FLAGS: gl::types::GLbitfield = gl::MAP_READ_BIT;
}
/// Marker type for a Read-Write buffer guard.
pub struct ReadWrite;
impl crate::sealed::Sealed for ReadWrite {}
unsafe impl MapAccess for ReadWrite {
    const FLAGS: gl::types::GLbitfield = gl::MAP_READ_BIT | gl::MAP_WRITE_BIT;
}

// TODO: Write only. It is substantially faster than `ReadWrite` if you don't need to read,
// but it is hard to wrap safely - Rust's type system assumes writable implies readable, so
// i'd instead need a bespoke opaque interface for a blackhole of bytes.

/// Read (and possibly write, as specified by [`MapAccess`]) access to a GL buffer. The buffer
/// memory is unmapped when this object is dropped.
///
/// For the most part the drop glue is infallible, except in very rare circumstances where the
/// graphics memory becomes lost, which causes a panic. Use [`MapGuard::unmap`] to handle this
/// condition.
///
/// This type dereferences to a (possibly mutable) byte slice.
pub struct MapGuard<'active, Binding: Target, Access: MapAccess> {
    // We hold it the slot and buffer mutably, as it is an error to use the buffer for any operation
    // until it is unmapped. Holding it this way also ensures that Self::drop has safe access
    // to gl calls due to safety precondition of `crate::GLHF`.
    _active: &'active mut Active<Binding, NotDefault>,
    access: std::marker::PhantomData<Access>,
    ptr: *mut u8,
    len: usize,
}

impl<Binding: Target, Access: MapAccess> MapGuard<'_, Binding, Access> {
    /// Explicitly unmap the datastore.
    /// This is the same as `Drop`ping the guard, however it allows for catching rare mapping failures.
    #[doc(alias = "glUnmapBuffer")]
    pub fn unmap(self) -> Result<(), UnmapError> {
        // We are manually implementing the drop glue, DON'T DOUBLE DROP PLS :3
        std::mem::forget(self);

        let success = unsafe { gl::UnmapBuffer(Binding::TARGET) } == true.into();

        if success {
            Ok(())
        } else {
            Err(UnmapError::Lost)
        }
    }
}

impl<Binding: Target, Access: MapAccess> std::ops::Deref for MapGuard<'_, Binding, Access> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        // Safety: not null (that's an error condition and self wouldn't have been made)
        // Align is one.
        unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
    }
}
impl<Binding: Target> std::ops::DerefMut for MapGuard<'_, Binding, ReadWrite> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: not null (that's an error condition and self wouldn't have been made)
        // Align is one.
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}
impl<Binding: Target, Access: MapAccess> Drop for MapGuard<'_, Binding, Access> {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(gl::UnmapBuffer(Binding::TARGET), true.into());
        }
    }
}

#[derive(Debug)]
pub enum UnmapError {
    /// For implementation-specific reasons, the buffer's datastore was lost as a result of
    /// this mapping. The entire buffer's contents become undefined, and must be re-written.
    Lost,
}

/// Entry points for `glBuffer*`
#[derive(Debug)]
pub struct Active<Slot, Kind>(std::marker::PhantomData<(Kind, Slot)>);
impl<Binding: Target> Active<Binding, NotDefault> {
    /// (Re)allocate the datastore of the buffer and fill with bytes from `data`.
    // FIXME: The reference has verbage about the alignment of the buffer, and that
    // it must be properly aligned to the datatype of the buffer but... Well,, what
    // type?!?!? https://registry.khronos.org/OpenGL-Refpages/es3.0/
    #[doc(alias = "glBufferData")]
    pub fn data(
        &mut self,
        data: &[u8],
        frequency: usage::Frequency,
        access: usage::Access,
    ) -> &mut Self {
        unsafe {
            gl::BufferData(
                Binding::TARGET,
                data.len().try_into().unwrap(),
                data.as_ptr().cast(),
                usage::as_gl(frequency, access),
            );
        }
        self
    }
    /// [`Self::data`], but does not initialize the data store.
    ///
    /// # Safety
    /// Host or GL read accesses on uninitialized memory is undefined behavior, ensure the
    /// buffer gets overwritten before any reads can take place.
    #[doc(alias = "glBufferData")]
    pub unsafe fn data_uninit(
        &mut self,
        len: usize,
        frequency: usage::Frequency,
        access: usage::Access,
    ) -> &mut Self {
        unsafe {
            gl::BufferData(
                Binding::TARGET,
                len.try_into().unwrap(),
                // Null for uninit
                std::ptr::null(),
                usage::as_gl(frequency, access),
            );
        }
        self
    }
    /// Ovwerite a sub-range of the data store.
    // FIXME: same alignment confusion as `Self::data`.
    #[doc(alias = "glBufferSubData")]
    pub fn sub_data(&mut self, offset: usize, data: &[u8]) -> &mut Self {
        unsafe {
            gl::BufferSubData(
                Binding::TARGET,
                offset.try_into().unwrap(),
                data.len().try_into().unwrap(),
                data.as_ptr().cast(),
            );
        }
        self
    }
    /// Copy bytes from one region of this buffer to another.
    ///
    /// The source and destination regions must not overlap.
    /// Neither the read nor write ranges may extend past the end of the buffer.
    #[doc(alias = "glCopyBufferSubData")]
    pub fn copy_self(&mut self, read_offset: usize, write_offset: usize, len: usize) -> &mut Self {
        // This has to be it's own unique fn because the other calls require mut and immutable ref
        // to self, obviously an error.

        // Make both a unique and shared ref to this binding, for a temporary time.
        // Usually this is instant UB, but we don't actually hold any mem - the mutability of self
        // is merely a hint!
        self.copy_from::<Binding>(super::zst_ref(), read_offset, write_offset, len)
    }
    /// Copy bytes from the other buffer into `self`.
    ///
    /// Neither the read nor write ranges may extend past the end of their respective buffers.
    #[doc(alias = "glCopyBufferSubData")]
    pub fn copy_from<OtherBinding: Target>(
        &mut self,
        _other: &Active<OtherBinding, NotDefault>,
        read_offset: usize,
        write_offset: usize,
        len: usize,
    ) -> &mut Self {
        if len == 0 {
            return self;
        }
        unsafe {
            gl::CopyBufferSubData(
                OtherBinding::TARGET,
                Binding::TARGET,
                read_offset.try_into().unwrap(),
                write_offset.try_into().unwrap(),
                len.try_into().unwrap(),
            );
        }
        self
    }
    /// See [`Active::copy_from`].
    #[doc(alias = "glCopyBufferSubData")]
    pub fn copy_to<OtherBinding: Target>(
        &self,
        other: &mut Active<OtherBinding, NotDefault>,
        read_offset: usize,
        write_offset: usize,
        len: usize,
    ) -> &Self {
        other.copy_from(self, read_offset, write_offset, len);
        self
    }
    /// Map a byte range. Use the marker types [`Read`] and [`ReadWrite`] to specify access mode.
    /// Even Read-only access requires mutable access to the buffer.
    ///
    /// If the range is unbounded to the right, a `glGet` is invoked to map the rest of the buffer size.
    ///
    /// Usage:
    /// ```no_run
    /// use glhf::{slot::buffer};
    /// # let gl : glhf::GLHF = todo!();
    /// # let buffer : glhf::buffer::Buffer = todo!();
    ///
    /// gl.buffer.array.bind(&buffer)
    ///     .map::<buffer::ReadWrite>(..)
    ///     .fill(10u8);
    /// ```
    /// # Alignment
    /// Unfortunately, the GLES API makes no guarantees on the alignment of the returned byte slice. Do
    /// not assume the pointer is aligned stronger than `1`.
    ///
    /// # Panics
    /// * The range end is before the beginning.
    /// * The range extends beyond the end of the datastore.
    /// * The implementation is out of memory.
    ///
    /// # Safety
    /// This function is safe to call in all situations.
    ///
    /// However, no part of the returned byte slice may be passed as an argument
    /// to *any* GL APIs other than `glUnmapBuffer`.
    // FIXME: same alignment confusion as `Self::data`.
    #[doc(alias = "glMapBuffer")]
    #[doc(alias = "glMapBufferRange")]
    pub unsafe fn map<Access: MapAccess>(
        &mut self,
        range: impl std::ops::RangeBounds<usize>,
    ) -> MapGuard<Binding, Access> {
        use std::ops::Bound;
        let left = range.start_bound().cloned();
        let right = range.end_bound().cloned();
        // Min offset, inclusive.
        let left = match left {
            Bound::Unbounded => 0,
            Bound::Included(x) => x,
            Bound::Excluded(x) => x.checked_add(1).unwrap(),
        };
        // Max offset, exclusive.
        let right = match right {
            Bound::Unbounded => self.len(),
            Bound::Included(x) => x.checked_add(1).unwrap(),
            Bound::Excluded(x) => x,
        };
        let len = right
            .checked_sub(left)
            .expect("left bound should be less than right bound");

        self.map_impl(left, len)
    }
    unsafe fn map_impl<Access: MapAccess>(
        &mut self,
        offset: usize,
        len: usize,
    ) -> MapGuard<Binding, Access> {
        let ptr = unsafe {
            gl::MapBufferRange(
                Binding::TARGET,
                offset.try_into().unwrap(),
                len.try_into().unwrap(),
                Access::FLAGS,
            )
        };
        assert!(!ptr.is_null());
        MapGuard {
            _active: self,
            access: std::marker::PhantomData,
            ptr: ptr.cast(),
            len,
        }
    }
    /// This is not cached and invokes a `glGet`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Get the length of the buffer, in bytes.
    ///
    /// This is not cached and invokes a `glGet`.
    #[doc(alias = "glGetBufferParameter")]
    #[doc(alias = "glGetBufferParameteriv")]
    #[doc(alias = "glGetBufferParameteri64v")]
    #[doc(alias = "GL_BUFFER_SIZE")]
    #[must_use]
    pub fn len(&self) -> usize {
        let len = unsafe {
            let mut len = std::mem::MaybeUninit::uninit();
            gl::GetBufferParameteri64v(Binding::TARGET, gl::BUFFER_SIZE, len.as_mut_ptr());
            len.assume_init()
        };
        len.try_into().unwrap()
    }
    /// Get the usage hints used at the time of the datastore's allocation.
    ///
    /// This is not cached and invokes a `glGet`.
    #[doc(alias = "glGetBufferParameter")]
    #[doc(alias = "glGetBufferParameteriv")]
    #[doc(alias = "GL_BUFFER_USAGE")]
    #[must_use]
    pub fn usage(&self) -> (usage::Frequency, usage::Access) {
        use usage::{Access as A, Frequency as F};
        let usage = unsafe {
            let mut usage = std::mem::MaybeUninit::uninit();
            gl::GetBufferParameteriv(Binding::TARGET, gl::BUFFER_USAGE, usage.as_mut_ptr());
            usage.assume_init()
        };
        match usage as GLenum {
            gl::STATIC_COPY => (F::Static, A::Copy),
            gl::STATIC_DRAW => (F::Static, A::Draw),
            gl::STATIC_READ => (F::Static, A::Read),

            gl::STREAM_COPY => (F::Stream, A::Copy),
            gl::STREAM_DRAW => (F::Stream, A::Draw),
            gl::STREAM_READ => (F::Stream, A::Read),

            gl::DYNAMIC_COPY => (F::Dynamic, A::Copy),
            gl::DYNAMIC_DRAW => (F::Dynamic, A::Draw),
            gl::DYNAMIC_READ => (F::Dynamic, A::Read),

            _ => unreachable!(),
        }
    }
}

pub struct Slot<Binding: Target>(
    pub(crate) NotSync,
    pub(crate) std::marker::PhantomData<Binding>,
);
impl<Binding: Target> Slot<Binding> {
    /// Bind a buffer to this slot.
    #[doc(alias = "glBindBuffer")]
    pub fn bind(&mut self, buffer: &Buffer) -> &mut Active<Binding, NotDefault> {
        unsafe {
            gl::BindBuffer(Binding::TARGET, buffer.name().get());
        }
        super::zst_mut()
    }
    /// Make the slot empty.
    #[doc(alias = "glBindBuffer")]
    pub fn unbind(&mut self) -> &mut Active<Binding, IsDefault> {
        unsafe {
            gl::BindBuffer(Binding::TARGET, 0);
        }
        super::zst_mut()
    }
    /// Inherit the currently bound buffer - this may be no buffer at all.
    ///
    /// Most functionality is limited when the status of the buffer (`Default` or `NotDefault`) is not known.
    #[must_use]
    pub fn inherit(&self) -> &Active<Binding, Unknown> {
        super::zst_ref()
    }
    /// Inherit the currently bound buffer - this may be no buffer at all.
    ///
    /// Most functionality is limited when the status of the buffer (`Default` or `NotDefault`) is not known.
    #[must_use]
    pub fn inherit_mut(&mut self) -> &mut Active<Binding, Unknown> {
        super::zst_mut()
    }
}

pub struct Slots {
    pub array: Slot<Array>,
    pub copy_read: Slot<CopyRead>,
    pub copy_write: Slot<CopyWrite>,
    pub element_array: Slot<ElementArray>,
    pub pixel_pack: Slot<PixelPack>,
    pub pixel_unpack: Slot<PixelUnpack>,
    pub transform_feedback: Slot<TransformFeedback>,
    pub uniform: Slot<Uniform>,
}
impl Slots {
    /// Delete buffers. If any were bound to a slot, the slot becomes unbound.
    #[doc(alias = "glDeleteBuffers")]
    pub fn delete<const N: usize>(&mut self, buffers: [Buffer; N]) {
        unsafe { crate::gl_delete_with(gl::DeleteBuffers, buffers) }
    }
}
