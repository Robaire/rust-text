extern crate gl;
use gl::types::{GLchar, GLenum, GLint, GLuint};

extern crate sdl2;
use sdl2::event::{Event, WindowEvent};
use sdl2::video::GLProfile;

extern crate nalgebra;
use nalgebra::Matrix4;
use nalgebra::Orthographic3;

pub mod shader;
use shader::{Program, Shader};

pub mod gl_util;

use std::ffi::CString;
use std::{collections::HashMap, ptr::null};

fn main() {
    // The initial size of the window, as a fraction of the display width,
    // maintaining the aspect ratio of the display
    // TODO: Fix issues with virtual resolution (aka display scaling)
    let initial_window_size = 0.5;

    // Initialize SDL and create a window
    let (sdl_context, window, _gl_context, video_subsystem) = {
        // Initialize SDL
        let sdl_context = match sdl2::init() {
            Ok(context) => context,
            Err(message) => panic!("SDL Init Failed: {}", message),
        };

        // Ask SDL to initialize the video system
        let video_subsystem = match sdl_context.video() {
            Ok(video_subsystem) => video_subsystem,
            Err(message) => panic!("Failed to create video subsystem: {}", message),
        };

        // Set the attributes of the OpenGL Context
        // let gl_attributes = video_subsystem.gl_attr();
        // gl_attributes.set_context_profile(GLProfile::Core);
        // gl_attributes.set_context_flags().debug().set();
        // gl_attributes.set_context_version(3, 3);

        // Determine the size of the window to open
        let (width, height) = match video_subsystem.desktop_display_mode(0) {
            Ok(display_mode) => {
                // Compute the width and height of the window
                let width = display_mode.w as f32 * initial_window_size;
                let height = width / (display_mode.w as f32 / display_mode.h as f32);

                (width as u32, height as u32)
            }
            Err(message) => panic!("Failed to get desktop display mode: {}", message),
        };

        // Create the window
        let window = match video_subsystem
            .window("Rust Font Rendering", width, height)
            .position_centered()
            .resizable()
            .opengl()
            .build()
        {
            Ok(window) => window,
            Err(message) => panic!("Failed to create window: {}", message),
        };

        // Create the OpenGL Context
        let gl_context = match window.gl_create_context() {
            Ok(context) => context,
            Err(message) => panic!("Failed to create OpenGL Context: {}", message),
        };

        // Load the OpenGL Functions
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::ffi::c_void);

        (sdl_context, window, gl_context, video_subsystem)
    };

    // Generate font textures with Freetype
    struct Character {
        id: GLuint,
        size: (i32, i32),
        bearing: (i32, i32),
        advance: i32,
    }

    let char_map = {
        // Initialize Freetype
        let ft_library = freetype::Library::init().unwrap();

        // Load the font face we want to use
        let face = match ft_library.new_face("./src/fonts/KottaOne.ttf", 0) {
            Ok(face) => face,
            Err(message) => panic!("Unable to open font: {}", message),
        };

        // Set the character size using the display DPI
        let dpi = match video_subsystem.display_dpi(0) {
            Ok(dpi) => dpi,
            Err(_) => (200.0, 200.0, 200.0),
        };

        face.set_char_size(0, 32 * 64, dpi.0 as u32, dpi.1 as u32)
            .unwrap();

        // Create the map that will store our character textures
        let mut char_map: HashMap<u8, Character> = HashMap::new();

        // Disable byte-alignment
        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        }

        // Generate a texture for every ascii character
        for c in 0..128 {
            // Attempt to load the glyph
            match face.load_char(c as usize, freetype::face::LoadFlag::RENDER) {
                Ok(_) => (),
                Err(_) => continue,
            };
            let glyph = face.glyph();

            // Generate a texture and copy the glyphs bitmap into it
            let id = gl_util::generate_texture();
            gl_util::bind_texture(id);

            unsafe {
                // Upload bitmap data to the texture
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::R8 as i32,
                    glyph.bitmap().width(),
                    glyph.bitmap().rows(),
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    glyph.bitmap().buffer().as_ptr() as *const gl::types::GLvoid,
                );

                // Set texture options
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            }

            // Add this character to our map
            char_map.insert(
                c as u8,
                Character {
                    id,
                    size: (glyph.bitmap().width(), glyph.bitmap().rows()),
                    bearing: (glyph.bitmap_left(), glyph.bitmap_top()),
                    advance: glyph.advance().x as i32,
                },
            );
        }

        char_map
    };

    let (vao, vbo) = {
        // Create buffers for rendering text
        let vao = gl_util::generate_vertex_array();
        let vbo = gl_util::generate_buffer();

        gl_util::bind_array(vao);
        gl_util::bind_buffer(vbo);

        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * 6 * 4) as gl::types::GLsizeiptr,
                null(),
                gl::DYNAMIC_DRAW,
            );

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                4,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as i32,
                null(),
            );
        }

        gl_util::bind_array(0);
        gl_util::bind_buffer(0);

        (vao, vbo)
    };

    // Create shader programs to render the font
    let shader_program = {
        // Load shaders
        let vertex_shader =
            match Shader::new_from_file("./src/shaders/vertex.glsl", gl::VERTEX_SHADER) {
                Ok(shader) => shader,
                Err(message) => panic!(format!("Failed to create vertex shader: {}", message)),
            };

        let fragment_shader =
            match Shader::new_from_file("./src/shaders/fragment.glsl", gl::FRAGMENT_SHADER) {
                Ok(shader) => shader,
                Err(message) => panic!(format!("Failed to create fragment shader: {}", message)),
            };

        // Create shader program
        let shader_program = match Program::new()
            .attach_shader(&vertex_shader)
            .attach_shader(&fragment_shader)
            .link()
        {
            Ok(program) => program,
            Err(message) => panic!(format!("Failed to create shader program: {}", message)),
        };

        // Use shader program
        shader_program.set_used();

        shader_program
    };

    // Set the projection matrix
    let projection_id = unsafe {
        gl::GetUniformLocation(
            shader_program.id,
            CString::new("projection").unwrap().as_ptr(),
        )
    };

    // A function to calculate a projection matrix based on the window dimensions and update the GPU with it
    let update_projection = || {
        let w = window.size().0 as f32 / 2.0;
        let h = window.size().1 as f32 / 2.0;

        let projection = Orthographic3::new(-w, w, -h, h, -1.0, 1.0);

        // Write the projection to the GPU
        unsafe {
            gl::UniformMatrix4fv(
                projection_id,
                1,
                gl::FALSE,
                projection.to_homogeneous().as_slice().as_ptr(),
            );
        };
    };

    // Main text to draw
    let text = "Hello World!";

    let get_text_width = |text: &str| {
        let mut width = 0.0;

        // Compute the total width of the string so we can center it
        for (i, c) in text.chars().enumerate() {
            // Get the character from the character map
            let ch: &Character = match char_map.get(&(c as u8)) {
                Some(character) => character,
                None => continue,
            };

            // Add the width of each character (its advance)
            width += ch.advance as f32 / 64.0;

            // Add corrections for first and last character
            if i == 0 {
                width -= ch.bearing.0 as f32;
            } else if i == text.len() - 1 {
                width -= (ch.advance as f32 / 64.0) - (ch.bearing.0 + ch.size.0) as f32;
            }
        }

        width
    };

    // Renders a line of text to the screen at a specified position
    let render_text = |text: &str, ypos: f32| {
        // Render some text to the screen
        let mut x = -get_text_width(text) / 2.0;
        let y = ypos;

        shader_program.set_used();

        gl_util::bind_array(vao);

        for (i, c) in text.chars().enumerate() {
            let ch: &Character = match char_map.get(&(c as u8)) {
                Some(character) => character,
                None => continue,
            };

            // Character units are expressed in 26.6 pixel format (1/64th of a pixel)
            /*
            For pixel perfect font rendering we need to apply the correct transformation to the view space.
            This involves determining the conversion of 'font pixels' to 'double unit cube' coordinates.
            Effectively this is a translation and scaling in the X and Y axes (aka an orthographic projection)
            This is different than the orthographic projection that would be normally used for transforming 'world coordinates'
            to 'view space' coordinates.
            */

            let xpos;

            if i == 0 {
                xpos = x;
            } else {
                xpos = x + ch.bearing.0 as f32;
            }

            let ypos = y - (ch.size.1 - ch.bearing.1) as f32;

            let w = ch.size.0 as f32;
            let h = ch.size.1 as f32;

            let vertices = vec![
                xpos,
                ypos + h,
                0.0,
                0.0,
                xpos,
                ypos,
                0.0,
                1.0,
                xpos + w,
                ypos,
                1.0,
                1.0,
                xpos,
                ypos + h,
                0.0,
                0.0,
                xpos + w,
                ypos,
                1.0,
                1.0,
                xpos + w,
                ypos + h,
                1.0,
                0.0,
            ];

            gl_util::bind_texture(ch.id);
            gl_util::set_buffer_data(vbo, &vertices);
            gl_util::draw_triangles(6);

            x += ch.advance as f32 / 64.0;
        }
    };

    // Go ahead and update the projection
    update_projection();

    // Configure some OpenGL functionality
    unsafe {
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);

        // gl::Enable(gl::CULL_FACE);

        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    };

    let mut cursor_pos = (0, 0);

    // Enter the main event loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'main_loop: loop {
        // Clear the event queue
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(x, y) => unsafe {
                        gl::Viewport(0, 0, x, y);

                        // Compute the projection
                        update_projection();
                    },
                    _ => {}
                },
                Event::MouseMotion { x, y, .. } => {
                    cursor_pos = (x, y);
                }
                _ => {}
            };
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        render_text("Hello World!", 100.0);
        render_text(
            &format!("Cursor: {}, {}", cursor_pos.0, cursor_pos.1),
            -100.0,
        );
        render_text(
            &format!("Window Size: {}, {}", window.size().0, window.size().1),
            0.0,
        );

        gl_util::bind_array(0);
        gl_util::bind_texture(0);

        // Swap the buffers
        window.gl_swap_window();

        let sleep_time = std::time::Duration::from_millis(5);
        std::thread::sleep(sleep_time);
    }
}
