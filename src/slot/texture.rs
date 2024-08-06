//! Slots for Texture2D, Texture3D, TextureCubeMap, and Texture2DArray.

use crate::{
    gl,
    texture::{
        Cube, D2Array, Dimensionality, Filter, InternalFormat, Stateless, Swizzle, Texture, D2, D3,
    },
    DepthCompareFunc, GLEnum, GLenum, NonZero, NotSync,
};

/// Entry points for `glTex*`
pub struct Active<'slot, Dim: Dimensionality>(
    std::marker::PhantomData<&'slot ()>,
    std::marker::PhantomData<Dim>,
);

impl<Dim: Dimensionality> Active<'_, Dim> {
    unsafe fn tex_parameter_enum(&self, pname: GLenum, param: GLenum) {
        gl::TexParameteri(Dim::TARGET, pname, param as _);
    }
    pub fn swizzle(&self, swizzle: [Swizzle; 4]) -> &Self {
        let [r, g, b, a] = swizzle.map(|swizzle| swizzle.as_gl());
        unsafe {
            self.tex_parameter_enum(gl::TEXTURE_SWIZZLE_R, r);
            self.tex_parameter_enum(gl::TEXTURE_SWIZZLE_G, g);
            self.tex_parameter_enum(gl::TEXTURE_SWIZZLE_B, b);
            self.tex_parameter_enum(gl::TEXTURE_SWIZZLE_A, a);
        }
        self
    }
    pub fn min_filter(&self, texel: Filter, mip: Option<Filter>) -> &Self {
        let filter = match (texel, mip) {
            (Filter::Nearest, None) => gl::NEAREST,
            (Filter::Linear, None) => gl::LINEAR,
            (Filter::Nearest, Some(Filter::Nearest)) => gl::NEAREST_MIPMAP_NEAREST,
            (Filter::Nearest, Some(Filter::Linear)) => gl::NEAREST_MIPMAP_LINEAR,
            (Filter::Linear, Some(Filter::Nearest)) => gl::LINEAR_MIPMAP_NEAREST,
            (Filter::Linear, Some(Filter::Linear)) => gl::LINEAR_MIPMAP_LINEAR,
        };
        unsafe {
            self.tex_parameter_enum(gl::TEXTURE_MIN_FILTER, filter);
        }
        self
    }
    pub fn mag_filter(&self, texel: Filter) -> &Self {
        let filter = match texel {
            Filter::Nearest => gl::NEAREST,
            Filter::Linear => gl::LINEAR,
        };
        unsafe {
            self.tex_parameter_enum(gl::TEXTURE_MAG_FILTER, filter);
        }
        self
    }
    pub fn compare_mode(&self, mode: Option<DepthCompareFunc>) -> &Self {
        if let Some(mode) = mode {
            unsafe {
                self.tex_parameter_enum(gl::TEXTURE_COMPARE_MODE, gl::COMPARE_REF_TO_TEXTURE);
                self.tex_parameter_enum(gl::TEXTURE_COMPARE_FUNC, mode.as_gl());
            }
        } else {
            unsafe {
                self.tex_parameter_enum(gl::TEXTURE_COMPARE_MODE, gl::NONE);
            }
        }
        self
    }
}

impl Active<'_, D2> {
    pub fn storage(
        &self,
        levels: NonZero<u32>,
        format: InternalFormat,
        width: NonZero<u32>,
        height: NonZero<u32>,
    ) -> &Self {
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
    pub fn bind(&mut self, texture: &Texture<Dim>) -> Active<Dim> {
        unsafe { gl::BindTexture(Dim::TARGET, texture.0.get()) };
        self.inherit()
    }
    /// Bind a stateless texture, turning it into a `Texture` with the dimensionality of this slot.
    pub fn initialize(&mut self, texture: Stateless) -> (Texture<Dim>, Active<Dim>) {
        // Transition the type to an initialized one
        let texture = Texture(texture.0, std::marker::PhantomData);
        // bind it!
        let bind = self.bind(&texture);
        (texture, bind)
    }
    /// Inherit the currently bound texture. This may be the default texture.
    pub fn inherit(&self) -> Active<Dim> {
        Active(std::marker::PhantomData, std::marker::PhantomData)
    }
    /// Delete textures. If any were bound to this slot, the slot becomes bound to the default texture.
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
    pub fn unit(&mut self, slot: u32) -> &mut Self {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0.checked_add(slot).unwrap());
        }
        self
    }
}
