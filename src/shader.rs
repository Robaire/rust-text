extern crate gl;
use gl::types::{GLchar, GLenum, GLint, GLuint};

use std::ffi::CString;
use std::fs;

/// Represents a compiled shader object
pub struct Shader {
    id: GLuint,
    kind: GLenum,
}

impl Drop for Shader {
    /// Deletes the shader from the GPU
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        };
    }
}

impl Shader {
    /// Returns a new shader object from a file
    /// # Arguments
    /// * `path` - A string slice that holds the file path
    /// * `kind` - The type of shader to create, i.e. vertext, fragment, compute, etc...
    pub fn new_from_file(path: &str, kind: GLenum) -> Result<Shader, String> {
        // Read the source file in as a string
        let source = match fs::read_to_string(path) {
            Ok(string) => string,
            Err(message) => panic!(format!("Shader creation failed: {}", message)),
        };

        // Create a shader object on the GPU
        let id = unsafe { gl::CreateShader(kind) };

        // Compile the shader
        unsafe {
            gl::ShaderSource(
                id,
                1,
                &CString::new(source).unwrap().as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(id);
        };

        // Check if the shader compiled
        let mut compile_status: GLint = 1;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut compile_status);
        };

        // Return the shader if it compiled
        if compile_status == 1 {
            Ok(Shader { id, kind })
        } else {
            // Get the length of the error log
            let mut len: i32 = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            };

            // Read the error
            let error = create_cstring(len as u32);
            unsafe {
                gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
            };

            // Return the error
            Err(error.to_string_lossy().into_owned())
        }
    }
}

/// Represents a complete shader program
pub struct Program {
    pub id: GLuint,
    shaders: Vec<GLuint>,
}

impl Program {
    /// Creates a new shader program on the GPU
    pub fn new() -> Program {
        let id = unsafe { gl::CreateProgram() };
        Program {
            id,
            shaders: vec![],
        }
    }

    /// Attaches a shader to a shader program object
    /// * `shader` - The ID of the shader to attach
    pub fn attach_shader(mut self, shader: &Shader) -> Program {
        unsafe {
            gl::AttachShader(self.id, shader.id);
        };
        self.shaders.push(shader.id);
        self
    }

    /// Links the shader program
    pub fn link(mut self) -> Result<Program, String> {
        // Link the program
        unsafe {
            gl::LinkProgram(self.id);
        };

        // Check if the program linked correctly
        let mut link_status: GLint = 1;
        unsafe {
            gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut link_status);
        };

        if link_status == 1 {
            // Detach all shaders after linking
            for shader_id in self.shaders.drain(..) {
                unsafe {
                    gl::DetachShader(self.id, shader_id);
                };
            }

            Ok(self)
        } else {
            // Get the length of the error log
            let mut len: i32 = 0;
            unsafe {
                gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut len);
            };

            // Read the error message
            let error = create_cstring(len as u32);
            unsafe {
                gl::GetProgramInfoLog(
                    self.id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut GLchar,
                );
            };

            // Return the error
            Err(error.to_string_lossy().into_owned())
        }
    }

    /// Sets this as the active shader program
    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        };
    }
}

impl Drop for Program {
    /// Deletes the shader program from the GPU
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        };
    }
}

/// Creates an empty Cstring
/// # Arguments
/// * `len` - The length of the string to create
fn create_cstring(len: u32) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}
