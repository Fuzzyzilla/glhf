use crate::{
    gl::{self, types::GLenum},
    slot::marker::{IsDefault, NotDefault, Unknown},
    vertex_array::{self, VertexArray},
    GLEnum, NotSync, ThinGLObject,
};

// Note - GLES3.X techinically uses Default / NotDefault language here.
// This is provided for backwards compatibility with GLES2, where literal
// client pointers could be bound to object 0, yikes! We don't do this, and
// instead treat object 0 as null.

impl Active<'_, NotDefault> {
    /// Set the properties of a vertex attribute slot. The source buffer is remembered
    /// internally, and does not need to be active at time of draw.
    ///
    /// See [`vertex_array::Attribute`].
    ///
    /// `enable` is provided as a convinience - if set to Some, will enable or disable
    /// the attribute after setting properties. If set to none, no action is taken,
    /// effectively inheriting the previous state. By default, attributes are disabled.
    pub fn attribute(
        &self,
        _source: &super::buffer::Active<'_, super::buffer::Array, NotDefault>,
        index: u32,
        attribute: vertex_array::Attribute,
        enable: Option<bool>,
    ) -> &Self {
        use vertex_array::AttributeType;
        let size = attribute.components.into();
        let stride = attribute
            .stride
            .map_or(0, |stride| stride.get().try_into().unwrap());

        // Safety - hoooh boy...
        // This pointer type is interpreted as a numeric offset as long as there's a array buffer bound -
        // the interpretation of it as a pointer is all but deprecated (alas, still supported by GLES3.X).
        //
        // We don't allow this usage, but...
        // If the user somehow executes an arrayed draw with this vao bound while GL_ARRAY_BUFFER_BINDING is
        // 0 (null), this byte offset value will be interpreted as a host pointer and POOF everything explodes.
        let offset_pointer: *const std::ffi::c_void = attribute.offset as _;

        // `is_aligned_to`
        assert_eq!(
            (offset_pointer as usize) % (attribute.ty.align_of()),
            0,
            "attribute offset must be aligned"
        );

        // TODO: I think offset_pointer must be aligned according to `attribute.ty`. It is frustratingly
        // hard to find docs on alignment requirements...

        match attribute.ty {
            // ========== glVertexAttribIPointer
            AttributeType::Integer(ty) => unsafe {
                gl::VertexAttribIPointer(index, size, ty.as_gl(), stride, offset_pointer)
            },
            // ========== glVertexAttribPointer
            AttributeType::Float(ty) => unsafe {
                gl::VertexAttribPointer(index, size, ty.as_gl(), gl::FALSE, stride, offset_pointer)
            },
            // Scaled (normalized = false)
            AttributeType::PackedScaled(ty) => unsafe {
                gl::VertexAttribPointer(index, size, ty.as_gl(), gl::FALSE, stride, offset_pointer)
            },
            AttributeType::Scaled(ty) => unsafe {
                gl::VertexAttribPointer(index, size, ty.as_gl(), gl::FALSE, stride, offset_pointer)
            },
            // Normalized
            AttributeType::Normalized(ty) => unsafe {
                gl::VertexAttribPointer(index, size, ty.as_gl(), gl::TRUE, stride, offset_pointer)
            },
            AttributeType::PackedNormalized(ty) => unsafe {
                gl::VertexAttribPointer(index, size, ty.as_gl(), gl::TRUE, stride, offset_pointer)
            },
        }

        if let Some(enable) = enable {
            self.set_attribute_enabled(index, enable)
        } else {
            self
        }
    }
    /// Enable or disable the attribute at `index`. By default, all attributes are disabled.
    pub fn set_attribute_enabled(&self, index: u32, enabled: bool) -> &Self {
        if enabled {
            unsafe { gl::EnableVertexAttribArray(index) }
        } else {
            unsafe { gl::DisableVertexAttribArray(index) }
        }
        self
    }
}

/// Entry points for `gl*VertexAttrib*`.
pub struct Active<'slot, Kind>(
    std::marker::PhantomData<&'slot ()>,
    std::marker::PhantomData<Kind>,
);
pub struct Slot(pub(crate) NotSync);
impl Slot {
    /// Bind a user-defined array to this slot.
    pub fn bind(&mut self, array: &VertexArray) -> Active<NotDefault> {
        unsafe {
            gl::BindVertexArray(array.name().get());
        }
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Make the slot empty.
    pub fn unbind(&mut self) -> Active<IsDefault> {
        unsafe {
            gl::BindVertexArray(0);
        }
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Inherit the currently bound array - this may be no array at all.
    ///
    /// Most functionality is limited when the status of the array (Empty or NotEmpty) is not known.
    pub fn inherit(&self) -> Active<Unknown> {
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Delete vertex arrays. If any were bound to this slot, the slot becomes unbound.
    pub fn delete<const N: usize>(&mut self, arrays: [VertexArray; N]) {
        unsafe { crate::gl_delete_with(gl::DeleteVertexArrays, arrays) }
    }
}
