extern crate gl;
use gl::types::{GLchar, GLenum, GLint, GLuint};

extern crate sdl2;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::video::GLProfile;

use std::collections::HashMap;

extern crate nalgebra;
use nalgebra::Orthographic3;

fn init_sdl() -> (sdl2::Sdl, sdl2::video::Window, sdl2::video::GLContext) {
    // Initialize SDL
    let sdl_context = match sdl2::init() {
        Ok(context) => context,
        Err(message) => panic!("SDL Init Failed: {}", message),
    };

    // Ask SDL to initialize the vide system
    let video_subsystem = match sdl_context.video() {
        Ok(video_subsystem) => video_subsystem,
        Err(message) => panic!("Failed to create video subsystem: {}", message),
    };

    // Set the attributes of the OpenGL Context
    let gl_attributes = video_subsystem.gl_attr();
    gl_attributes.set_context_profile(GLProfile::Core);
    gl_attributes.set_context_flags().debug().set();
    gl_attributes.set_context_version(3, 3);

    // Create the window
    let window = match video_subsystem
        .window("Rust Font Rendering", 600, 600)
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

    return (sdl_context, window, gl_context);
}

struct Character {
    id: GLuint,
    size: (i32, i32),
    bearing: (i32, i32),
    advance: i32,
}

fn init_freetype(font: &str) -> HashMap<u8, Character> {
    // Initialize Freetype
    let ft_library = freetype::Library::init().unwrap();

    // Load the font face we want to use
    let face = match ft_library.new_face(font, 0) {
        Ok(face) => face,
        Err(message) => panic!("Unable to open font: {}", message),
    };

    // Set the font height in pixels
    face.set_pixel_sizes(0, 50).unwrap();

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
        let mut id = 0;
        unsafe {
            // Create a texture
            gl::GenTextures(1, &mut id);
            assert_ne!(id, 0);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // Upload bitmap data to the texture
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as i32,
                glyph.bitmap().width(),
                glyph.bitmap().rows(),
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                glyph.bitmap().buffer().as_ptr() as *const gl::types::GLvoid,
            );

            // Set texture options
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
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

    return char_map;
}

fn main() {
    // Initialize SDL and create a window
    let (sdl_context, window, _gl_context) = init_sdl();

    let char_map = init_freetype("./src/fonts/KottaOne.ttf");

    

    // Set the projection matrix
    let projection_id = unsafe {
        gl::GetUniformLocation(
            shader_program.id,
            CString::new("projection").unwrap().as_ptr()
        )
    };

    let aspect = 1.0;
    let projection = Orthographic3::new(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);

    // Write the projection to the gpu
    unsafe {
        gl::UniformMatrix4fv(
            projection_id,
            1,
            gl::FALSE,
            projection.to_homogeneous().as_slice().as_ptr()
        );
    };
    
    // Last Bit
    unsafe {
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);

        gl::Enable(gl::CULL_FACE);

        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    };

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
                        let aspect = x as f32 / y as f32;
                        let projection = Orthographic3::new(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);

                        // Write the projection to the gpu
                        shader_program.set_used();
                        gl::UniformMatrix4fv(
                            projection_id,
                            1,
                            gl::FALSE,
                            projection.to_homogeneous().as_slice().as_ptr(),
                        );
                    },
                    _ => {}
                },
                _ => {}
            };
        }

        // Swap the buffers
        window.gl_swap_window();

        let sleep_time = std::time::Duration::from_millis(5);
        std::thread::sleep(sleep_time);
    }
}
