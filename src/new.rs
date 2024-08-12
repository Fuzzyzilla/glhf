//! Rust-flavored allocation functions for GL objects.
use crate::{
    buffer, framebuffer, gl, gl_delete_with, gl_gen_with, program, renderbuffer, texture,
    vertex_array, NonZeroName, NotSync,
};

/// Entry points for allocating and deallocating GL objects, wrapping `glGen*`.
///
/// It is generally more efficientto allocate many resources at the same time.
///
/// Some stateless objects can be deallocated through this interface. For stateful objects -
/// e.g., [`texture::Texture2D`] - use their relavant [Slot](`crate::slot`) to
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
pub struct New(pub(crate) NotSync);
impl New {
    /// Generate a set of new texture objects.
    #[doc(alias = "glGenTextures")]
    pub fn textures<const N: usize>(&self) -> [texture::Stateless; N] {
        unsafe { gl_gen_with(gl::GenTextures) }
    }
    /// Delete stateless textures. To delete stateful textures, use the
    /// [relavent `Slot` interface](crate::slot::texture::Slot::delete).
    #[doc(alias = "glDeleteTextures")]
    pub fn delete_textures<const N: usize>(&self, textures: [texture::Stateless; N]) {
        unsafe { gl_delete_with(gl::DeleteTextures, textures) }
    }
    /// Generate a set of new framebuffer objects.
    #[doc(alias = "glGenFramebuffers")]
    pub fn framebuffers<const N: usize>(&self) -> [framebuffer::Incomplete; N] {
        unsafe { gl_gen_with(gl::GenFramebuffers) }
    }
    /// Generate a set of new vertex array objects.
    #[doc(alias = "glGenVertexArrays")]
    pub fn vertex_arrays<const N: usize>(&self) -> [vertex_array::VertexArray; N] {
        unsafe { gl_gen_with(gl::GenVertexArrays) }
    }
    /// Generate a set of new buffer objects.
    #[doc(alias = "glGenBuffers")]
    pub fn buffers<const N: usize>(&self) -> [buffer::Buffer; N] {
        unsafe { gl_gen_with(gl::GenBuffers) }
    }
    /// Generate a set of renderbuffer objects.
    #[doc(alias = "glGenRenderbuffers")]
    pub fn render_buffers<const N: usize>(&self) -> [renderbuffer::Renderbuffer; N] {
        unsafe { gl_gen_with(gl::GenRenderbuffers) }
    }
    /// Initialize a shader object of the given type.
    /// # Panics
    /// On GL-internal error.
    #[doc(alias = "glCreateShader")]
    pub fn shader<Ty: program::Type>(&self) -> program::EmptyShader<Ty> {
        let value = unsafe { gl::CreateShader(Ty::TYPE) };
        let name: NonZeroName = value
            .try_into()
            .expect("internal gl error while creating shader");

        // Safety: Precondition of ThinGLOject.
        unsafe { std::mem::transmute(name) }
    }
    /// Initialize a program object.
    /// # Panics
    /// On GL-internal error.
    #[doc(alias = "glCreateProgram")]
    pub fn program(&self) -> program::Program {
        let value = unsafe { gl::CreateProgram() };
        let name: NonZeroName = value
            .try_into()
            .expect("internal gl error while creating program");

        // Safety: Precondition of ThinGLOject.
        unsafe { std::mem::transmute(name) }
    }
}
