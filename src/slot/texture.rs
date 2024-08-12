//! Binding and manipulating `Texture{2D, 2DArray, 3D, Cube}`.

use crate::{
    gl,
    state::CompareFunc,
    texture::{
        self, Cube, D2Array, Dimensionality, Filter, InternalFormat, Stateless, Swizzle, Texture,
        D2, D3,
    },
    GLEnum, GLenum, NonZero, NotSync,
};

/// Entry points for `glTex*`
pub struct Active<Dim: Dimensionality>(core::marker::PhantomData<Dim>);

impl<Dim: Dimensionality> Active<Dim> {
    unsafe fn tex_parameter_enum(pname: GLenum, param: GLenum) {
        gl::TexParameteri(Dim::TARGET, pname, param as _);
    }
    #[doc(alias = "glTexParameter")]
    #[doc(alias = "glTexParameteri")]
    #[doc(alias = "GL_TEXTURE_SWIZZLE")]
    #[doc(alias = "GL_TEXTURE_SWIZZLE_R")]
    #[doc(alias = "GL_TEXTURE_SWIZZLE_G")]
    #[doc(alias = "GL_TEXTURE_SWIZZLE_B")]
    #[doc(alias = "GL_TEXTURE_SWIZZLE_A")]
    pub fn swizzle(&mut self, swizzle: [Swizzle; 4]) -> &mut Self {
        let [r, g, b, a] = swizzle.map(|swizzle| swizzle.as_gl());
        unsafe {
            Self::tex_parameter_enum(gl::TEXTURE_SWIZZLE_R, r);
            Self::tex_parameter_enum(gl::TEXTURE_SWIZZLE_G, g);
            Self::tex_parameter_enum(gl::TEXTURE_SWIZZLE_B, b);
            Self::tex_parameter_enum(gl::TEXTURE_SWIZZLE_A, a);
        }
        self
    }
    #[doc(alias = "glTexParameter")]
    #[doc(alias = "glTexParameteri")]
    #[doc(alias = "GL_TEXTURE_MIN_FILTER")]
    pub fn min_filter(&mut self, texel: Filter, mip: Option<Filter>) -> &mut Self {
        let filter = match (texel, mip) {
            (Filter::Nearest, None) => gl::NEAREST,
            (Filter::Linear, None) => gl::LINEAR,
            (Filter::Nearest, Some(Filter::Nearest)) => gl::NEAREST_MIPMAP_NEAREST,
            (Filter::Nearest, Some(Filter::Linear)) => gl::NEAREST_MIPMAP_LINEAR,
            (Filter::Linear, Some(Filter::Nearest)) => gl::LINEAR_MIPMAP_NEAREST,
            (Filter::Linear, Some(Filter::Linear)) => gl::LINEAR_MIPMAP_LINEAR,
        };
        unsafe {
            Self::tex_parameter_enum(gl::TEXTURE_MIN_FILTER, filter);
        }
        self
    }
    #[doc(alias = "glTexParameter")]
    #[doc(alias = "glTexParameteri")]
    #[doc(alias = "GL_TEXTURE_MAG_FILTER")]
    pub fn mag_filter(&mut self, texel: Filter) -> &mut Self {
        let filter = match texel {
            Filter::Nearest => gl::NEAREST,
            Filter::Linear => gl::LINEAR,
        };
        unsafe {
            Self::tex_parameter_enum(gl::TEXTURE_MAG_FILTER, filter);
        }
        self
    }
    #[doc(alias = "glTexParameter")]
    #[doc(alias = "glTexParameteri")]
    #[doc(alias = "GL_TEXTURE_COMPARE_MODE")]
    #[doc(alias = "GL_TEXTURE_COMPARE_FUNC")]
    pub fn compare_mode(&mut self, mode: Option<CompareFunc>) -> &mut Self {
        if let Some(mode) = mode {
            unsafe {
                Self::tex_parameter_enum(gl::TEXTURE_COMPARE_MODE, gl::COMPARE_REF_TO_TEXTURE);
                Self::tex_parameter_enum(gl::TEXTURE_COMPARE_FUNC, mode.as_gl());
            }
        } else {
            unsafe {
                Self::tex_parameter_enum(gl::TEXTURE_COMPARE_MODE, gl::NONE);
            }
        }
        self
    }
    /// Specifies wrapping behavior in the X, Y, and Z dimensions, respectively.
    #[doc(alias = "glTexParameter")]
    #[doc(alias = "glTexParameteri")]
    #[doc(alias = "TEXTURE_WRAP_S")]
    #[doc(alias = "TEXTURE_WRAP_T")]
    #[doc(alias = "TEXTURE_WRAP_R")]
    pub fn wrap(&mut self, mode: [texture::Wrap; 3]) -> &mut Self {
        let [s, t, r] = mode.map(|mode| mode.as_gl());
        unsafe {
            Self::tex_parameter_enum(gl::TEXTURE_WRAP_S, s);
            Self::tex_parameter_enum(gl::TEXTURE_WRAP_T, t);
            Self::tex_parameter_enum(gl::TEXTURE_WRAP_R, r);
        }
        self
    }
    /// Hints to the GL the continuous range of mipmap levels that have defined contents.
    ///
    /// The range may extend beyond the number of levels of `self`, it is silently clamped
    /// during texture lookup.
    #[doc(alias = "glTexParameter")]
    #[doc(alias = "glTexParameteri")]
    #[doc(alias = "TEXTURE_BASE_LEVEL")]
    #[doc(alias = "TEXTURE_MAX_LEVEL")]
    pub fn level_range(&mut self, range: impl core::ops::RangeBounds<u32>) -> &mut Self {
        // Min, inclusive.
        let min = match range.start_bound() {
            core::ops::Bound::Unbounded => 0,
            core::ops::Bound::Excluded(&n) => n.saturating_add(1),
            core::ops::Bound::Included(&n) => n,
        };
        // Max, *also* inclusive!
        let max = match range.end_bound() {
            // This is the GL default *very big* mip number, lol
            core::ops::Bound::Unbounded => 1000,
            core::ops::Bound::Excluded(&n) => n.saturating_add(1),
            core::ops::Bound::Included(&n) => n,
        };

        unsafe {
            gl::TexParameteri(Dim::TARGET, gl::TEXTURE_BASE_LEVEL, min as _);
            gl::TexParameteri(Dim::TARGET, gl::TEXTURE_MAX_LEVEL, max as _);
        }
        self
    }
    /// Clamps sampler level-of-detail calculations to the given range.
    ///
    /// The range may extend beyond the number of levels of `self`, it is silently clamped
    /// during texture lookup.
    #[doc(alias = "glTexParameter")]
    #[doc(alias = "glTexParameterf")]
    #[doc(alias = "TEXTURE_MIN_LOD")]
    #[doc(alias = "TEXTURE_MAX_LOD")]
    pub fn lod_range(&mut self, range: core::ops::RangeInclusive<f32>) -> &mut Self {
        // would be nice if range was impl RangeBounds, but next_up/down isn't stable yet :V

        unsafe {
            gl::TexParameterf(Dim::TARGET, gl::TEXTURE_MIN_LOD, *range.start());
            gl::TexParameterf(Dim::TARGET, gl::TEXTURE_MAX_LOD, *range.end());
        }
        self
    }
    /// Set whether the Depth or the Stencil component is returned when sampling a combined
    /// depth-stencil texture.
    #[doc(alias = "glTexParameter")]
    #[doc(alias = "glTexParameteri")]
    #[doc(alias = "GL_DEPTH_STENCIL_TEXTURE_MODE")]
    pub fn depth_stencil_mode(&mut self, mode: crate::texture::DepthStencilMode) -> &mut Self {
        unsafe {
            Self::tex_parameter_enum(gl::DEPTH_STENCIL_TEXTURE_MODE, mode.as_gl());
        }
        self
    }
}

impl Active<D2> {
    #[doc(alias = "glTexStorage2D")]
    pub fn storage(
        &mut self,
        levels: NonZero<u32>,
        format: InternalFormat,
        width: NonZero<u32>,
        height: NonZero<u32>,
    ) -> &mut Self {
        unsafe {
            gl::TexStorage2D(
                D2::TARGET,
                levels.get().try_into().unwrap(),
                format.as_gl(),
                width.get().try_into().unwrap(),
                height.get().try_into().unwrap(),
            );
        };
        self
    }
}
pub struct Slot<Dim: Dimensionality>(pub(crate) NotSync, pub(crate) core::marker::PhantomData<Dim>);
impl<Dim: Dimensionality> Slot<Dim> {
    /// Bind a texture, returning an active token.
    #[doc(alias = "glBindTexture")]
    pub fn bind(&mut self, texture: &Texture<Dim>) -> &mut Active<Dim> {
        unsafe { gl::BindTexture(Dim::TARGET, texture.0.get()) };
        super::zst_mut()
    }
    /// Bind a stateless texture, turning it into a `Texture` with the dimensionality of this slot.
    #[doc(alias = "glBindTexture")]
    pub fn initialize(&mut self, texture: Stateless) -> (Texture<Dim>, &mut Active<Dim>) {
        // Transition the type to an initialized one
        let texture = Texture(texture.0, core::marker::PhantomData);
        // bind it!
        let bind = self.bind(&texture);
        (texture, bind)
    }
    /// Inherit the currently bound texture. This may be the default texture.
    #[must_use]
    pub fn inherit(&self) -> &Active<Dim> {
        super::zst_ref()
    }
    /// Inherit the currently bound texture. This may be the default texture.
    #[must_use]
    pub fn inherit_mut(&mut self) -> &mut Active<Dim> {
        super::zst_mut()
    }
    /// Delete textures. If any were bound to this slot, the slot becomes bound to the default texture.
    #[doc(alias = "glDeleteTextures")]
    pub fn delete<const N: usize>(&mut self, textures: [Texture<Dim>; N]) {
        unsafe { crate::gl_delete_with(gl::DeleteTextures, textures) }
    }
}

pub type Slot2D = Slot<D2>;
pub type Slot2DArray = Slot<D2Array>;
pub type Slot3D = Slot<D3>;
pub type SlotCube = Slot<Cube>;

/// Slots for binding textures. Corresponds to texture `glTex*` operations with `TEXTURE_{2D, 2D_ARRAY, 3D, CUBE_MAP}` targets.
pub struct Slots {
    /// `TEXTURE_2D`
    pub d2: Slot2D,
    /// `TEXTURE_3D`
    pub d3: Slot3D,
    /// `TEXTURE_2D_ARRAY`
    pub d2_array: Slot2DArray,
    /// `TEXTURE_CUBE_MAP`
    pub cube: SlotCube,
}
impl Slots {
    /// Set the currently active texture unit. Corresponds to `glActiveTexture(GL_TEXTURE<slot>)`
    ///
    /// Each texture unit has its own current textures for all bind points. As such,
    /// this invalidates all [`Active`] texture handles.
    #[doc(alias = "glActiveTexture")]
    pub fn unit(&mut self, slot: u32) -> &mut Self {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0.checked_add(slot).unwrap());
        }
        self
    }
    /// Delete textures. If any were bound to a slot, the slot becomes bound to the default texture.
    ///
    /// Use [`Into::into`] to convert textures into a deletion token. Alternatively, delete them
    /// through their relavent [`Slot`]s to narrow the scope of lost bindings.
    ///
    /// This is provided primarily for bulk texture deletion of mixed dimensionality.
    #[doc(alias = "glDeleteTextures")]
    pub fn delete<const N: usize>(&mut self, textures: [texture::DeletionToken; N]) {
        unsafe { crate::gl_delete_with(gl::DeleteTextures, textures) }
    }
}
