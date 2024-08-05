//! Entry points for executing draw commands.
//!
//! Drawing can trigger some of some of the most dire unsafety within the GL API.
//! There are some configurations which will cause the GL to misinterpret byte-offset
//! values as raw pointers, with predictably bad outcomes! The case where these values
//! are treated as pointers is a backwards compatibility feature not supported by this crate.
//!
//! To remedy this, this API is built such that you must provide compile-time proof that
//! configuration is properly set up.

use crate::slot;

type ActiveProgram<'a> = slot::program::Active<'a, slot::program::NotEmpty>;
type ActiveVertexArray<'a> = slot::vertex_array::Active<'a, slot::vertex_array::NotEmpty>;
type ActiveArray<'a> = slot::buffer::Active<'a, slot::buffer::Array, slot::buffer::NotEmpty>;
type ActiveElementArray<'a> =
    slot::buffer::Active<'a, slot::buffer::ElementArray, slot::buffer::NotEmpty>;
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

#[derive(Copy, Clone)]
pub struct ArraysState<'a, Defaultness> {
    pub array: &'a ActiveArray<'a>,
    pub vertex_array: &'a ActiveVertexArray<'a>,
    pub framebuffer: &'a ActiveDrawFramebuffer<'a, Defaultness>,
    pub program: &'a ActiveProgram<'a>,
}

#[derive(Copy, Clone)]
pub struct ElementsState<'a, Defaultness> {
    pub array: &'a ActiveArray<'a>,
    pub elements: &'a ActiveElementArray<'a>,
    pub vertex_array: &'a ActiveVertexArray<'a>,
    pub framebuffer: &'a ActiveDrawFramebuffer<'a, Defaultness>,
    pub program: &'a ActiveProgram<'a>,
}

/// Bindings to `glDraw*`
pub struct Draw(pub(crate) NotSync);

impl Draw {
    /// Draw vertices from the `VertexArray`, using its enabled attributes.
    pub fn arrays<Defaultness>(
        &self,
        mode: Topology,
        vertices: std::ops::Range<usize>,
        instances: usize,
        _state: ArraysState<Defaultness>,
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
    /// Fetches the indices to draw from the bound `ElementBuffer`,
    /// and uses those to fetch to vertices from the `VertexArray`.
    pub fn elements<Defaultness>(
        &self,
        mode: Topology,
        element_type: ElementType,
        elements: std::ops::Range<usize>,
        instances: usize,
        _state: ElementsState<Defaultness>,
    ) {
        if elements.start == elements.end || instances == 0 {
            // Nothing to draw.
            return;
        }

        let count = elements
            .end
            .checked_sub(elements.start)
            .expect("draw range end before start");

        let byte_offset = count
            * match element_type {
                ElementType::U8 => std::mem::size_of::<u8>(),
                ElementType::U16 => std::mem::size_of::<u16>(),
                ElementType::U32 => std::mem::size_of::<u32>(),
            };

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
    /// Fetches the indices to draw from the bound `ElementBuffer`,
    /// and uses those to fetch to vertices from the `VertexArray`,
    /// additionally assuming that the indices fetched lie within `index_range`.
    ///
    /// This allows the implementation to perform optimized memory prefetching and
    /// ahead-of-time computation.
    ///
    /// # Safety
    /// All index values in the range given by `elements` within `ElementBuffer` must be within `index_range`.
    pub unsafe fn ranged_elements<Defaultness>(
        &self,
        mode: Topology,
        element_type: ElementType,
        elements: std::ops::Range<usize>,
        index_range: std::ops::RangeInclusive<usize>,
        _state: ElementsState<Defaultness>,
    ) {
        if elements.start == elements.end {
            // Nothing to draw.
            return;
        }

        let count = elements
            .end
            .checked_sub(elements.start)
            .expect("draw range end before start");

        let byte_offset = count
            * match element_type {
                ElementType::U8 => std::mem::size_of::<u8>(),
                ElementType::U16 => std::mem::size_of::<u16>(),
                ElementType::U32 => std::mem::size_of::<u32>(),
            };

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
