use std::num::{NonZero, NonZeroU32};

use glhf::ThinGLObject;
use glutin::prelude::*;
use ultraviolet::Vec3;

use gl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use glhf::gl;

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    pos: Vec3,
    normal: Vec3,
}
fn load_obj(mut read: impl std::io::BufRead) -> anyhow::Result<(Vec<Vertex>, Vec<u16>)> {
    use std::io::{BufRead, Read};
    let mut lines = read.lines();
    // OBJ uses 1-based indices, but all the structures below
    // maintain zero-based indexing.

    // Positions, in declaration order.
    let mut positions = vec![];
    // Normals, in declaration order.
    let mut normals = vec![];
    // We need to combine positions and normals into vertices on-the-fly:
    // Map from (position idx, normal idx) -> (vertex idx)
    // This is probably incredibly slow but that's no matter lol
    let mut map = std::collections::HashMap::<(u16, u16), u16>::new();
    // Combined vertices.
    let mut vertices = vec![];
    // Indices into combined vertices.
    let mut indices = vec![];
    for line in lines {
        let line = line?;
        let mut words = line.split_ascii_whitespace();
        let Some(ty) = words.next() else {
            continue;
        };
        match ty {
            "v" => {
                let mut parse_next_word = || -> anyhow::Result<_> {
                    words
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("not enough data"))?
                        .parse()
                        .map_err(Into::into)
                };

                let x: f32 = parse_next_word()?;
                let y: f32 = parse_next_word()?;
                let z: f32 = parse_next_word()?;

                positions.push(Vec3::new(x, y, z));
            }
            "vn" => {
                let mut parse_next_word = || -> anyhow::Result<_> {
                    words
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("not enough data"))?
                        .parse()
                        .map_err(Into::into)
                };

                let x: f32 = parse_next_word()?;
                let y: f32 = parse_next_word()?;
                let z: f32 = parse_next_word()?;

                // Normals not guaranteed to be length 1
                normals.push(Vec3::new(x, y, z).normalized());
            }
            "f" => {
                use std::num::NonZeroU16;
                let mut parse_next_word = || -> anyhow::Result<_> {
                    let next = words
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("not enough data"))?;
                    let mut components = next.split('/');

                    let v = components
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("not enough data"))?;
                    let uv = components
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("not enough data"))?;
                    let vn = components
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("not enough data"))?;

                    assert!(uv.is_empty());

                    Ok((v.parse()?, vn.parse()?))
                };

                // 1-indexed, hence the non-zero.
                let (v1, vn1): (NonZeroU16, NonZeroU16) = parse_next_word()?;
                let (v2, vn2): (NonZeroU16, NonZeroU16) = parse_next_word()?;
                let (v3, vn3): (NonZeroU16, NonZeroU16) = parse_next_word()?;

                assert!(words.next().is_none(), "did you forget to triangulate?");

                let mut index_of = |v: NonZeroU16, vn: NonZeroU16| -> anyhow::Result<u16> {
                    let v = v.get() - 1;
                    let vn = vn.get() - 1;
                    if let Some(index) = map.get(&(v, vn)).copied() {
                        // Already combined and inserted.
                        Ok(index)
                    } else {
                        // Combine position and normal into a vertex.
                        let pos = positions
                            .get(usize::from(v))
                            .copied()
                            .ok_or_else(|| anyhow::anyhow!("position index out of bounds"))?;
                        let normal = normals
                            .get(usize::from(vn))
                            .copied()
                            .ok_or_else(|| anyhow::anyhow!("normal index out of bounds"))?;

                        // Insert into global list and check the index.
                        vertices.push(Vertex { pos, normal });
                        let index = vertices.len() - 1;

                        // Share the index, and return it.
                        let index = index.try_into()?;
                        map.insert((v, vn), index);

                        Ok(index)
                    }
                };

                // Combine and insert all three of our verts!
                indices.extend_from_slice(&[
                    index_of(v1, vn1)?,
                    index_of(v2, vn2)?,
                    index_of(v3, vn3)?,
                ]);
            }
            "#" => (),
            unknown => println!("skipped obj attribute {unknown:?}"),
        }
    }

    Ok((vertices, indices))
}

struct App {
    window: Option<Window>,
}
impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.listen_device_events(winit::event_loop::DeviceEvents::Never);
        if self.window.is_none() {
            self.window = Some(Window::new(event_loop));
        }
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent as Event;
        match event {
            Event::CloseRequested
            | Event::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: winit::event::ElementState::Pressed,
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            Event::RedrawRequested => {
                if let Some(window) = &mut self.window {
                    window.redraw();
                }
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.window.request_redraw();
        }
    }
    fn suspended(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        self.window.take();
    }
    fn exiting(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        self.window.take();
    }
}
struct Window {
    // Field order: context must drop before window.
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
    window: winit::window::Window,

    program: GLuint,
    vertex_buffer: GLuint,
    index_buffer: GLuint,
    num_indices: GLsizei,
    vbo: GLuint,

    shadow_program: GLuint,
    shadow_texture: GLuint,
    shadow_framebuffer: GLuint,
}
impl Window {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        use glutin::display::{GetGlDisplay, GlDisplay};
        use winit::raw_window_handle::HasWindowHandle;

        let (window, config) = glutin_winit::DisplayBuilder::new()
            .build(
                event_loop,
                glutin::config::ConfigTemplateBuilder::new().with_api(glutin::config::Api::GLES3),
                |mut configs| configs.next().unwrap(),
            )
            .unwrap();
        assert!(window.is_none());

        let window = glutin_winit::finalize_window(
            event_loop,
            winit::window::WindowAttributes::default()
                .with_inner_size(winit::dpi::PhysicalSize::new(512, 512))
                .with_resizable(false),
            &config,
        )
        .unwrap();

        let display = config.display();
        let rwh = window.window_handle().unwrap().as_raw();
        // Safety: Window must be valid. It is. Nice. :3
        let surface = unsafe {
            display.create_window_surface(
                &config,
                &glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                    .build(
                        rwh,
                        window.inner_size().width.try_into().unwrap(),
                        window.inner_size().height.try_into().unwrap(),
                    ),
            ).unwrap()
        };
        // Safety: Window must be valid. It is. Cool. :>
        let context = unsafe {
            display.create_context(
                &config,
                &glutin::context::ContextAttributesBuilder::new()
                    .with_profile(glutin::context::GlProfile::Core)
                    .with_debug(true)
                    .with_context_api(glutin::context::ContextApi::Gles(Some(
                        glutin::context::Version::new(3, 1),
                    )))
                    .build(Some(rwh)),
            )
        }
        .unwrap()
        .make_current(&surface)
        .unwrap();

        // Load global proc addresses. This is only usable if there is ONE display in use for the lifetime of the program.
        gl::load_with(|sym| display.get_proc_address(&std::ffi::CString::new(sym).unwrap()));

        println!("Got context {:?}", context.context_api());
        unsafe {
            let mut major = 0;
            let mut minor = 0;
            gl::GetIntegerv(gl::MAJOR_VERSION, std::ptr::addr_of_mut!(major));
            Self::err();
            gl::GetIntegerv(gl::MINOR_VERSION, std::ptr::addr_of_mut!(minor));
            Self::err();
            println!("Version: {major}.{minor}");
            let mut workgroups = [0; 3];
            for i in 0..3 {
                gl::GetIntegeri_v(
                    gl::MAX_COMPUTE_WORK_GROUP_SIZE,
                    i,
                    std::ptr::addr_of_mut!(workgroups[i as usize]),
                );
                Self::err();
            }
            println!("Workgroups: {workgroups:?}");
        }

        let program = unsafe {
            Self::compile(
                r"#version 310 es
                precision highp float;

                layout(location = 0) uniform mat4 viewproj;
                layout(location = 4) uniform mat4 shadow_viewproj;

                layout(location = 0) in vec3 pos;
                layout(location = 1) in vec3 normal;

                layout(location = 0) out vec3 shadow_pos_ndc;
                layout(location = 1) out float sun;

                void main() {
                    vec3 sun_dir = normalize((inverse(shadow_viewproj) * vec4(0.0, 0.0, 1.0, 1.0)).xyz);
                    sun = max(-dot(sun_dir, normal) * 0.7 + 0.3, 0.0);

                    vec4 shadow_pos = shadow_viewproj * vec4(pos, 1.0);
                    shadow_pos_ndc = shadow_pos.xyz / shadow_pos.w;

                    gl_Position = viewproj * vec4(pos, 1.0);
                }",
                Some(
                    r"#version 310 es
                    precision highp float;
                    layout(location = 8) uniform highp sampler2DShadow shadow;

                    layout(location = 0) in vec3 shadow_pos_ndc;
                    layout(location = 1) in float sun;

                    layout(location = 0) out vec4 color;

                    void main() {
                        const float AMBIENT = 0.1;
                        // Note to future self, this don't work if shadow is non orthographic.
                        vec3 shadow_uvz = shadow_pos_ndc * 0.5 + 0.5;
                        float depth = texture(shadow, shadow_uvz);

                        float total_light = ((depth * 0.8 + 0.2) * sun) *(1.0 - AMBIENT) + AMBIENT;

                        ivec2 funnier_uv = ivec2(shadow_pos_ndc.xy * 20.0);
                        vec3 albedo = (funnier_uv.x + funnier_uv.y) % 2 == 0 ? vec3(1.0): vec3(0.8, 0.4, 0.9);

                        color = vec4(total_light * albedo, 1.0);
                    }",
                ),
            )
        }
        .unwrap();

        let shadow_program = unsafe {
            Self::compile(
                r"#version 310 es
                precision highp float;

                layout(location = 0) uniform mat4 viewproj;

                layout(location = 0) in vec3 pos;

                void main() {
                    gl_Position = viewproj * vec4(pos, 1.0);
                }",
                // Eh? Errors with "program lacks a fragment shader" if this is None, but
                // fragment shaders are optional?!?
                Some(
                    r"#version 310 es
                    void main() {}
                    ",
                ),
            )
        }
        .unwrap();

        let mut gl = unsafe { glhf::GLHF::current() };

        let [shadow] = gl.create.textures();
        let [shadow_framebuffer] = gl.create.framebuffers();

        let (shadow, texture_slot) = gl.texture.d2.initialize(shadow);
        texture_slot
            .storage(
                1.try_into().unwrap(),
                glhf::texture::InternalFormat::DepthComponent16,
                512.try_into().unwrap(),
                512.try_into().unwrap(),
            )
            // Enable `sampelr*Shadow` sampling
            .compare_mode(Some(glhf::DepthCompareFunc::LessEqual))
            // Enable PCF
            .min_filter(glhf::texture::Filter::Linear, None)
            .mag_filter(glhf::texture::Filter::Linear);

        gl.framebuffer
            .draw
            .bind(&shadow_framebuffer)
            .texture_2d(&shadow, glhf::framebuffer::Attachment::Depth, 0)
            // No fragment outputs.
            .draw_buffers(&[]);

        let (shadow_framebuffer, _) = gl
            .framebuffer
            .draw
            .try_complete(shadow_framebuffer)
            .unwrap();

        let shadow_texture = shadow.into_name().get();
        let shadow_framebuffer = shadow_framebuffer.into_name().get();

        // Camera at +,+ looking roughly towards origin.
        let camera_matrix = {
            let proj = ultraviolet::projection::rh_yup::perspective_gl(
                std::f32::consts::FRAC_PI_6,
                1.0,
                0.1,
                10.0,
            );
            let translate = ultraviolet::Mat4::from_translation(Vec3::new(-1.5, -1.4, -1.5));
            let rotate = ultraviolet::Mat4::from_rotation_around(
                ultraviolet::Vec4::unit_x(),
                std::f32::consts::FRAC_PI_6,
            ) * ultraviolet::Mat4::from_rotation_around(
                ultraviolet::Vec4::unit_y(),
                -std::f32::consts::FRAC_PI_4,
            );

            proj * (rotate * translate)
        };
        // Camera at light at 0,0 looking down.
        // Lol, what a metal variable name.
        let shadow_matrix = {
            let proj =
                ultraviolet::projection::rh_yup::orthographic_gl(-1.0, 1.0, -1.0, 1.0, 0.4, 1.2);
            let translate = ultraviolet::Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0));
            let rotate = ultraviolet::Mat4::from_rotation_around(
                ultraviolet::Vec4::unit_x(),
                std::f32::consts::FRAC_PI_2,
            );

            let funnier_rotate = ultraviolet::Mat4::from_rotation_around(
                ultraviolet::Vec4::new(0.5, 0.0, 1.0, 0.0).normalized(),
                std::f32::consts::FRAC_PI_6 + 0.2,
            );

            proj * funnier_rotate * (rotate * translate)
        };

        unsafe {
            gl::UseProgram(program);
            Self::err();
            gl::UniformMatrix4fv(0, 1, gl::FALSE, camera_matrix.as_ptr());
            Self::err();
            gl::UniformMatrix4fv(4, 1, gl::FALSE, shadow_matrix.as_ptr());
            Self::err();
            gl::Uniform1i(8, 0);
            Self::err();

            gl::UseProgram(shadow_program);
            Self::err();
            gl::UniformMatrix4fv(0, 1, gl::FALSE, shadow_matrix.as_ptr());
            Self::err();
        }

        let (vertices, indices) =
            load_obj(std::io::Cursor::new(include_bytes!("../test.obj"))).unwrap();

        let [vertex_buffer, index_buffer] = gl.create.buffers();

        gl.buffer.array.bind(&vertex_buffer).data(
            bytemuck::cast_slice(&vertices),
            glhf::buffer::usage::Frequency::Static,
            glhf::buffer::usage::Access::Draw,
        );
        gl.buffer.element_array.bind(&index_buffer).data(
            bytemuck::cast_slice(&indices),
            glhf::buffer::usage::Frequency::Static,
            glhf::buffer::usage::Access::Draw,
        );

        let num_indices = indices.len().try_into().unwrap();

        let vbo = unsafe { Self::make_vertex_vbo().unwrap() };

        Self {
            context,
            surface,
            window,
            program,

            num_indices,
            index_buffer: index_buffer.into_name().get(),
            vertex_buffer: vertex_buffer.into_name().get(),
            vbo,

            shadow_texture,
            shadow_framebuffer,
            shadow_program,
        }
    }
    unsafe fn make_vertex_vbo() -> anyhow::Result<GLuint> {
        let mut array = 0;
        gl::GenVertexArrays(1, std::ptr::addr_of_mut!(array));
        gl::BindVertexArray(array);
        let stride = std::mem::size_of::<Vertex>().try_into()?;
        // Position
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
        gl::EnableVertexAttribArray(0);
        // Normal
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            std::mem::offset_of!(Vertex, normal) as _,
        );
        gl::EnableVertexAttribArray(1);

        Ok(array)
    }
    unsafe fn compile(vertex: &str, fragment: Option<&str>) -> anyhow::Result<GLuint> {
        let program = gl::CreateProgram();

        let compile_shader = |ty: gl::types::GLenum, src: &str| -> anyhow::Result<GLuint> {
            let shader = gl::CreateShader(ty);
            let sources = [src.as_ptr().cast::<GLchar>()];
            let lengths = [GLint::try_from(src.len())?];
            // Sources *may* have nul-bytes, as they are UTF8 - I couldn't find any verbage that says this *isn't* allowed ;3
            gl::ShaderSource(shader, 1, sources.as_ptr(), lengths.as_ptr());
            Self::err();
            gl::CompileShader(shader);
            Self::err();
            let mut was_successful = gl::FALSE.into();
            gl::GetShaderiv(
                shader,
                gl::COMPILE_STATUS,
                std::ptr::addr_of_mut!(was_successful),
            );
            Self::err();
            if was_successful == gl::FALSE.into() {
                let mut length = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, std::ptr::addr_of_mut!(length));
                Self::err();
                let mut string_bytes = vec![0; length.try_into().unwrap()];
                gl::GetShaderInfoLog(
                    shader,
                    string_bytes.len().try_into().unwrap(),
                    std::ptr::null_mut(),
                    string_bytes.as_mut_ptr(),
                );
                Self::err();
                // i8 -> u8 reinterpret.
                let (ptr, len, cap) = (
                    string_bytes.as_mut_ptr(),
                    string_bytes.len(),
                    string_bytes.capacity(),
                );
                std::mem::forget(string_bytes);
                let string_bytes = Vec::from_raw_parts(ptr.cast::<u8>(), len, cap);

                let cstr = std::ffi::CString::from_vec_with_nul(string_bytes).unwrap();
                anyhow::bail!("shader failed to compile:\n{cstr:?}");
            }
            Ok(shader)
        };

        let vertex = compile_shader(gl::VERTEX_SHADER, vertex)?;
        gl::AttachShader(program, vertex);
        Self::err();
        let fragment = if let Some(fragment) = fragment {
            let fragment = compile_shader(gl::FRAGMENT_SHADER, fragment)?;
            gl::AttachShader(program, fragment);
            Self::err();
            Some(fragment)
        } else {
            None
        };

        gl::LinkProgram(program);

        let mut was_successful = gl::FALSE.into();
        gl::GetProgramiv(
            program,
            gl::LINK_STATUS,
            std::ptr::addr_of_mut!(was_successful),
        );
        Self::err();
        if was_successful == gl::FALSE.into() {
            let mut length = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, std::ptr::addr_of_mut!(length));
            Self::err();
            let mut string_bytes = vec![0; length.try_into().unwrap()];
            gl::GetProgramInfoLog(
                program,
                string_bytes.len().try_into().unwrap(),
                std::ptr::null_mut(),
                string_bytes.as_mut_ptr(),
            );
            Self::err();
            // i8 -> u8 reinterpret.
            let (ptr, len, cap) = (
                string_bytes.as_mut_ptr(),
                string_bytes.len(),
                string_bytes.capacity(),
            );
            std::mem::forget(string_bytes);
            let string_bytes = Vec::from_raw_parts(ptr.cast::<u8>(), len, cap);

            let cstr = std::ffi::CString::from_vec_with_nul(string_bytes).unwrap();
            anyhow::bail!("program failed to link:\n{cstr:?}");
        }

        gl::DetachShader(program, vertex);
        Self::err();
        gl::DeleteShader(vertex);
        Self::err();
        if let Some(fragment) = fragment {
            gl::DetachShader(program, fragment);
            Self::err();
            gl::DeleteShader(fragment);
            Self::err();
        }

        Ok(program)
    }
    fn err() {
        let err = unsafe { gl::GetError() };
        let string: Option<std::borrow::Cow<str>> = match err {
            gl::NO_ERROR => None,
            gl::INVALID_ENUM => Some("invalid enum".into()),
            gl::INVALID_VALUE => Some("invalid value".into()),
            gl::INVALID_OPERATION => Some("invalid operation".into()),
            _ => Some(format!("unknown error: 0x{err:x}").into()),
        };

        if let Some(string) = string {
            println!("gl err: {string}\n{}", std::backtrace::Backtrace::capture());
        }
    }
    fn redraw(&mut self) {
        unsafe {
            Self::err();
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_framebuffer);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::FRONT);
            Self::err();
            gl::UseProgram(self.shadow_program);
            Self::err();
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            Self::err();
            gl::BindVertexArray(self.vbo);
            Self::err();
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
            Self::err();
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
            Self::err();
            gl::EnableVertexAttribArray(0);
            Self::err();
            gl::EnableVertexAttribArray(1);
            Self::err();
            gl::DrawElements(
                gl::TRIANGLES,
                self.num_indices,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );
            Self::err();

            gl::CullFace(gl::BACK);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            Self::err();
            gl::UseProgram(self.program);
            Self::err();
            gl::ClearColor(0.0, 0.5, 0.8, 1.0);
            Self::err();
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            Self::err();

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.shadow_texture);
            Self::err();
            gl::DrawElements(
                gl::TRIANGLES,
                self.num_indices,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );

            Self::err();
        }
        self.window.pre_present_notify();
        self.surface.swap_buffers(&self.context).unwrap();
    }
}

fn main() {
    let mut app = App { window: None };

    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    event_loop.run_app(&mut app).unwrap();
}
