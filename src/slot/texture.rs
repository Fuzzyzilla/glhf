//! Slots for Texture2D, Texture3D, TextureCubeMap, and Texture2DArray.

use crate::{
    gl,
    state::CompareFunc,
    texture::{
        Cube, D2Array, Dimensionality, Filter, InternalFormat, Stateless, Swizzle, Texture, D2, D3,
    },
    GLEnum, GLenum, NonZero, NotSync,
};

/// Entry points for `glTex*`
pub struct Active<'slot, Dim: Dimensionality>(
    std::marker::PhantomData<&'slot ()>,
    std::marker::PhantomData<Dim>,
);

impl<Dim: Dimensionality> Active<'_, Dim> {
    unsafe fn tex_parameter_enum(pname: GLenum, param: GLenum) {
        gl::TexParameteri(Dim::TARGET, pname, param as _);
    }
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
}

impl Active<'_, D2> {
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
pub struct Slot<Dim: Dimensionality>(pub(crate) NotSync, pub(crate) std::marker::PhantomData<Dim>);
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
        let texture = Texture(texture.0, std::marker::PhantomData);
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
}
