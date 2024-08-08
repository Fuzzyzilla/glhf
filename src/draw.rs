//! Entry points for executing draw commands.
//!
//! Drawing can trigger some of some of the most dire unsafety within the GL API.
//! There are some configurations which will cause the GL to misinterpret byte-offset
//! values as raw pointers, with predictably bad outcomes! The case where these values
//! are treated as pointers is a backwards compatibility feature not supported by this crate.
//!
//! To remedy this, this API is built such that you must provide compile-time proof that
//! configuration is properly set up.

use crate::slot::{self, marker};

type ActiveProgram<'a> = slot::program::Active<'a, marker::NotDefault>;
type ActiveVertexArray<'a> = slot::vertex_array::Active<'a, marker::NotDefault>;
type ActiveArray<'a> = slot::buffer::Active<'a, slot::buffer::Array, marker::NotDefault>;
type ActiveElementArray<'a> =
    slot::buffer::Active<'a, slot::buffer::ElementArray, marker::NotDefault>;
type ActiveDrawFramebuffer<'a, Defaultness> = slot::framebuffer::Active<
    'a,
    slot::framebuffer::Draw,
    Defaultness,
    crate::framebuffer::Complete,
>;

use super::{gl, GLEnum, NotSync};

#[repr(u32)]
pub enum Topology {
    Points = gl::POINTS,
    LineStrip = gl::LINE_STRIP,
    LineLoop = gl::LINE_LOOP,
    Lines = gl::LINES,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
    Triangles = gl::TRIANGLES,
}
// Safety: is repr(u32) enum.
unsafe impl GLEnum for Topology {}

/// Specifies the datatype of indices to fetch from the ElementArray.
#[repr(u32)]
pub enum ElementType {
    U8 = gl::UNSIGNED_BYTE,
    U16 = gl::UNSIGNED_SHORT,
    U32 = gl::UNSIGNED_INT,
}
// Safety: is repr(u32) enum.
unsafe impl GLEnum for ElementType {}

impl ElementType {
    pub fn size_of(&self) -> usize {
        match self {
            Self::U8 => std::mem::size_of::<u8>(),
            Self::U16 => std::mem::size_of::<u16>(),
            Self::U32 => std::mem::size_of::<u32>(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct ArrayState<'a, Default: marker::Defaultness> {
    /// Static proof that a non-null Vertex Array is bound.
    pub vertex_array: &'a ActiveVertexArray<'a>,
    /// Static proof that a Complete framebuffer is bound.
    pub framebuffer: &'a ActiveDrawFramebuffer<'a, Default>,
    /// Static proof that a successfully-linked program is bound.
    pub program: &'a ActiveProgram<'a>,
}

#[derive(Copy, Clone)]
pub struct ElementState<'a, Default: marker::Defaultness> {
    /// Static proof that a non-null Element Array is bound.
    pub elements: &'a ActiveElementArray<'a>,
    /// Static proof that a non-null Vertex Array is bound.
    pub vertex_array: &'a ActiveVertexArray<'a>,
    /// Static proof that a Complete framebuffer is bound.
    pub framebuffer: &'a ActiveDrawFramebuffer<'a, Default>,
    /// Static proof that a successfully-linked program is bound.
    pub program: &'a ActiveProgram<'a>,
}

/// Bindings to `glDraw*`
pub struct Draw(pub(crate) NotSync);

impl Draw {
    /// Draw consecutive vertices from the [vertex array](ArrayState::vertex_array),
    /// using its enabled buffers and attributes.
    pub fn arrays<Default: marker::Defaultness>(
        &self,
        mode: Topology,
        vertices: std::ops::Range<usize>,
        instances: usize,
        _state: ArrayState<Default>,
    ) {
        if vertices.start == vertices.end || instances == 0 {
            // Nothing to draw.
            return;
        }

        let count = vertices
            .end
            .checked_sub(vertices.start)
            .expect("draw range end before start");

        if instances == 1 {
            // AFAIK, treating instances == 1 as a regular draw is not observably different
            // from an actual instanced call with count = 1.
            unsafe {
                gl::DrawArrays(
                    mode.as_gl(),
                    vertices.start.try_into().unwrap(),
                    count.try_into().unwrap(),
                );
            }
        } else {
            unsafe {
                gl::DrawArraysInstanced(
                    mode.as_gl(),
                    vertices.start.try_into().unwrap(),
                    count.try_into().unwrap(),
                    instances.try_into().unwrap(),
                );
            }
        }
    }
    /// Fetches the indices to draw from the bound [element buffer](ElementState::elements),
    /// and uses those to fetch to vertices from the [vertex array](ElementState::vertex_array).
    pub fn elements<Default: marker::Defaultness>(
        &self,
        mode: Topology,
        element_type: ElementType,
        elements: std::ops::Range<usize>,
        instances: usize,
        _state: ElementState<Default>,
    ) {
        if elements.start == elements.end || instances == 0 {
            // Nothing to draw.
            return;
        }

        let count = elements
            .end
            .checked_sub(elements.start)
            .expect("draw range end before start");

        let byte_offset = elements.start.checked_mul(element_type.size_of()).unwrap();

        if instances == 1 {
            // AFAIK, treating instances == 1 as a regular draw is not observably different
            // from an actual instanced call with count = 1.
            unsafe {
                gl::DrawElements(
                    mode.as_gl(),
                    count.try_into().unwrap(),
                    element_type.as_gl(),
                    // Bigggg unsafe here. This is a byte offset, but if there is no
                    // element array bound, *it will be treated as a client pointer* - yikes.
                    // `_state` ensures we have an element buffer bound at time of call.
                    byte_offset as _,
                );
            }
        } else {
            unsafe {
                gl::DrawElementsInstanced(
                    mode.as_gl(),
                    count.try_into().unwrap(),
                    element_type.as_gl(),
                    byte_offset as _,
                    instances.try_into().unwrap(),
                )
            }
        }
    }
    /// Fetches the indices to draw from the bound [element buffer](ElementState::elements),
    /// and uses those to fetch to vertices from the [vertex array](ElementState::vertex_array),
    /// additionally assuming that the indices fetched lie within `index_range`.
    ///
    /// This allows the implementation to perform optimized memory prefetching and
    /// ahead-of-time computation. For maximum performance, the range should be as small as possible with
    /// minimal unused indices.
    ///
    /// # Safety
    /// All index values in the range given by `elements` within the element buffer must be within `index_range`.
    pub unsafe fn ranged_elements<Default: marker::Defaultness>(
        &self,
        mode: Topology,
        element_type: ElementType,
        elements: std::ops::Range<usize>,
        index_range: std::ops::RangeInclusive<usize>,
        _state: ElementState<Default>,
    ) {
        if elements.start == elements.end {
            // Nothing to draw.
            return;
        }

        let count = elements
            .end
            .checked_sub(elements.start)
            .expect("draw range end before start");

        let byte_offset = elements.start.checked_mul(element_type.size_of()).unwrap();

        // (why is there no Instanced form?)
        unsafe {
            gl::DrawRangeElements(
                mode.as_gl(),
                (*index_range.start()).try_into().unwrap(),
                (*index_range.end()).try_into().unwrap(),
                count.try_into().unwrap(),
                element_type.as_gl(),
                byte_offset as _,
            );
        }
    }
}
