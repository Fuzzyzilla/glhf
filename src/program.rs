//! Types and parameter enums for Shaders and Programs.
use crate::ThinGLObject;

use super::{gl, GLenum, NonZeroName};

/// Types for uniform variables
pub mod uniform {
    pub enum Ty {
        U32,
        I32,
        F32,
    }
    /// Marker trait for types which may be used in arguments to `glUniform*` calls.
    /// # Safety
    /// Only implement for types which are wholly represented by the type corresponding to `Ty`.
    pub unsafe trait Value: crate::sealed::Sealed + 'static {
        const TYPE: Ty;
    }
    unsafe impl Value for f32 {
        const TYPE: Ty = Ty::F32;
    }
    impl crate::sealed::Sealed for f32 {}
    unsafe impl Value for i32 {
        const TYPE: Ty = Ty::I32;
    }
    impl crate::sealed::Sealed for i32 {}
    unsafe impl Value for u32 {
        const TYPE: Ty = Ty::U32;
    }
    impl crate::sealed::Sealed for u32 {}

    #[repr(C)]
    pub struct Vec2<T: Value>(pub [T; 2]);
    #[repr(C)]
    pub struct Vec3<T: Value>(pub [T; 3]);
    #[repr(C)]
    pub struct Vec4<T: Value>(pub [T; 4]);

    macro_rules! matrix {
        (pub struct $name:ident(pub $ty:ty)) => {
            #[repr(C)]
            pub struct $name(pub $ty);
            impl ::core::convert::From<$ty> for $name {
                fn from(value: $ty) -> Self {
                    Self(value)
                }
            }
            impl ::core::convert::From<$name> for $ty {
                fn from(value: $name) -> Self {
                    value.0
                }
            }
        };
    }
    // GLES doesn't support [ui]mat? weird.
    matrix!(pub struct Mat2(pub [[f32; 2]; 2]));
    matrix!(pub struct Mat3(pub [[f32; 3]; 3]));
    matrix!(pub struct Mat4(pub [[f32; 4]; 4]));
    matrix!(pub struct Mat2x3(pub [[f32; 3]; 2]));
    matrix!(pub struct Mat2x4(pub [[f32; 4]; 2]));
    matrix!(pub struct Mat3x2(pub [[f32; 2]; 3]));
    matrix!(pub struct Mat3x4(pub [[f32; 4]; 3]));
    matrix!(pub struct Mat4x3(pub [[f32; 3]; 4]));

    /// Value for a matrix uniform.
    /// If the uniform is not an array, the slice should have one element.
    pub enum Matrix<'a> {
        Mat2(&'a [Mat2]),
        Mat3(&'a [Mat3]),
        Mat4(&'a [Mat4]),
        Mat2x3(&'a [Mat2x3]),
        Mat2x4(&'a [Mat2x4]),
        Mat3x2(&'a [Mat3x2]),
        Mat3x4(&'a [Mat3x4]),
        Mat4x3(&'a [Mat4x3]),
    }
    impl Matrix<'_> {
        #[must_use]
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }
        #[must_use]
        pub fn len(&self) -> usize {
            match self {
                Self::Mat2(s) => s.len(),
                Self::Mat3(s) => s.len(),
                Self::Mat4(s) => s.len(),
                Self::Mat2x3(s) => s.len(),
                Self::Mat3x2(s) => s.len(),
                Self::Mat4x3(s) => s.len(),
                Self::Mat2x4(s) => s.len(),
                Self::Mat3x4(s) => s.len(),
            }
        }
        /// Get the number of locations consumed by this array.
        #[must_use]
        pub fn locations(&self) -> usize {
            let columns = match self {
                Self::Mat2(_) => 2,
                Self::Mat3(_) => 3,
                Self::Mat4(_) => 4,
                Self::Mat2x3(_) => 2,
                Self::Mat3x2(_) => 3,
                Self::Mat4x3(_) => 4,
                Self::Mat2x4(_) => 2,
                Self::Mat3x4(_) => 3,
            };
            self.len() * columns
        }
    }

    macro_rules! matrix_froms {
        {$from:tt} => {
            impl<'a> ::core::convert::From<&'a $from> for Matrix<'a> {
                fn from(value: &'a $from) -> Self {
                    Self::$from(::core::slice::from_ref(value))
                }
            }
            impl<'a> ::core::convert::From<&'a [$from]> for Matrix<'a> {
                fn from(value: &'a [$from]) -> Self {
                    Self::$from(value)
                }
            }
        }
    }

    matrix_froms!(Mat2);
    matrix_froms!(Mat3);
    matrix_froms!(Mat4);
    matrix_froms!(Mat2x3);
    matrix_froms!(Mat2x4);
    matrix_froms!(Mat3x2);
    matrix_froms!(Mat3x4);
    matrix_froms!(Mat4x3);

    /// Value for a non-matrix uniform.
    /// If the uniform is not an array, the slice should have one element.
    pub enum Vector<'a, T: Value> {
        // Ironic to have this in an enum called "Vector", huh?
        /// Scalar value(s).
        ///
        /// `i32` is a special scalar type which may be used to bind some opaque
        /// objects, e.g. `sampler2D`.
        ///
        /// It is not valid to pass e.g. 4 scalars for a Vec4 value.
        Scalar(&'a [T]),
        Vec2(&'a [Vec2<T>]),
        Vec3(&'a [Vec3<T>]),
        Vec4(&'a [Vec4<T>]),
    }
    impl<T: Value> Vector<'_, T> {
        #[must_use]
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }
        #[must_use]
        pub fn len(&self) -> usize {
            match self {
                Self::Scalar(s) => s.len(),
                Self::Vec2(s) => s.len(),
                Self::Vec3(s) => s.len(),
                Self::Vec4(s) => s.len(),
            }
        }
        /// Get the number of locations consumed by this array.
        #[must_use]
        pub fn locations(&self) -> usize {
            self.len()
        }
    }

    macro_rules! vector_froms {
        {$from:tt} => {
            impl<'a, T: crate::program::uniform::Value> ::core::convert::From<&'a $from<T>> for Vector<'a, T> {
                fn from(value: &'a $from<T>) -> Self {
                    Self::$from(::core::slice::from_ref(value))
                }
            }
            impl<'a, T: crate::program::uniform::Value> ::core::convert::From<&'a [$from<T>]> for Vector<'a, T> {
                fn from(value: &'a [$from<T>]) -> Self {
                    Self::$from(value)
                }
            }
        }
    }

    impl<'a, T: Value> From<&'a T> for Vector<'a, T> {
        fn from(value: &'a T) -> Self {
            Self::Scalar(core::slice::from_ref(value))
        }
    }
    impl<'a, T: Value> From<&'a [T]> for Vector<'a, T> {
        fn from(value: &'a [T]) -> Self {
            Self::Scalar(value)
        }
    }
    vector_froms!(Vec2);
    vector_froms!(Vec3);
    vector_froms!(Vec4);
}

/// Marker trait for the many shader targets.
pub trait Type: crate::sealed::Sealed {
    const TYPE: GLenum;
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
        impl Type for $marker {
            const TYPE: GLenum = gl::$value;
        }
    };
}

target!(pub struct Vertex = VERTEX_SHADER);
target!(pub struct Fragment = FRAGMENT_SHADER);

pub enum ProgramShaders<'a> {
    Graphics {
        vertex: &'a CompiledShader<Vertex>,
        /// Contrary to OpenGL, OpenGLES requires a fragment shader.
        fragment: &'a CompiledShader<Fragment>,
    },
}

/// A shader which has no source code.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks resources"]
#[derive(Debug)]
pub struct EmptyShader<Ty: Type>(pub(crate) NonZeroName, core::marker::PhantomData<Ty>);
impl<Ty: Type> EmptyShader<Ty> {
    /// Convert the typestate without checking for correctness.
    ///
    /// # Safety
    /// If `glGetShaderiv(self, GL_COMPILE_STATUS)` would return `true`, this is safe.
    pub unsafe fn into_compiled_unchecked(self) -> CompiledShader<Ty> {
        // Safety: ThinGLObject requires that NonZeroName is a valid CompiledShader
        unsafe { core::mem::transmute(self.into_name()) }
    }
}

impl<Ty: Type> crate::sealed::Sealed for EmptyShader<Ty> {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl<Ty: Type> crate::ThinGLObject for EmptyShader<Ty> {}

/// A shader which has been successfully compiled.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks resources"]
#[derive(Debug)]
pub struct CompiledShader<Ty: Type>(pub(crate) NonZeroName, core::marker::PhantomData<Ty>);

impl<Ty: Type> crate::sealed::Sealed for CompiledShader<Ty> {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl<Ty: Type> crate::ThinGLObject for CompiledShader<Ty> {}

/// Forget the compiled and source code bind status of the shader.
impl<Ty: Type> From<CompiledShader<Ty>> for EmptyShader<Ty> {
    fn from(value: CompiledShader<Ty>) -> Self {
        // Safety: Procondition of ThinGLObject
        unsafe { core::mem::transmute(value) }
    }
}

/// A program which has not been linked.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks resources"]
#[derive(Debug)]
pub struct Program(pub(crate) NonZeroName);
impl Program {
    /// Convert the typestate without checking for correctness.
    ///
    /// # Safety
    /// If `glGetProgramiv(self, GL_LINK_STATUS)` would return `true`, this is safe.
    pub unsafe fn into_linked_unchecked(self) -> LinkedProgram {
        // Safety: ThinGLObject requires that NonZeroName is a valid LinkedProgram
        unsafe { core::mem::transmute(self.into_name()) }
    }
}

impl crate::sealed::Sealed for Program {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl crate::ThinGLObject for Program {}

/// A program which has been successfully linked.
#[repr(transparent)]
#[must_use = "dropping a gl handle leaks resources"]
#[derive(Debug)]
pub struct LinkedProgram(pub(crate) NonZeroName);

/// Forget the linked status of the program.
impl From<LinkedProgram> for Program {
    fn from(value: LinkedProgram) -> Self {
        // Safety: Procondition of ThinGLObject
        unsafe { core::mem::transmute(value) }
    }
}

impl crate::sealed::Sealed for LinkedProgram {}
// # Safety
// Repr(transparent) over a NonZero<u32> (and some ZSTs), so can safely transmute.
unsafe impl crate::ThinGLObject for LinkedProgram {}
