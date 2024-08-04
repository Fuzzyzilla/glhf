use crate::{
    buffer::{usage, Buffer},
    gl, GLEnum, GLenum, NotSync, ThinGLObject,
};

/// Marker for an active buffer which is known to be null. When a slot is
/// in this state, some of it's operations may instead be directed to host
/// memory via a simple pointer.
#[derive(Debug)]
pub struct Empty;
/// Marker for an active buffer is unknown whether it is the null or not.
#[derive(Debug)]
pub struct Unknown;
/// Marker for an active buffer which is known to be non-null.
#[derive(Debug)]
pub struct NotEmpty;

/// Marker trait for the many buffer targets.
pub trait Target: crate::sealed::Sealed {
    const TARGET: GLenum;
}

macro_rules! target {
    (pub struct $marker:ident = $value:ident) => {
        // This doc comment does not work with RA, but does at doc-build. weird.
        #[doc = "Marker for `"]
        #[doc = stringify!($value)]
        #[doc = "`"]
        #[derive(Debug)]
        pub struct $marker;
        impl crate::sealed::Sealed for $marker {}
        impl Target for $marker {
            const TARGET: GLenum = gl::$value;
        }
    };
}

target!(pub struct Array = ARRAY_BUFFER);
target!(pub struct CopyRead = COPY_READ_BUFFER);
target!(pub struct CopyWrite = COPY_WRITE_BUFFER);
target!(pub struct ElementArray = ELEMENT_ARRAY_BUFFER);
target!(pub struct PixelPack = PIXEL_PACK_BUFFER);
target!(pub struct PixelUnpack = PIXEL_UNPACK_BUFFER);
target!(pub struct TransformFeedback = TRANSFORM_FEEDBACK_BUFFER);
target!(pub struct Uniform = UNIFORM_BUFFER);

/// Marker trait for the many buffer targets.
/// # Safety
/// `FLAGS` should must contain `MAP_READ_BIT and optionally `MAP_WRITE_BIT`, and no others.
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

pub struct BufferMapGuard<'active, 'slot: 'active, T: Target, Access: MapAccess> {
    // We hold it the slot and buffer mutably, as it is an error to use the buffer for any operation
    // until it is unmapped. Holding it this way also ensures that Self::drop has safe access
    // to gl calls due to safety precondition of `crate::GLHF`.
    _active: &'active mut Active<'slot, T, NotEmpty>,
    access: std::marker::PhantomData<Access>,
    ptr: *mut u8,
    len: usize,
}

impl<T: Target, Access: MapAccess> std::ops::Deref for BufferMapGuard<'_, '_, T, Access> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        // Safety: not null (that's an error condition and self wouldn't have been made)
        // Align is one.
        unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
    }
}
impl<T: Target> std::ops::DerefMut for BufferMapGuard<'_, '_, T, ReadWrite> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: not null (that's an error condition and self wouldn't have been made)
        // Align is one.
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}
impl<T: Target, Access: MapAccess> Drop for BufferMapGuard<'_, '_, T, Access> {
    fn drop(&mut self) {
        unsafe {
            // Does raise errors, AFAIKT,
            gl::UnmapBuffer(T::TARGET);
        }
    }
}

#[derive(Debug)]
pub struct Active<'slot, Slot, Kind>(
    std::marker::PhantomData<&'slot mut ()>,
    std::marker::PhantomData<(Kind, Slot)>,
);
impl<'slot, T: Target> Active<'slot, T, NotEmpty> {
    /// (Re)allocate the datastore of the buffer and fill with bytes from `data`.
    // FIXME: The reference has verbage about the alignment of the buffer, and that
    // it must be properly aligned to the datatype of the buffer but... Well,, what
    // type?!?!? https://registry.khronos.org/OpenGL-Refpages/es3.0/
    pub fn data(&self, data: &[u8], frequency: usage::Frequency, access: usage::Access) -> &Self {
        unsafe {
            gl::BufferData(
                T::TARGET,
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
    pub unsafe fn data_uninit(
        &self,
        len: usize,
        frequency: usage::Frequency,
        access: usage::Access,
    ) -> &Self {
        unsafe {
            gl::BufferData(
                T::TARGET,
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
    pub fn sub_data(&self, offset: usize, data: &[u8]) -> &Self {
        unsafe {
            gl::BufferSubData(
                T::TARGET,
                offset.try_into().unwrap(),
                data.len().try_into().unwrap(),
                data.as_ptr().cast(),
            );
        }
        self
    }
    /// Map a byte range. Use the marker types [`Read`] and [`ReadWrite`] to specify access mode.
    ///
    /// If the range is unbounded to the right, a glGet is invoked to map the rest of the buffer size.
    /// # Panics
    /// If the range end is before the beginning.
    // FIXME: same alignment confusion as `Self::data`.
    pub fn map<'this, Access: MapAccess>(
        &'this mut self,
        range: impl std::ops::RangeBounds<usize>,
    ) -> BufferMapGuard<'this, 'slot, T, Access> {
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
            Bound::Unbounded => unsafe {
                // Query the size of the buffer.
                let mut size = 0;
                gl::GetBufferParameteri64v(
                    T::TARGET,
                    gl::BUFFER_SIZE,
                    std::ptr::addr_of_mut!(size),
                );
                size.try_into().unwrap()
            },
            Bound::Included(x) => x.checked_add(1).unwrap(),
            Bound::Excluded(x) => x,
        };
        let len = right
            .checked_sub(left)
            .expect("left bound should be less than right bound");

        self.map_impl(left, len)
    }
    fn map_impl<'this, Access: MapAccess>(
        &'this mut self,
        offset: usize,
        len: usize,
    ) -> BufferMapGuard<'this, 'slot, T, Access> {
        let ptr = unsafe {
            gl::MapBufferRange(
                T::TARGET,
                offset.try_into().unwrap(),
                len.try_into().unwrap(),
                Access::FLAGS,
            )
        };
        if ptr.is_null() {
            panic!("Map failed.");
        }
        BufferMapGuard {
            _active: self,
            access: std::marker::PhantomData,
            ptr: ptr.cast(),
            len,
        }
    }
}

pub struct Slot<T: Target>(pub(crate) NotSync, pub(crate) std::marker::PhantomData<T>);
impl<T: Target> Slot<T> {
    /// Bind a buffer to this slot.
    pub fn bind(&mut self, buffer: &Buffer) -> Active<T, NotEmpty> {
        unsafe {
            gl::BindBuffer(T::TARGET, buffer.name().get());
        }
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Make the slot empty.
    pub fn unbind(&mut self) -> Active<T, Empty> {
        unsafe {
            gl::BindBuffer(T::TARGET, 0);
        }
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Inherit the currently bound buffer - this may be no buffer at all.
    ///
    /// Most functionality is limited when the status of the buffer (Empty or NotEmpty) is not known.
    pub fn get(&self) -> Active<T, Unknown> {
        Active(std::marker::PhantomData, std::marker::PhantomData)
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
    pub fn delete<const N: usize>(&mut self, buffers: [Buffer; N]) {
        unsafe { crate::gl_delete_with(gl::DeleteBuffers, buffers) }
    }
}
