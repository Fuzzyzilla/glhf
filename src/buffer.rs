//! Types and parameter enums for Buffers.

use crate::{gl, GLenum, NonZeroName};

/// Hints to the GL as to how often and in what way a buffer will be used.
///
/// *It is very important to get this right* - while it is just a hint (and thus
/// does not restrict the actual abilities of the buffer) using a buffer in a way
/// inconsistant with its usage may be several orders of magnitude slower.
///
/// In practice, this determines in what memory space the buffer lives, and whether
/// it is transparently double-buffered by the GL.
pub mod usage {
    /// Describes the relationship between reads and writes
    pub enum Frequency {
        /// Contents will be read at most a few times after a write.
        Stream,
        /// Contents will be written once and read many times.
        Static,
        /// Contents will be written many times and read many times.
        Dynamic,
    }
    /// Describes the sources and destinations of reads and writes.
    pub enum Access {
        /// Host writes, GL reads.
        Draw,
        /// Host reads, GL writes.
        Read,
        /// GL writes, GL reads.
        Copy,
    }
    /// Combine a frequency and access into the corresponding `GLenum`.
    #[must_use]
    pub fn as_gl(frequency: Frequency, access: Access) -> super::GLenum {
        use super::gl;
        use Access as A;
        use Frequency as F;

        match (frequency, access) {
            // This can be done with arithmetic but that sounds evil >w<
            (F::Stream, A::Draw) => gl::STREAM_DRAW,
            (F::Stream, A::Read) => gl::STREAM_READ,
            (F::Stream, A::Copy) => gl::STREAM_COPY,

            (F::Static, A::Draw) => gl::STATIC_DRAW,
            (F::Static, A::Read) => gl::STATIC_READ,
            (F::Static, A::Copy) => gl::STATIC_COPY,

            (F::Dynamic, A::Draw) => gl::DYNAMIC_DRAW,
            (F::Dynamic, A::Read) => gl::DYNAMIC_READ,
            (F::Dynamic, A::Copy) => gl::DYNAMIC_COPY,
        }
    }
}

bitflags::bitflags! {
    /// Specifies buffer access mode.
    #[repr(transparent)]
    pub struct RawMapAccess: gl::types::GLbitfield {
        /// # Safety
        /// If not set, it is illegal to read from the mapped pointer range.
        /// Doing so may result in program termination.
        const Read = gl::MAP_READ_BIT;
        /// # Safety
        /// If not set, it is illegal to write to the mapped pointer range.
        /// Doing so may result in program termination.
        const Write = gl::MAP_WRITE_BIT;
        /// Read and write access.
        /// # Safety
        /// All accesses types are allowed within the range, but may be further restricted
        /// by the [`MapHint`].
        const ReadWrite = gl::MAP_READ_BIT | gl::MAP_WRITE_BIT;
    }
}
bitflags::bitflags! {
    /// Additional flags for mapping operations.
    #[repr(transparent)]
    pub struct RawMapHint: gl::types::GLbitfield {
        /// Discard the data within the range of the mapping.
        ///
        /// Only usable with Write-only access.
        ///
        /// # Safety
        /// Contents of the buffer within the range (and, consequently, the mapping)
        /// become undefined. Host or GL read accesses on undefined data are undefined behavior.
        /// Ensure the range is overwritten before any read access within the range.
        const InvalidateRange = gl::MAP_INVALIDATE_RANGE_BIT;
        /// Discard the data within the entire buffer.
        ///
        /// Only usable with Write-only access.
        ///
        /// # Safety
        /// Contents of the buffer (and, consequently, the mapping)
        /// become undefined. Host or GL read accesses on undefined data are undefined behavior.
        /// Ensure the entire buffer is overwritten before any read access within the buffer.
        const InvalidateBuffer = gl::MAP_INVALIDATE_BUFFER_BIT;
        /// Disable automatic flushing upon `unmap`.
        ///
        /// Mapping must have Write access.
        ///
        /// # Safety
        /// If modified but not flushed, the contents of the buffer in the range are undefined.
        const FlushExplicit = gl::MAP_FLUSH_EXPLICIT_BIT;
        /// Do not wait for any GL operations which may read or write the buffer to complete prior to
        /// mapping memory.
        ///
        /// Usable with Read and/or Write access.
        ///
        /// # Safety
        /// Data races abound. Do not cause a data race.
        const Unsynchronized = gl::MAP_UNSYNCHRONIZED_BIT;
    }
}

/// An application-owned memory buffer. Buffers simply represent a list of bytes,
/// who's interpretation is based wholly on the slot the buffer is bound to.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks resources"]
pub struct Buffer(pub(crate) NonZeroName);

impl crate::sealed::Sealed for Buffer {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl crate::ThinGLObject for Buffer {}
