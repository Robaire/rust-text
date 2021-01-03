extern crate gl;
use gl::types::{GLchar, GLenum, GLint, GLuint};

extern crate sdl2;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::video::GLProfile;

use std::collections::HashMap;

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

    (sdl_context, window, gl_context)
}

struct Character {
    id: GLuint,
    size: (u32, u32),
    bearing: (u32, u32),
    advance: u32,
}

fn main() {
    // Initialize SDL and create a window
    let (sdl_context, window, _gl_context) = init_sdl();

    // Initialize Freetype
    let ft_library = freetype::Library::init().unwrap();

    // Load the font face we want to use
    let face = match ft_library.new_face("./src/fonts/KottaOne.ttf", 0) {
        Ok(face) => face,
        Err(message) => panic!("Unable to open font: {}", message),
    };

    // Set the font height in pixels
    face.set_pixel_sizes(0, 50).unwrap();

    // Create the map that will store our character textures
    let mut char_map: HashMap<u8, Character> = HashMap::new();

    // Generate a texture for every ascii character

    // Disable byte-alignment
    unsafe {
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    }

    for c in 0..128 {
        // Attempt to load the glyph
        // This can fail for some ascii characters that are not displayed
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
                glyph.bitmap().buffer().as_ptr() as *const gl::types::GLvoid
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
                id: 0,
                size: (glyph.bitmap().width() as u32, glyph.bitmap().rows() as u32),
                bearing: (0, 0),
                advance: 0,
            },
        );
    }

    face.load_char('G' as usize, freetype::face::LoadFlag::RENDER)
        .unwrap();

    let glyph = face.glyph().bitmap();

    println!(
        "Width: {}, Rows: {}, Pitch: {}",
        glyph.width(),
        glyph.rows(),
        glyph.pitch()
    );
    println!("{}", glyph.buffer().len());

    for h in 0..glyph.rows() {
        for w in 0..glyph.width() {
            if glyph.buffer()[(h * glyph.width() + w) as usize] != 0 {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!();
    }
}
