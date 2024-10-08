//! Using and compiling Shaders and Programs.
use crate::{
    gl::{
        self,
        types::{GLchar, GLenum, GLint, GLsizei, GLuint},
    },
    program::{self, CompiledShader, EmptyShader, LinkedProgram, Program, ProgramShaders, Type},
    slot::marker::{IsDefault, NotDefault, Unknown},
    NotSync, ThinGLObject,
};
#[cfg(feature = "alloc")]
unsafe fn info_log(
    name: GLuint,
    get_iv: unsafe fn(GLuint, GLenum, *mut GLint),
    fetch_log: unsafe fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar),
) -> alloc::ffi::CString {
    // Fetch the length of buffer to allocate.
    let mut length = 0;
    get_iv(name, gl::INFO_LOG_LENGTH, core::ptr::addr_of_mut!(length));

    // Exit early if zero length. Otherwise, assert fails below.
    if length == 0 {
        return alloc::ffi::CString::default();
    }

    // Allocate and populate.
    let mut string_bytes = alloc::vec::Vec::<u8>::with_capacity(length.try_into().unwrap());
    fetch_log(
        name,
        // In param for max length
        string_bytes.capacity().try_into().unwrap(),
        // Out param for actual length
        core::ptr::addr_of_mut!(length),
        // GL uses i8 char, we want u8. This is totally fine.
        string_bytes.as_mut_ptr().cast(),
    );
    // Get call writes `length` to be the size of log, +1 for nul terminator
    let actual_length = usize::try_from(length).unwrap().checked_add(1).unwrap();
    string_bytes.set_len(actual_length);

    // Expect nul-terminated string from vec.
    alloc::ffi::CString::from_vec_with_nul(string_bytes).unwrap()
}
#[cfg(feature = "alloc")]
unsafe fn shader_log(shader: GLuint) -> alloc::ffi::CString {
    info_log(shader, gl::GetShaderiv, gl::GetShaderInfoLog)
}
#[cfg(feature = "alloc")]
unsafe fn program_log(program: GLuint) -> alloc::ffi::CString {
    info_log(program, gl::GetProgramiv, gl::GetProgramInfoLog)
}

#[derive(Debug)]
#[must_use = "dropping a gl handle leaks resources"]
/// If the feature `alloc` is enabled, includes the GL-provided error log.
pub struct CompileError<Ty: Type> {
    pub shader: EmptyShader<Ty>,
    #[cfg(feature = "alloc")]
    pub error: alloc::ffi::CString,
}

#[derive(Debug)]
#[must_use = "dropping a gl handle leaks resources"]
/// If the feature `alloc` is enabled, includes the GL-provided error log.
pub struct LinkError {
    pub program: Program,
    #[cfg(feature = "alloc")]
    pub error: alloc::ffi::CString,
}

impl Active<NotDefault> {
    /// Starting at `base_location`, bind one (or an array) of uniform scalars or vectors.
    /// The value may only be an array if it was declared as an array within the shader.
    ///
    /// The number of uniform locations consumed is given by `value.slots()`
    #[doc(alias = "glUniform")]
    #[doc(alias = "glUniform1f")]
    #[doc(alias = "glUniform2f")]
    #[doc(alias = "glUniform3f")]
    #[doc(alias = "glUniform4f")]
    #[doc(alias = "glUniform1i")]
    #[doc(alias = "glUniform2i")]
    #[doc(alias = "glUniform3i")]
    #[doc(alias = "glUniform4i")]
    #[doc(alias = "glUniform1ui")]
    #[doc(alias = "glUniform2ui")]
    #[doc(alias = "glUniform3ui")]
    #[doc(alias = "glUniform4ui")]
    #[doc(alias = "glUniform1fv")]
    #[doc(alias = "glUniform2fv")]
    #[doc(alias = "glUniform3fv")]
    #[doc(alias = "glUniform4fv")]
    #[doc(alias = "glUniform1iv")]
    #[doc(alias = "glUniform2iv")]
    #[doc(alias = "glUniform3iv")]
    #[doc(alias = "glUniform4iv")]
    #[doc(alias = "glUniform1uiv")]
    #[doc(alias = "glUniform2uiv")]
    #[doc(alias = "glUniform3uiv")]
    #[doc(alias = "glUniform4uiv")]
    pub fn uniform<
        'tiny,
        T: program::uniform::Value,
        Value: Into<program::uniform::Vector<'tiny, T>>,
    >(
        &mut self,
        base_location: u32,
        value: Value,
    ) -> &mut Self {
        use program::uniform::{Ty, Vector};

        let value = value.into();

        if value.is_empty() {
            return self;
        }

        let location = base_location.try_into().unwrap();

        // Nightmare match, lol.
        match value {
            Vector::Scalar(s) => match T::TYPE {
                Ty::F32 => unsafe {
                    gl::Uniform1fv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
                Ty::I32 => unsafe {
                    gl::Uniform1iv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
                Ty::U32 => unsafe {
                    gl::Uniform1uiv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
            },

            Vector::Vec2(s) => match T::TYPE {
                Ty::F32 => unsafe {
                    gl::Uniform2fv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
                Ty::I32 => unsafe {
                    gl::Uniform2iv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
                Ty::U32 => unsafe {
                    gl::Uniform2uiv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
            },

            Vector::Vec3(s) => match T::TYPE {
                Ty::F32 => unsafe {
                    gl::Uniform3fv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
                Ty::I32 => unsafe {
                    gl::Uniform3iv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
                Ty::U32 => unsafe {
                    gl::Uniform3uiv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
            },

            Vector::Vec4(s) => match T::TYPE {
                Ty::F32 => unsafe {
                    gl::Uniform4fv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
                Ty::I32 => unsafe {
                    gl::Uniform4iv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
                Ty::U32 => unsafe {
                    gl::Uniform4uiv(location, s.len().try_into().unwrap(), s.as_ptr().cast());
                },
            },
        }
        self
    }
    /// Starting at `base_location`, bind one (or an array) of uniform matrices.
    /// The value may only be an array if it was declared as an array within the shader.
    ///
    /// The number of uniform locations consumed is given by `value.slots()`
    #[doc(alias = "glUniform")]
    #[doc(alias = "glUniformMatrix")]
    #[doc(alias = "glUniformMatrix2fv")]
    #[doc(alias = "glUniformMatrix3fv")]
    #[doc(alias = "glUniformMatrix4fv")]
    #[doc(alias = "glUniformMatrix2x3fv")]
    #[doc(alias = "glUniformMatrix3x2fv")]
    #[doc(alias = "glUniformMatrix2x4fv")]
    #[doc(alias = "glUniformMatrix4x2fv")]
    #[doc(alias = "glUniformMatrix3x4fv")]
    #[doc(alias = "glUniformMatrix4x3fv")]
    pub fn uniform_matrix<'tiny>(
        &mut self,
        base_location: u32,
        value: impl Into<program::uniform::Matrix<'tiny>>,
    ) -> &mut Self {
        use program::uniform::Matrix;
        let value = value.into();

        if value.is_empty() {
            return self;
        }

        let location = base_location.try_into().unwrap();

        // Another nightmare match, lmao.
        match value {
            Matrix::Mat2(s) => unsafe {
                gl::UniformMatrix2fv(
                    location,
                    s.len().try_into().unwrap(),
                    gl::FALSE,
                    s.as_ptr().cast(),
                );
            },
            Matrix::Mat3(s) => unsafe {
                gl::UniformMatrix3fv(
                    location,
                    s.len().try_into().unwrap(),
                    gl::FALSE,
                    s.as_ptr().cast(),
                );
            },
            Matrix::Mat4(s) => unsafe {
                gl::UniformMatrix4fv(
                    location,
                    s.len().try_into().unwrap(),
                    gl::FALSE,
                    s.as_ptr().cast(),
                );
            },
            Matrix::Mat2x3(s) => unsafe {
                gl::UniformMatrix2x3fv(
                    location,
                    s.len().try_into().unwrap(),
                    gl::FALSE,
                    s.as_ptr().cast(),
                );
            },
            Matrix::Mat2x4(s) => unsafe {
                gl::UniformMatrix2x4fv(
                    location,
                    s.len().try_into().unwrap(),
                    gl::FALSE,
                    s.as_ptr().cast(),
                );
            },
            Matrix::Mat3x2(s) => unsafe {
                gl::UniformMatrix3x2fv(
                    location,
                    s.len().try_into().unwrap(),
                    gl::FALSE,
                    s.as_ptr().cast(),
                );
            },
            Matrix::Mat3x4(s) => unsafe {
                gl::UniformMatrix3x4fv(
                    location,
                    s.len().try_into().unwrap(),
                    gl::FALSE,
                    s.as_ptr().cast(),
                );
            },
            Matrix::Mat4x3(s) => unsafe {
                gl::UniformMatrix4x3fv(
                    location,
                    s.len().try_into().unwrap(),
                    gl::FALSE,
                    s.as_ptr().cast(),
                );
            },
            Matrix::Mat4x2(s) => unsafe {
                gl::UniformMatrix4x2fv(
                    location,
                    s.len().try_into().unwrap(),
                    gl::FALSE,
                    s.as_ptr().cast(),
                );
            },
        }
        self
    }
}

/// Entry points for working with `glUse`d programs.
pub struct Active<Kind>(core::marker::PhantomData<Kind>);
pub struct Slot(pub(crate) NotSync);
impl Slot {
    /// `glUse` a linked program.
    #[doc(alias = "glUseProgram")]
    pub fn bind(&mut self, program: &LinkedProgram) -> &mut Active<NotDefault> {
        unsafe {
            gl::UseProgram(program.name().get());
        }
        super::zst_mut()
    }
    /// Make the used program slot empty.
    #[doc(alias = "glUseProgram")]
    pub fn unbind(&mut self) -> &mut Active<IsDefault> {
        unsafe {
            gl::UseProgram(0);
        }
        super::zst_mut()
    }
    /// Set the GLSL ES source code of a shader, then attempt to compile it.
    // Is there a usecase for allowing each step of this process manually...?
    #[doc(alias = "glShaderSource")]
    #[doc(alias = "glCompileShader")]
    pub fn compile<Ty: Type>(
        &self,
        shader: EmptyShader<Ty>,
        source: &str,
    ) -> Result<CompiledShader<Ty>, CompileError<Ty>> {
        let sources = [source.as_ptr().cast::<gl::types::GLchar>()];
        let lengths = [source.len().try_into().unwrap()];

        let success = unsafe {
            // Source *may* have nul-bytes, as they are UTF8 - I couldn't find any verbage that says this *isn't* allowed ;3
            gl::ShaderSource(shader.name().get(), 1, sources.as_ptr(), lengths.as_ptr());
            gl::CompileShader(shader.name().get());

            let mut was_successful = gl::FALSE.into();
            gl::GetShaderiv(
                shader.name().get(),
                gl::COMPILE_STATUS,
                core::ptr::addr_of_mut!(was_successful),
            );
            was_successful == gl::TRUE.into()
        };

        if success {
            // Safety: we just checked, silly goose!
            Ok(unsafe { shader.into_compiled_unchecked() })
        } else {
            #[cfg(feature = "alloc")]
            {
                Err(CompileError {
                    error: unsafe { shader_log(shader.name().get()) },
                    shader,
                })
            }
            #[cfg(not(feature = "alloc"))]
            {
                Err(CompileError { shader })
            }
        }
    }
    /// Link together several compiled shaders into a [`LinkedProgram`]
    // Is there a usecase for allowing each step of this process manually...?
    #[doc(alias = "glLinkProgram")]
    #[doc(alias = "glAttachShader")]
    pub fn link(
        &self,
        program: Program,
        shaders: ProgramShaders,
    ) -> Result<LinkedProgram, LinkError> {
        let ProgramShaders::Graphics { vertex, fragment } = shaders;
        let success = unsafe {
            gl::AttachShader(program.name().get(), vertex.name().get());
            gl::AttachShader(program.name().get(), fragment.name().get());

            gl::LinkProgram(program.name().get());

            let mut was_successful = gl::FALSE.into();
            gl::GetProgramiv(
                program.name().get(),
                gl::LINK_STATUS,
                core::ptr::addr_of_mut!(was_successful),
            );

            gl::DetachShader(program.name().get(), vertex.name().get());
            gl::DetachShader(program.name().get(), fragment.name().get());

            was_successful == gl::TRUE.into()
        };

        if success {
            // Safety: we just checked, knucklehead!
            Ok(unsafe { program.into_linked_unchecked() })
        } else {
            #[cfg(feature = "alloc")]
            {
                Err(LinkError {
                    error: unsafe { program_log(program.name().get()) },
                    program,
                })
            }
            #[cfg(not(feature = "alloc"))]
            {
                Err(LinkError { program })
            }
        }
    }
    /// Inherit the currently bound program - this may be no program at all.
    ///
    /// Most functionality is limited when the status of the program (`Empty` or `NotEmpty`) is not known.
    #[must_use]
    pub fn inherit(&self) -> &Active<Unknown> {
        super::zst_ref()
    }
    /// Inherit the currently bound program - this may be no program at all.
    ///
    /// Most functionality is limited when the status of the program (`Empty` or `NotEmpty`) is not known.
    #[must_use]
    pub fn inherit_mut(&mut self) -> &mut Active<Unknown> {
        super::zst_mut()
    }
    /// Delete a program. If the program is currently bound to the slot, it remains so
    /// and will be deleted at the moment it is no longer bound.
    ///
    /// To delete a [`LinkedProgram`], use [`Into::into`].
    // Unlike most deletion functions, this one takes shared ref self - DeleteProgram
    // defers the deletion until another program is bound, weirdly enough, and thus
    // does not invalidate outstanding `Active` markers.
    #[doc(alias = "glDeleteProgram")]
    pub fn delete(&self, program: Program) {
        unsafe { gl::DeleteProgram(program.into_name().get()) }
    }
    /// Delete a shader. If the shader is currently attached to any program, it remains so
    /// and will be deleted at the moment it is no longer attached to any program.
    ///
    /// To delete a [`CompiledShader`], use [`Into::into`].
    #[doc(alias = "glDeleteShader")]
    pub fn delete_shader<Ty: Type>(&self, shader: EmptyShader<Ty>) {
        unsafe { gl::DeleteShader(shader.into_name().get()) }
    }
}
