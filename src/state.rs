use super::{gl, GLEnum, NotSync};

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Color { r, g, b, a }
    }
}

pub struct ColorMask {
    pub r: bool,
    pub g: bool,
    pub b: bool,
    pub a: bool,
}
impl From<[bool; 4]> for ColorMask {
    fn from([r, g, b, a]: [bool; 4]) -> Self {
        ColorMask { r, g, b, a }
    }
}
impl From<bool> for ColorMask {
    fn from(value: bool) -> Self {
        ColorMask {
            r: value,
            g: value,
            b: value,
            a: value,
        }
    }
}

#[repr(u32)]
pub enum CompareFunc {
    LessEqual = gl::LEQUAL,
    GreaterEqual = gl::GEQUAL,
    Less = gl::LESS,
    Greater = gl::GREATER,
    Equal = gl::EQUAL,
    NotEqual = gl::NOTEQUAL,
    Always = gl::ALWAYS,
    Never = gl::NEVER,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for CompareFunc {}

#[repr(u32)]
pub enum CullFace {
    Front = gl::FRONT,
    Back = gl::BACK,
    /// All polygons are culled. Lines and points are not, as they do not
    /// have a facing.
    FrontAndBack = gl::FRONT_AND_BACK,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for CullFace {}

#[repr(u32)]
pub enum FrontFace {
    Clockwise = gl::CW,
    CounterClockwise = gl::CCW,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for FrontFace {}

#[repr(u32)]
pub enum BlendEquation {
    /// `(src * factor) + (dst * factor)`
    Add = gl::FUNC_ADD,
    /// `(src * factor) - (dst * factor)`
    Subtract = gl::FUNC_SUBTRACT,
    /// `(dst * factor) - (src * factor)`
    ReverseSubtract = gl::FUNC_REVERSE_SUBTRACT,
    /// `min(src, dst)`. *Note*: multiply factors are not used.
    Min = gl::MIN,
    /// `max(src, dst)`. *Note*: multiply factors are not used.
    Max = gl::MAX,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for BlendEquation {}

#[repr(u32)]
pub enum BlendFactor {
    Zero = gl::ZERO,
    One = gl::ONE,

    SrcColor = gl::SRC_COLOR,
    OneMinusSrcColor = gl::ONE_MINUS_SRC_COLOR,
    SrcAlpha = gl::SRC_ALPHA,
    OneMinusSrcAlpha = gl::ONE_MINUS_SRC_ALPHA,

    DstColor = gl::DST_COLOR,
    OneMinusDstColor = gl::ONE_MINUS_DST_COLOR,
    DstAlpha = gl::DST_ALPHA,
    OneMinusDstAlpha = gl::ONE_MINUS_DST_ALPHA,

    ConstantColor = gl::CONSTANT_COLOR,
    OneMinusConstantColor = gl::ONE_MINUS_CONSTANT_COLOR,
    ConstantAlpha = gl::CONSTANT_ALPHA,
    OneMinusConstantAlpha = gl::ONE_MINUS_CONSTANT_ALPHA,

    SrcAlphaSaturate = gl::SRC_ALPHA_SATURATE,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for BlendFactor {}

pub struct BlendFunc {
    src_factor: BlendFactor,
    dst_factor: BlendFactor,
}

/// Arguments to `gl{Enable, Disable}`.
#[repr(u32)]
pub enum Capability {
    /// Blending using the user-defined blend equation and factors. If disabled,
    /// attachment colors are written as-is from fragment outputs.
    ///
    /// See [`State::blend_func`], [`State::blend_equation`].
    Blend = gl::BLEND,
    /// Discarding of polygons based on their facing, in framebuffer space.
    ///
    /// See [`State::cull_face`], [`State::front_face`].
    CullFace = gl::CULL_FACE,
    /// Generation of helpful debug messages, especially in debug contexts.
    DebugOutput = gl::DEBUG_OUTPUT,
    /// Whether [debug output](Capability::DebugOutput) should occur immediately within the callstack of the
    /// GL function which produced it. If not enabled, messages from a call may be arbitrarily deferred, and may
    /// even occur on a separate thread.
    DebugOutputSynchronous = gl::DEBUG_OUTPUT_SYNCHRONOUS,
    /// Update and test fragments against the depth buffer.
    ///
    /// See [`State::depth_func`], [`State::depth_mask`].
    DepthTest = gl::DEPTH_TEST,
    /// Framebuffer colors should be dithered to give the illusion of greater color accuracy.
    ///
    /// This effects `Clear` commands.
    Dither = gl::DITHER,
    /// Polygon depth offset values should be applied to fragments.
    ///
    /// See [`State::polygon_offset`].
    PolygonOffsetFill = gl::POLYGON_OFFSET_FILL,
    /// The special index value [`crate::draw::ElementType`]`::MAX` should restart `*Strip` and `*Loop`
    /// [primitive modes](crate::draw::Topology).
    PrimitiveRestartFixedIndex = gl::PRIMITIVE_RESTART_FIXED_INDEX,
    /// Discard processed geometry immediately before the rasterization state.
    /// Fragments are not executed, but transform feedback may still be acquired.
    ///
    /// This includes `Clear` commands.
    RasterizerDiscard = gl::RASTERIZER_DISCARD,
    /// Bitwise `AND` the fragment coverage value with a temporary mask based on the
    /// alpha of each sample. This can be used for cheap approximate order-independent transparency.
    SampleAlphaToCoverage = gl::SAMPLE_ALPHA_TO_COVERAGE,
    /// Bitwise `AND` the fragment coverage value with a user-defined mask.
    SampleMask = gl::SAMPLE_MASK,
    /// Discard fragments outside of the scissor rectangle.
    ///
    /// This effects `Clear` commands.
    ScissorTest = gl::SCISSOR_TEST,
    /// Update and test fragments against the stencil buffer.
    ///
    /// See [`State::stencil_func`], [`State::stencil_op`], [`State::stencil_mask`]
    StencilTest = gl::STENCIL_TEST,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for Capability {}

#[repr(u32)]
pub enum StencilOp {
    Keep = gl::KEEP,
    Zero = gl::ZERO,
    /// Write the reference value.
    Replace = gl::REPLACE,

    SaturatingIncrement = gl::INCR,
    WrappingIncrement = gl::INCR_WRAP,
    SaturatingDecrement = gl::DECR,
    WrappingDecrement = gl::DECR_WRAP,

    /// Bitwise `NOT` the current value.
    Invert = gl::INVERT,
}
// Safety: is repr(u32) enum.
unsafe impl crate::GLEnum for StencilOp {}

/// Read and write global state.
pub struct State(pub(crate) NotSync);
impl State {
    /// Set the blend constant. Values are not clamped at a global level, but
    /// are clamped during blending when the destination buffer is an unsigned fixed-point format.
    pub fn blend_color(&self, color: impl Into<Color>) -> &Self {
        let color = color.into();
        unsafe {
            gl::BlendColor(color.r, color.g, color.b, color.a);
        }
        self
    }
    /// Set the function used to combine source and destination colors.
    /// If `alpha_equation` is Some, separate equations are used for RGB and A. Otherwise, `equation`
    /// is used for all components.
    pub fn blend_equation(
        &self,
        equation: BlendEquation,
        alpha_equation: Option<BlendEquation>,
    ) -> &Self {
        if let Some(alpha_equation) = alpha_equation {
            unsafe {
                gl::BlendEquationSeparate(equation.as_gl(), alpha_equation.as_gl());
            }
        } else {
            unsafe {
                gl::BlendEquation(equation.as_gl());
            }
        }
        self
    }
    /// Set the multiplicative factors used to scale source and destination colors before
    /// being combined in the blend equation.
    /// If `alpha_func` is Some, separate factors are used for RGB and A. Otherwise, `func`
    /// is used for all components.
    pub fn blend_func(&self, func: BlendFunc, alpha_func: Option<BlendFunc>) -> &Self {
        if let Some(alpha_func) = alpha_func {
            unsafe {
                gl::BlendFuncSeparate(
                    func.src_factor.as_gl(),
                    func.dst_factor.as_gl(),
                    alpha_func.src_factor.as_gl(),
                    alpha_func.dst_factor.as_gl(),
                );
            }
        } else {
            unsafe {
                gl::BlendFunc(func.src_factor.as_gl(), func.dst_factor.as_gl());
            }
        }
        self
    }
    /// What color value to clear color buffers to in a `glClear`.
    pub fn clear_color(&self, color: impl Into<Color>) -> &Self {
        let color = color.into();
        unsafe {
            gl::ClearColor(color.r, color.g, color.b, color.a);
        }
        self
    }
    /// What floating point value to clear the depth buffer to in a `glClear`.
    pub fn clear_depth(&self, depth: f32) -> &Self {
        unsafe {
            gl::ClearDepthf(depth);
        }
        self
    }
    /// What bit value to clear the stencil buffer to in a `glClear`.
    pub fn clear_stencil(&self, stencil: u32) -> &Self {
        unsafe {
            // it's an unsigned value but C apis are allergic to uint i think
            gl::ClearStencil(stencil as _)
        }
        self
    }
    /// Enable or disable writes to color channels of all buffers.
    /// E.g., if `r` is `false`, drawing operations will not affect any red channels.
    ///
    /// This effects `Clear` commands.
    // Todo: is this a framebuffer or global up?
    pub fn color_mask(&self, write: impl Into<ColorMask>) -> &Self {
        let write = write.into();
        unsafe {
            // it's an unsigned value but C apis are allergic to uint i think
            gl::ColorMask(
                write.r.into(),
                write.g.into(),
                write.b.into(),
                write.a.into(),
            );
        }
        self
    }
    /// Which polygon faces to cull when [`Capability::CullFace`] is enabled
    pub fn cull_face(&self, face: CullFace) -> &Self {
        unsafe {
            gl::CullFace(face.as_gl());
        }
        self
    }
    /// The function used to check a fragment's depth against the depth buffer.
    pub fn depth_func(&self, func: CompareFunc) -> &Self {
        unsafe {
            gl::DepthFunc(func.as_gl());
        }
        self
    }
    /// Whether fragments that pass the fragment test should write to the depth buffer.
    ///
    /// This effects `Clear` commands.
    pub fn depth_mask(&self, write: bool) -> &Self {
        unsafe {
            gl::DepthMask(write.into());
        }
        self
    }
    /// Defines a linear mapping from [-1, 1] NDC space to `range` in depth map space.
    /// Range may be reversed, i.e. `1.0..=-1.0` is a valid range.
    pub fn depth_rangef(&self, range: std::ops::RangeInclusive<f32>) -> &Self {
        unsafe {
            gl::DepthRangef(*range.start(), *range.end());
        }
        self
    }
    /// Disable a capability. See [`Capability`] for info.
    pub fn disable(&self, capability: Capability) -> &Self {
        unsafe {
            gl::Disable(capability.as_gl());
        }
        self
    }
    /// Enable a capability. See [`Capability`] for info.
    pub fn enable(&self, capability: Capability) -> &Self {
        unsafe {
            gl::Enable(capability.as_gl());
        }
        self
    }
    /// Defines what winding order, in framebuffer space, is consindered the "front" of a polygon.
    pub fn front_face(&self, winding: FrontFace) -> &Self {
        unsafe {
            gl::FrontFace(winding.as_gl());
        }
        self
    }
    pub fn line_width(&self, width: f32) -> &Self {
        unsafe {
            gl::LineWidth(width);
        }
        self
    }
    pub fn polygon_offset(&self, factor: f32, units: f32) -> &Self {
        unsafe {
            gl::PolygonOffset(factor, units);
        }
        self
    }
    pub fn sample_coverage(&self, value: f32, invert: bool) -> &Self {
        unsafe {
            gl::SampleCoverage(value, invert.into());
        }
        self
    }
    /// Specify the scissor rectangle for scissor testing, if enabled.
    ///
    /// `min` is the lower-left.
    pub fn scissor(&self, min: [u32; 2], size: [u32; 2]) -> &Self {
        unsafe {
            gl::Scissor(
                min[0].try_into().unwrap(),
                min[1].try_into().unwrap(),
                size[0].try_into().unwrap(),
                size[1].try_into().unwrap(),
            );
        }
        self
    }
    /// Specify the conditions for passing the stencil check.
    ///
    /// For example, if func is [`CompareFunc::GreaterEqual`], the check is
    /// `(reference & mask) >= (stencil & mask)`
    pub fn stencil_func(&self, func: CompareFunc, reference: u32, mask: u32) -> &Self {
        unsafe {
            gl::StencilFunc(func.as_gl(), reference as _, mask);
        }
        self
    }
    /// Specify write-protection of bits within the stencil mask.
    /// Where a 1 appears, the corresponding stencil bit is writable, where a 0 appears,
    /// it is read-only.
    ///
    /// This affects `Clear` commands.
    pub fn stencil_mask(&self, mask: u32) -> &Self {
        unsafe {
            gl::StencilMask(mask as _);
        }
        self
    }
    /// Specify the modifications to make to the stencil buffer when the stencil
    /// test fails, the depth test fails, or neither test fails, respectively.
    pub fn stencil_op(
        &self,
        stencil_fail: StencilOp,
        depth_fail: StencilOp,
        pass: StencilOp,
    ) -> &Self {
        unsafe {
            gl::StencilOp(stencil_fail.as_gl(), depth_fail.as_gl(), pass.as_gl());
        }
        self
    }
    /// Specifies the transform from NDC space to framebuffer space.
    /// The vertex x and y output ranges of `[-1, 1]` are mapped onto this rectangle.
    ///
    /// `min` is the lower-left.
    pub fn viewport(&self, min: [u32; 2], size: [u32; 2]) -> &Self {
        unsafe {
            gl::Viewport(
                min[0].try_into().unwrap(),
                min[1].try_into().unwrap(),
                size[0].try_into().unwrap(),
                size[1].try_into().unwrap(),
            );
        }
        self
    }
}
