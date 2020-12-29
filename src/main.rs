extern crate gl;
extern crate sdl2;
use sdl2::video::GLProfile;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;

fn init_sdl() -> (sdl2::Sdl, sdl2::video::Window, sdl2::video::GLContext) {

    // Initialize SDL
    let sdl_context = match sdl2::init() {
        Ok(context) => context,
        Err(message) => panic!(format!("SDL Init Failed: {}", message))
    };

    // Ask SDL to initialize the vide system
    let video_subsystem = match sdl_context.video() {
        Ok(video_subsystem) => video_subsystem,
        Err(message) => panic!(format!("Failed to create video subsystem: {}", message))
    };

    // Set the attributes of the OpenGL Context
    let gl_attributes = video_subsystem.gl_attr();
    gl_attributes.set_context_profile(GLProfile::Core);
    gl_attributes.set_context_flags().debug().set();
    gl_attributes.set_context_version(3, 3);

    // Create the window
    let window = match video_subsystem
        .window("Rust Rouge", 600, 600)
        .position_centered()
        .resizable()
        .opengl()
        .build() {
            Ok(window) => window,
            Err(message) => panic!(format!("Failed to create window: {}", message))
        };

    // Create the OpenGL Context
    let gl_context  = match window.gl_create_context() {
        Ok(context) => context,
        Err(message) => panic!(format!("Failed to create OpenGL Context: {}", message))
    };

    // Load the OpenGL Functions
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::ffi::c_void);

    (sdl_context, window, gl_context)
}


fn main() {
    // Initialize Freetype
    let ft_library = freetype::Library::init().unwrap();

    let face = ft_library.new_face("./src/fonts/Pixeletter.ttf", 0).unwrap();

    face.set_char_size(5000, 0, 50, 0).unwrap();

    face.load_char('S' as usize, freetype::face::LoadFlag::RENDER).unwrap();

    let glyph = face.glyph().bitmap();

    println!("({}, {})", glyph.width(), glyph.rows());
    println!("{}", glyph.buffer().len());

    for h in 0..glyph.rows() {
        for w in 0..glyph.width() {

            if glyph.buffer()[(h * 31 + w) as usize] != 0 {
                print!("#");
            } else {
                print!(" ");
            }
            // print!("{}", glyph.buffer()[((w-1) * 31 + h-1) as usize]);

        }

        println!();
    }

    // Initialize SDL and create a window
    let (sdl_context, window, _gl_context) = init_sdl();
}
