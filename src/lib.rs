//! # GL, HF.
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
//!
//! ## Doc Aliases
//! Unfortunately the design requirements of this crate required that causes functionality be
//! spread out across many namespaces. This crate makes use of `#[doc(alias = ...)]` to allow
//! searching the docs for a GL API.
//!
//! Currently, the full name must be typed **exactly**, case-sensitive, including the
//! `gl`/`GL` prefix.
//!
//! For `glGet*` and `gl*Parameter*`, search by the GL `pname` constant - for example:
//! * `GL_BUFFER_SIZE` will find [`slot::buffer::Active::len`].
//! * `GL_TEXTURE_SWIZZLE_R` will find [`slot::texture::Active::swizzle`].
//!
//! For other functions, search by the GL function name - for example:
//! * `glActiveTexture` will find [`slot::texture::Slots::unit`].
//! * `glReleaseShaderCompiler` will find [`hint::Hint::release_compiler`].

#![warn(rustdoc::all)]

use gl::types::{GLenum, GLsizei, GLuint};
use std::num::NonZero;
type NonZeroName = NonZero<GLuint>;

pub mod gl {
    #![doc(hidden)]
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

// Users may need to name these explicitly if they're working with `unsafe`, or writing
// functions that accept `Active` slots, re-export them in a slightly more accessible place.
pub use slot::marker;

pub mod buffer;
pub mod draw;
pub mod framebuffer;
pub mod hint;
pub mod new;
pub mod program;
pub mod renderbuffer;
pub mod slot;
pub mod state;
pub mod texture;
pub mod vertex_array;

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
    pub new: new::New,
    /// `glUseProgram`
    pub program: slot::program::Slot,
    /// `glDraw*`
    pub draw: draw::Draw,
    /// `glHint` and miscellaneous implementation hints.
    pub hint: hint::Hint,
    /// Miscellaneous global state, such as clear values, blend modes, etc.
    pub state: state::State,
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
    #[must_use]
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
            new: new::New(PhantomData),
            program: program::Slot(PhantomData),
            hint: hint::Hint(PhantomData),
            draw: draw::Draw(PhantomData),
            state: state::State(PhantomData),
            _cant_destructure: (),
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// # Safety
/// * A pointer to `self` must be safely readable and writable as `NonZero<GLuint>`.
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
    /// Export the `GLuint` name, losing the typestate.
    #[must_use = "dropping a gl handle leaks resources"]
    fn into_name(self) -> NonZeroName {
        // Safety - the user can't thrash the type state, since they are
        // moving the name out of the type state system.
        let name = unsafe { self.name() };
        std::mem::forget(self);
        name
    }
}

/// Trait for rusty `GLenum`s.
///
/// # Safety
/// * Must be implemented only on enums.
/// * The enum must be `#[repr(u32)]`
/// * Every variant must be a correct constant of `GLenum`.
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
        assert!(
            !names.assume_init().into_iter().any(|name| name == 0),
            "gl returned a zeroed texture name, UB abounds."
        );
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
