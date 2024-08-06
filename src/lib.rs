//! # Open**GL**, **H**ightened **F**riendliness~!
//!
//! Compile-time-checked, type-state bindings for OpenGL ES 3.X, smoothing over many
//! of the OpenGL gotchas and foot-guns and providing an expressive coating of syntactic sugar.
//!
//! (Ab)uses the borrow checker to check that the user understands what resources will be
//! modified by a GL call, with zero run-time overhead. With few exceptions, every associated
//! function is a transparent wrapper around the relavant GL function. As such, it's power
//! mostly comes from preventing accidental misuse rather than actively checking validity.
//!
//! This is not an object-oriented approach, nor does it aim to implement automatic resource
//! management - it is simply a projection of the OpenGL ownership hierarchy to the rust type
//! system.

use gl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use std::num::NonZero;
type NonZeroName = NonZero<GLuint>;

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

// Users may need to name these explicitly if they're working with `unsafe`, or writing
// functions that accept `Active` slots, re-export them in a slightly more accessible place.
pub use slot::marker;

pub mod buffer;
pub mod draw;
pub mod framebuffer;
pub mod program;
pub mod slot;
pub mod texture;
pub mod vertex_array;

#[repr(u32)]
pub enum DepthCompareFunc {
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
unsafe impl crate::GLEnum for DepthCompareFunc {}

/// Entry points for allocating and deallocating GL objects, wrapping `glGen*`.
///
/// It is generally more efficientto allocate many resources at the same time.
///
/// Some stateless objects can be deallocated through this interface. For stateful objects -
/// e.g., [`texture::Texture2D`] - use their relavant [Slot](`slot`) to
/// destroy them.
///
/// Usage:
/// ```no_run
/// # let gl : glhf::GLHF = todo!();
/// let [one_texture] = gl.create.textures();
/// let [a, bunch, of, framebuffers] = gl.create.framebuffers();
/// ```
// Interestingly, many `glGen*`s are *optional* - you can just make up a number
// and use it. We intentionally don't support this usecase.
pub struct Create(NotSync);
impl Create {
    /// Generate a set of new texture objects.
    pub fn textures<const N: usize>(&self) -> [texture::Stateless; N] {
        unsafe { gl_gen_with(gl::GenTextures) }
    }
    /// Delete stateless textures. To delete stateful textures, use the relevant [`slot::texture`] interface.
    pub fn delete_textures<const N: usize>(&self, textures: [texture::Stateless; N]) {
        unsafe { gl_delete_with(gl::DeleteTextures, textures) }
    }
    /// Generate a set of new framebuffer objects.
    pub fn framebuffers<const N: usize>(&self) -> [framebuffer::Incomplete; N] {
        unsafe { gl_gen_with(gl::GenFramebuffers) }
    }
    /// Generate a set of new framebuffer objects.
    pub fn vertex_arrays<const N: usize>(&self) -> [vertex_array::VertexArray; N] {
        unsafe { gl_gen_with(gl::GenVertexArrays) }
    }
    /// Generate a set of new buffer objects.
    pub fn buffers<const N: usize>(&self) -> [buffer::Buffer; N] {
        unsafe { gl_gen_with(gl::GenBuffers) }
    }
    /// Initialize a shader object of the given type.
    pub fn shader<Ty: program::Type>(&self) -> program::EmptyShader<Ty> {
        let value = unsafe { gl::CreateShader(Ty::TYPE) };
        let name: NonZeroName = value
            .try_into()
            .expect("internal gl error while creating shader");

        // Safety: Precondition of ThinGLOject.
        unsafe { std::mem::transmute(name) }
    }
    /// Initialize a program object.
    pub fn program(&self) -> program::Program {
        let value = unsafe { gl::CreateProgram() };
        let name: NonZeroName = value
            .try_into()
            .expect("internal gl error while creating program");

        // Safety: Precondition of ThinGLOject.
        unsafe { std::mem::transmute(name) }
    }
}

pub struct Hint(NotSync);
impl Hint {
    /// Hint to the GL that you won't be compiling more shaders or programs.
    ///
    /// It is still valid to issue compilation and linking calls after this,
    /// but there may be a significant performance penalty.
    pub fn release_compiler(&self) {
        unsafe { gl::ReleaseShaderCompiler() }
    }
}

/// Entry point for GL calls.
// That's not what we're doing, clippy!
#[allow(clippy::manual_non_exhaustive)]
pub struct GLHF {
    /// `glBindTexture`
    pub texture: slot::texture::Slots,
    /// `glBindFramebuffer`
    pub framebuffer: slot::framebuffer::Slots,
    /// `glBindBuffer`
    pub buffer: slot::buffer::Slots,
    /// `glBindVertexArray`
    pub vertex_array: slot::vertex_array::Slot,
    /// `glGen*`
    pub create: Create,
    /// `glUseProgram`
    pub program: slot::program::Slot,
    pub draw: draw::Draw,
    pub hint: Hint,
    _cant_destructure: (),
}
impl GLHF {
    /// Create a wrapper for the currently bound context.
    /// This is a no-op function, and is free to recreate every frame.
    ///
    /// # Safety
    /// * There must be a current GL context on the calling thread.
    /// * The current GL context should be version ES3.X.
    /// * The `gl` module must have been fully initialized with [`gl::load_with`]
    /// * The GL context that was current at the time of creation must be valid and current
    ///   on the accessing thread at the time of any interaction with this `Self` object
    ///   or any objects derived from it.
    /// * There must be no other Self object representing the this context.
    /// * If multiple `Self` objects exist, it is invalid to use objects derived from one's context
    ///   in methods on another one's context.
    pub unsafe fn current() -> Self {
        use slot::{buffer, framebuffer, program, texture, vertex_array};
        use std::marker::PhantomData;

        // I find it really funny that all this code is constructing a ZST, and is thus a no-op, Lol
        Self {
            texture: texture::Slots {
                d2: texture::Slot::<crate::texture::D2>(PhantomData, PhantomData),
                d3: texture::Slot::<crate::texture::D3>(PhantomData, PhantomData),
                d2_array: texture::Slot::<crate::texture::D2Array>(PhantomData, PhantomData),
                cube: texture::Slot::<crate::texture::Cube>(PhantomData, PhantomData),
            },
            framebuffer: framebuffer::Slots {
                draw: framebuffer::Slot(PhantomData, PhantomData),
                read: framebuffer::Slot(PhantomData, PhantomData),
            },
            buffer: buffer::Slots {
                array: buffer::Slot(PhantomData, PhantomData),
                copy_read: buffer::Slot(PhantomData, PhantomData),
                copy_write: buffer::Slot(PhantomData, PhantomData),
                element_array: buffer::Slot(PhantomData, PhantomData),
                pixel_pack: buffer::Slot(PhantomData, PhantomData),
                pixel_unpack: buffer::Slot(PhantomData, PhantomData),
                transform_feedback: buffer::Slot(PhantomData, PhantomData),
                uniform: buffer::Slot(PhantomData, PhantomData),
            },
            vertex_array: vertex_array::Slot(PhantomData),
            create: Create(PhantomData),
            program: program::Slot(PhantomData),
            hint: Hint(PhantomData),
            draw: draw::Draw(PhantomData),
            _cant_destructure: (),
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// # Safety
/// * A pointer to `self` must be safely writable and writable as `NonZero<GLuint>`.
/// * A value of `NonZero<GLuint>` is a fully-initialized value of `self`.
pub unsafe trait ThinGLObject: sealed::Sealed + Sized {
    /// Fetch the "name" of the object, the unique ID used to interact with the GL.
    /// # Safety
    /// TODO: document all the ways misuse could thrash the typestate x3
    /// For now uhh, don't thrash the typestate, thanx.
    unsafe fn name(&self) -> NonZeroName {
        // Safety - the trait precondition!
        unsafe { *std::ptr::from_ref(self).cast() }
    }
    /// Export the GLuint name, losing the typestate.
    fn into_name(self) -> NonZeroName {
        // Safety - the user can't thrash the type state, since they are
        // moving the name out of the type state system.
        let name = unsafe { self.name() };
        std::mem::forget(self);
        name
    }
}

/// Trait for rusty GLenums.
///
/// # Safety
/// * Must be implemented only on enums.
/// * The enum must be `#[repr(u32)]`
/// * Every variant must be a correct constant of GLenum.
pub unsafe trait GLEnum {
    /// Access the raw `GLenum` value of this enum.
    fn as_gl(&self) -> GLenum {
        unsafe { *std::ptr::from_ref(self).cast() }
    }
}
/// # Safety
/// * The context associated with `gl_gen` must be current on the calling thread.
/// * `gl_gen` must be the appropriate GL generator for objects of type `T`.
/// * `gl_gen` must populate the range given by length and pointer with non-zero values.
unsafe fn gl_gen_with<const N: usize, T: ThinGLObject>(
    gl_gen: unsafe fn(GLsizei, *mut GLuint),
) -> [T; N] {
    // Hm. What if usize is smaller than GLsizei?
    const { assert!(N <= GLsizei::MAX as _) };
    let mut names = std::mem::MaybeUninit::<[T; N]>::uninit();

    // `cast` here goes from array of something repr(NonZero<GLuint>) to GLuint (Safety precondition of impl ThinGLObject).
    gl_gen(N as _, names.as_mut_ptr().cast());

    // At `assume_init`, the objects of type T are allowed to assume they have been initialized with a
    // NON ZERO value. This requirement is forwarded to the signature of this fn.
    #[cfg(debug_assertions)]
    {
        let names = std::mem::transmute_copy::<_, std::mem::MaybeUninit<[GLuint; N]>>(&names);
        if names.assume_init().into_iter().any(|name| name == 0) {
            panic!("gl returned a zeroed texture name, UB abounds.")
        }
    }

    names.assume_init()
}
/// # Safety
/// * The context associated with `gl_delete` must be current on the calling thread.
/// * `gl_delete` must be the appropriate GL deleter for objects of type `T`.
/// * `gl_delete` must destrpy the values in the range given by length and pointer.
unsafe fn gl_delete_with<const N: usize, T: ThinGLObject>(
    gl_delete: unsafe fn(GLsizei, *const GLuint),
    mut names: [T; N],
) {
    // Hm. What if usize is smaller than GLsizei?
    const { assert!(N <= GLsizei::MAX as _) };

    // Cast: impl ThinGLObject is safely interpretable as GLuint
    gl_delete(N as _, names.as_mut_ptr().cast());
}

type NotSync = std::marker::PhantomData<std::cell::Cell<()>>;
