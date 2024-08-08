use glhf::vertex_array;
use glutin::prelude::*;
use ultraviolet::Vec3;

use glhf::gl;

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    pos: Vec3,
    normal: Vec3,
}
fn load_obj(read: impl std::io::BufRead) -> anyhow::Result<(Vec<Vertex>, Vec<u16>)> {
    let lines = read.lines();
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

    program: glhf::program::LinkedProgram,
    index_buffer: glhf::buffer::Buffer,
    num_indices: usize,
    vao: glhf::vertex_array::VertexArray,

    shadow_program: glhf::program::LinkedProgram,
    shadow_texture: glhf::texture::Texture2D,
    shadow_framebuffer: glhf::framebuffer::Complete,
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
        let api = context.context_api();

        println!("Got context {:?}", context.context_api());
        assert!(
            matches!(api, glutin::context::ContextApi::Gles(_)),
            "GLHF is built for GLES!"
        );

        // Load global proc addresses. This is only usable if there is ONE display in use for the lifetime of the program.
        gl::load_with(|sym| display.get_proc_address(&std::ffi::CString::new(sym).unwrap()));

        let mut gl = unsafe { glhf::GLHF::current() };

        // Compile the program used to draw the visual scene, sampling from the shadow mask
        // and accepting direct lighting fron the sun.
        let program = {
            let vertex = gl.new.shader::<glhf::program::Vertex>();
            let vertex = gl
                .program
                .compile(
                    vertex,
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
                )
                .unwrap();
            let fragment = gl.new.shader::<glhf::program::Fragment>();
            let fragment = gl
                .program
                .compile(
                    fragment,
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
                )
                .unwrap();

            let program = gl.new.program();
            let program = gl
                .program
                .link(
                    program,
                    glhf::program::ProgramShaders::Graphics {
                        vertex: &vertex,
                        fragment: &fragment,
                    },
                )
                .unwrap();

            gl.program.delete_shader(vertex.into());
            gl.program.delete_shader(fragment.into());

            program
        };

        // Compile the program for use during the shadow pass.
        let shadow_program = {
            let vertex = gl.new.shader::<glhf::program::Vertex>();
            let vertex = gl
                .program
                .compile(
                    vertex,
                    r"#version 310 es
                precision highp float;

                layout(location = 0) uniform mat4 viewproj;

                layout(location = 0) in vec3 pos;

                void main() {
                    gl_Position = viewproj * vec4(pos, 1.0);
                }",
                )
                .unwrap();
            let fragment = gl.new.shader::<glhf::program::Fragment>();
            let fragment = gl
                .program
                .compile(
                    fragment,
                    r"#version 310 es

                    // Fragments need not do anything! Since we have no color buffers during
                    // this pass, there is nothing to do here anyway.
                    // However, unlike OpenGL, GLES requires that fragment shaders be present.
                    void main() {}
                    ",
                )
                .unwrap();

            let program = gl.new.program();
            let program = gl
                .program
                .link(
                    program,
                    glhf::program::ProgramShaders::Graphics {
                        vertex: &vertex,
                        fragment: &fragment,
                    },
                )
                .unwrap();

            gl.program.delete_shader(vertex.into());
            gl.program.delete_shader(fragment.into());

            program
        };

        // We've compiled all we need :3
        gl.hint.release_compiler();

        // Setup a framebuffer to use for our shadow pass.
        // We will render the scene from the sun's POV into a depth texture,
        // which we can then use to sample from to determine which fragments are visible
        // to the sun (and thus illuminated)
        let [shadow_texture] = gl.new.textures();
        let [shadow_framebuffer] = gl.new.framebuffers();

        // Initialize the texture as 2D.
        let (shadow_texture, texture_slot) = gl.texture.d2.initialize(shadow_texture);
        texture_slot
            // Give it a size and a format.
            .storage(
                // Mip levels. We don't care about mipmapping here.
                1.try_into().unwrap(),
                glhf::texture::InternalFormat::DepthComponent16,
                512.try_into().unwrap(),
                512.try_into().unwrap(),
            )
            // Enable `sampler*Shadow` sampling, which gives us a `true`/`false` lighting
            // value in the shader as opposed to a raw depth reading.
            .compare_mode(Some(glhf::state::CompareFunc::LessEqual))
            // Enable PCF, this makes the shadows softer at some cost of memory bandwidth.
            .min_filter(glhf::texture::Filter::Linear, None)
            .mag_filter(glhf::texture::Filter::Linear);

        gl.framebuffer
            .draw
            .bind(&shadow_framebuffer)
            // Bind our shadow texture as the depth map.
            .texture_2d(&shadow_texture, glhf::framebuffer::Attachment::Depth, 0)
            // No fragment outputs.
            .draw_buffers(&[]);

        // Done specifying attachments, check with the GL to ensure the framebuffer
        // specification is up-to-snuff.
        let (shadow_framebuffer, _) = gl
            .framebuffer
            .draw
            .try_complete(shadow_framebuffer)
            .unwrap();

        // Set up uniforms for the camera and sun matrices.
        // I was too lazy to set up any proper math for this, so it's just done by eye.
        // Good luck.
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
        // Sun "camera", off to the right of the main camera.
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

        // Convert ultraviolet matrices into GLHF matrices.
        let camera_matrix = glhf::program::uniform::Mat4::from(
            camera_matrix.as_component_array().map(|v| *v.as_array()),
        );
        let shadow_matrix = glhf::program::uniform::Mat4::from(
            shadow_matrix.as_component_array().map(|v| *v.as_array()),
        );
        Self::err();

        // In our main program...
        gl.program
            .bind(&program)
            // Bind the matrices we just calculated!
            .uniform_matrix(0, &camera_matrix)
            // Note the location here - matrices take up many uniform slots.
            .uniform_matrix(4, &shadow_matrix)
            // Bind texture unit 0, where we'll put the shadow texture at
            // draw time.
            .uniform(8, &0i32);

        // The shadow program only needs the sun matrix.
        gl.program
            .bind(&shadow_program)
            .uniform_matrix(0, &shadow_matrix);

        // Load a test scene.
        let (vertices, indices) =
            load_obj(std::io::Cursor::new(include_bytes!("test.obj"))).unwrap();

        // Generate unique buffer names.
        let [vertex_buffer, index_buffer] = gl.new.buffers();

        // Bulk upload our scene data
        let vbo = gl.buffer.array.bind(&vertex_buffer);
        // Vertex arrays, containing both position and normals interleaved.
        vbo.data(
            bytemuck::cast_slice(&vertices),
            glhf::buffer::usage::Frequency::Static,
            glhf::buffer::usage::Access::Draw,
        );
        // Index (or, in gl terms, "element") buffer.
        gl.buffer.element_array.bind(&index_buffer).data(
            bytemuck::cast_slice(&indices),
            glhf::buffer::usage::Frequency::Static,
            glhf::buffer::usage::Access::Draw,
        );

        // Vertex arrays store references to (potentially many) "Array" buffers,
        // specifying the layout of vertex attributes within them.
        let [vao] = gl.new.vertex_arrays();

        // Calculate the distance between consecutive attributes.
        // (e.g., the distance from one `Vertex.pos` to the next.)
        let stride = std::mem::size_of::<Vertex>().try_into().unwrap();

        // Set up the vertex specification.
        gl.vertex_array
            .bind(&vao)
            .attribute(
                // Read from our vertex buffer,
                &vbo,
                // Vertex shader location zero,
                0,
                vertex_array::Attribute {
                    // Is a vec3<f32>...
                    components: vertex_array::Components::Vec3,
                    ty: vertex_array::FloatingAttribute::F32.into(),
                    // Each value this many bytes apart...
                    stride: Some(stride),
                    // Offset from the beginning of the buffer by...
                    offset: std::mem::offset_of!(Vertex, pos),
                },
                // Enable fetching for this attribute.
                Some(true),
            )
            .attribute(
                // Again, for the normals.
                &vbo,
                1,
                vertex_array::Attribute {
                    components: vertex_array::Components::Vec3,
                    ty: vertex_array::FloatingAttribute::F32.into(),
                    stride: Some(stride),
                    // Except this time the offset differs.
                    offset: std::mem::offset_of!(Vertex, normal),
                },
                // Enable fetching for this attribute.
                Some(true),
            );

        // We don't need any cleanup, since all our resources last for the lifetime of the program.
        // This here is actually a resource leak! Handles do not implement any kind of resource
        // management. However, we don't need access to this handle anymore, as it's state is
        // captured within the `vao`
        drop(vertex_buffer);

        Self {
            context,
            surface,
            window,
            program,

            num_indices: indices.len(),
            index_buffer,
            vao,

            shadow_texture,
            shadow_framebuffer,
            shadow_program,
        }
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
        use glhf::state;
        let mut gl = unsafe { glhf::GLHF::current() };
        gl.state
            // A nice blue color
            .clear_color([0.0, 0.5, 0.8, 1.0])
            // Clear to 1.0 (max depth for our fixed-point zbuffer!)
            .clear_depth(1.0)
            // Cull the face that is towards the sun. This is a funny trick to
            // reduce "shadow acne" at the cost of some "peter panning" (we graphics creatures love our jargon)
            .cull_face(state::CullFace::Front)
            .enable(state::Capability::CullFace)
            // Pass fragments that are less far than the current zbuffer value
            .depth_func(state::CompareFunc::Less)
            .enable(state::Capability::DepthTest);

        // Bind the index buffer and the vertex array, which contains references to our vertex buffer.
        let elements = gl.buffer.element_array.bind(&self.index_buffer);
        let vertex_array = gl.vertex_array.bind(&self.vao);
        // Bind the shadow framebuffer and the shadow program.
        let framebuffer = gl.framebuffer.draw.bind_complete(&self.shadow_framebuffer);
        let program = gl.program.bind(&self.shadow_program);
        // Clear the depth buffer.
        framebuffer.clear(glhf::slot::framebuffer::AspectMask::DEPTH);

        // Provide static proof-of-state to the `draw.elements` call.
        let draw_info = glhf::draw::ElementState {
            // We have an element buffer bound...
            elements: &elements,
            // ...a complete framebuffer...
            framebuffer: &framebuffer,
            // ...a linked program...
            program: &program,
            // ...and a vertex array!
            vertex_array: &vertex_array,
        };
        unsafe {
            // Draw our indexed mesh.
            gl.draw.elements(
                glhf::draw::Topology::Triangles,
                glhf::draw::ElementType::U16,
                0..self.num_indices,
                1,
                draw_info,
            )
        };

        // Switch to the "default" framebuffer, which is the window surface.
        let framebuffer = gl.framebuffer.draw.bind_default();
        // Clear it and it's depth-bufffer.
        framebuffer.clear(glhf::slot::framebuffer::AspectMask::all());

        // Use the program that samples our shadow mask and calculates lighting.
        let program = gl.program.bind(&self.program);

        // Use a more traditional backface culling.
        gl.state.cull_face(state::CullFace::Back);

        // `program` is set up to read the shadow texture (rendered in the pass above) from slot 0,
        // so ensure that texture is bound there.
        gl.texture.unit(0).d2.bind(&self.shadow_texture);

        // And draw again!
        let draw_info = glhf::draw::ElementState {
            elements: &elements,
            framebuffer: &framebuffer,
            program: &program,
            vertex_array: &vertex_array,
        };
        unsafe {
            gl.draw.elements(
                glhf::draw::Topology::Triangles,
                glhf::draw::ElementType::U16,
                0..self.num_indices,
                1,
                draw_info,
            )
        };

        // Tell winit we're about to swap, and then show it to the user!
        self.window.pre_present_notify();
        self.surface.swap_buffers(&self.context).unwrap();
    }
}

fn main() {
    let mut app = App { window: None };

    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    event_loop.run_app(&mut app).unwrap();
}
