use std::ffi::CString;
use std::{ptr, mem};
use cgmath::{Matrix4, Vector2, Deg, Vector3, Point3, SquareMatrix, Vector4};
use glutin::{self, PossiblyCurrent};
use self::gl::types::*;
use rusttype::{point, Scale};

const VS_SRC_2D: &[u8] = b"
#version 330 core

layout (location = 0) in vec2 position;
layout (location = 1) in vec4 color;

out vec4 v_color;

void main() {
    v_color = color;
    gl_Position = vec4(position, 0.0, 1.0);
}
\0";

const FS_SRC_2D: &[u8] = b"
#version 330 core

in vec4 v_color;

void main() {
    gl_FragColor = v_color;
}
\0";

const VS_SRC: &[u8] = b"
#version 330 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec4 color;

uniform mat4 world;
uniform mat4 view;
uniform mat4 proj;

out vec3 v_normal;
out vec4 v_color;
out vec3 fragment_position;

void main() {
    mat4 worldview = view * world;
    v_normal = transpose(inverse(mat3(worldview))) * normal;
    v_color = color;
    // TODO check if world is correct to use below, original said model
    fragment_position = vec3(world * vec4(position, 1.0));
    // fragment_position = position;
    gl_Position = proj * worldview * vec4(position, 1.0);
}
\0";

const FS_SRC: &[u8] = b"
#version 330 core

in vec3 v_normal;
in vec4 v_color;
in vec3 fragment_position;

struct Light {
    vec3 position;
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform vec3 view_position;
uniform Light light;

// const vec3 LIGHT = vec3(1.0, 1.0, 1.0);

void main() {

    vec3 norm = normalize(v_normal);
    vec3 light_direction = normalize(light.position - fragment_position);
    vec3 view_direction = normalize(view_position - fragment_position);
    vec3 reflection_direction = reflect(-light_direction, norm);

    vec3 ambient = light.ambient; 
    vec3 diffuse = light.diffuse * max(dot(norm, light_direction), 0.0);
    vec3 specular = light.specular * pow(max(dot(view_direction, reflection_direction), 0.0), 32);

    gl_FragColor = vec4((ambient + diffuse + specular), 1.0) * v_color;
}
\0";

const VS_SRC_2D_TEXTURE: &[u8] = b"
#version 330 core

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 tex_coords;
layout (location = 2) in vec4 color;

out vec2 v_tex_coords;
out vec4 v_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_tex_coords = tex_coords;
    v_color = color;
}
\0";

const FS_SRC_2D_TEXTURE: &[u8] = b"
#version 330 core

uniform sampler2D tex;
in vec2 v_tex_coords;
in vec4 v_color;
out vec4 f_color;

void main() {
    f_color = v_color * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
}
\0";

pub struct Camera {
    pub focus: Point3<f32>,
    pub distance: f32,
    pub rot_horizontal: f32,
    pub rot_vertical: f32,
    pub fovy: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            focus: Point3::new(0.0, 0.0, 0.0),
            distance: 10.0,
            rot_horizontal: 0.5,
            rot_vertical: 0.5,
            fovy: 45.0,
        }
    }

    pub fn rotate(&mut self, horizontal: f32, vertical: f32) {
        self.rot_horizontal += horizontal;
        self.rot_vertical += vertical;
        if self.rot_vertical < 0.001 {
            self.rot_vertical = 0.001;
        }
        if self.rot_vertical > std::f32::consts::PI {
            self.rot_vertical = std::f32::consts::PI - 0.001;
        }
    }

    pub fn position(&self) -> Point3<f32> {
        Point3::new(
            self.focus.z + self.distance * self.rot_vertical.sin() * self.rot_horizontal.sin(),
            self.focus.y + self.distance * self.rot_vertical.cos(),
            self.focus.x + self.distance * self.rot_vertical.sin() * self.rot_horizontal.cos()
        )
    }
}

pub struct Model {
    pub vao: u32,
    pub vertex_buffer_length: i32,
    pub position: Vector3<i32>,
    pub rotation: Vector3<i32>,
    pub transform: Matrix4<f32>,
    pub position_offset: Vector3<f32>,
    pub rotation_offset: Vector3<f32>,
    pub bounding_box: BoundingBox,
}

impl Model {
    pub fn set_transform(&mut self) {
        let position = Vector3::new(self.position.x as f32 * 0.5, self.position.y as f32 * 0.2, self.position.z as f32 * 0.5);
        self.transform = Matrix4::from_translation(position - self.position_offset)
            * Matrix4::from_angle_x(Deg((self.rotation.x * 90) as f32 - self.rotation_offset.x))
            * Matrix4::from_angle_y(Deg((self.rotation.y * 90) as f32 - self.rotation_offset.y))
            * Matrix4::from_angle_z(Deg((self.rotation.z * 90) as f32 - self.rotation_offset.z))
    }
}

pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

pub struct Uniforms {
    world: GLint,
    view: GLint,
    proj: GLint,
    view_position: GLint,
    light_position: GLint,
    light_direction: GLint,
    light_ambient: GLint,
    light_diffuse: GLint,
    light_specular: GLint,
}

fn unproject(source: Vector3<f32>, view: Matrix4<f32>, proj: Matrix4<f32>) -> Vector3<f32> {
    let view_proj = (proj * view).invert().unwrap();
    let q = view_proj * Vector4::new(source.x, source.y, source.z, 1.0);
    Vector3::new(q.x / q.w, q.y / q.w, q.z / q.w)
}

fn get_mouse_ray(aspect_ratio: f32, mouse_position: Vector2<f32>, camera: &Camera) -> (Point3<f32>, Vector3<f32>) {
    let view = Matrix4::look_at(camera.position(), camera.focus, Vector3::new(0.0, 1.0, 0.0));
    let proj = cgmath::perspective(Deg(camera.fovy), aspect_ratio, 0.01, 100.0);
    let near = unproject(Vector3::new(mouse_position.x, mouse_position.y, 0.0), view, proj);
    let far = unproject(Vector3::new(mouse_position.x, mouse_position.y, 1.0), view, proj);
    let direction = far - near;
    (camera.position(), direction)
}

pub mod gl {
    pub use self::Gles2 as Gl;
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub struct Graphics {
    pub window_width: i32,
    pub window_height: i32,
    program: u32,
    program_2d: u32,
    pub gl: gl::Gl,
    uniforms: Uniforms,
    font: rusttype::Font<'static>,
}

fn create_shader(gl: &gl::Gl, shader_type: u32, source: &'static [u8]) -> u32 {
    unsafe {
        let id = gl.CreateShader(shader_type);
        gl.ShaderSource(
            id,
            1,
            [source.as_ptr() as *const _].as_ptr(),
            std::ptr::null()
        );
        gl.CompileShader(id);
        let mut success: gl::types::GLint = 1;
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            let error = {
                let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
                buffer.extend([b' '].iter().cycle().take(len as usize));
                CString::from_vec_unchecked(buffer)
            };
            gl.GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
            eprintln!("{}", error.to_string_lossy());
        }
        id
    }
}

fn create_program(
    gl: &gl::Gl,
    vertex_shader: &'static [u8],
    fragment_shader: &'static [u8],
) -> u32 {
    let vs = create_shader(gl, gl::VERTEX_SHADER, vertex_shader);
    let fs = create_shader(gl, gl::FRAGMENT_SHADER, fragment_shader);
    
    unsafe {
        let program = gl.CreateProgram();
        gl.AttachShader(program, vs);
        gl.AttachShader(program, fs);
        gl.LinkProgram(program);
        let mut success: gl::types::GLint = 1;
        gl.GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl.GetShaderiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let error = {
                let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
                buffer.extend([b' '].iter().cycle().take(len as usize));
                CString::from_vec_unchecked(buffer)
            };
            gl.GetProgramInfoLog(program, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
            eprintln!("{}", error.to_string_lossy());
        }
        gl.DeleteShader(vs);
        gl.DeleteShader(fs);
        program
    }

}

pub fn init(
    gl_context: &glutin::Context<PossiblyCurrent>,
    window_width: i32,
    window_height: i32,
) -> Graphics {

    let gl = gl::Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);
    let font = rusttype::Font::try_from_bytes(include_bytes!("/usr/share/fonts/TTF/DejaVuSans.ttf") as &[u8]).unwrap();

    unsafe {
        gl.Enable(gl::DEPTH_TEST);
        gl.DepthFunc(gl::LESS);
        gl.Disable(gl::CULL_FACE);
        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);

        gl.Viewport(0, 0, window_width, window_height);
    }

    let program = create_program(&gl, VS_SRC, FS_SRC);
    let program_2d = create_program(&gl, VS_SRC_2D, FS_SRC_2D);

    let uniforms = unsafe {
        Uniforms {
            world: gl.GetUniformLocation(program, b"world\0".as_ptr() as *const _),
            view: gl.GetUniformLocation(program, b"view\0".as_ptr() as *const _),
            proj: gl.GetUniformLocation(program, b"proj\0".as_ptr() as *const _),
            view_position: gl.GetUniformLocation(program, b"view_position\0".as_ptr() as *const _),
            light_position: gl.GetUniformLocation(program, b"light.position\0".as_ptr() as *const _),
            light_direction: gl.GetUniformLocation(program, b"light.direction\0".as_ptr() as *const _),
            light_ambient: gl.GetUniformLocation(program, b"light.ambient\0".as_ptr() as *const _),
            light_diffuse: gl.GetUniformLocation(program, b"light.diffuse\0".as_ptr() as *const _),
            light_specular: gl.GetUniformLocation(program, b"light.specular\0".as_ptr() as *const _),
        }
    };
    Graphics {
        window_height,
        window_width,
        program,
        program_2d,
        gl,
        uniforms,
        font,
    }
}

impl Graphics {

    pub fn clear(&self, color: [f32; 4]) {
        unsafe {
            self.gl.ClearColor(color[0], color[1], color[2], color[3]);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn draw_rect(&self, x: i32, y: i32, width: i32, height: i32, color: [f32; 4]) {
        let gl = &self.gl;

        let x = x as f32 * 2.0 / self.window_width as f32 - 1.0;
        let y = 1.0 - y as f32 * 2.0 / self.window_height as f32;
        let width = width as f32 * 2.0 / self.window_width as f32;
        let height = -1.0 * height as f32 * 2.0 / self.window_height as f32;

        let (mut vao_2d, mut vbo_2d) = (0, 0);
        let vertices = [
            x, y, color[0], color[1], color[2], color[3],
            x + width, y, color[0], color[0], color[2], color[3],
            x + width, y + height, color[0], color[0], color[2], color[3],
            x, y, color[0], color[1], color[2], color[3],
            x + width, y + height, color[0], color[0], color[2], color[3],
            x, y + height, color[0], color[0], color[2], color[3], 
        ];
        unsafe {
            gl.Enable(gl::DEPTH_TEST);

            gl.GenVertexArrays(1, &mut vao_2d);
            gl.GenBuffers(1, &mut vbo_2d);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo_2d);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );
            gl.BindVertexArray(vao_2d);
            let stride = 6 * mem::size_of::<GLfloat>() as GLsizei;
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const _);

            self.gl.UseProgram(self.program_2d);
            self.gl.BindVertexArray(vao_2d);
            self.gl.DrawArrays(gl::TRIANGLES, 0, vertices.len() as GLsizei);

            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);
        }
    }

    // TODO remove this function, figure out what we want to do with drawing 2d
    pub fn draw_2d(&self) {
        unsafe {
            // 2d
            self.gl.BindVertexArray(0);
        }
    }

    pub fn set_screen_size(&mut self, x: i32, y: i32) {
        unsafe {
            self.gl.Viewport(0, 0, x, y);
        }
        self.window_width = x;
        self.window_height = y;
    }

    // pub fn draw_bounding_box(&self, a: [f32; 3], b: [f32; 3], world: [f32; 16], view: [f32; 16], proj: [f32; 16], view_position: [f32; 3], light: [f32; 15]) {
    //     let c = vec![0.0, 1.0, 1.0, 0.3];
    //     let vertices = vec![
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], a[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         a[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], b[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3],
    //         b[0], a[1], b[2], 0.0, 1.0, 0.0, c[0], c[1], c[2], c[3]
    //     ];
    //     self.draw_model(&vertices, world, view, proj, view_position, light);
    // }

    pub fn load_model(&mut self, vertices: &[f32]) -> (u32, i32) {
        // TODO there is no "unload_model" right now because this is meant to be run once for each
        // model, and all the memory can be cleaned up when the program exits.
        let gl = &self.gl;
        let (mut vao, mut vbo) = (0, 0);
        unsafe {
            gl.GenVertexArrays(1, &mut vao);
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );
            gl.BindVertexArray(vao);
            let stride = 10 * mem::size_of::<GLfloat>() as GLsizei;
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const _);
            gl.EnableVertexAttribArray(2);
            gl.VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<GLfloat>()) as *const _);
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);
        }
        (vao, vertices.len() as i32)
    }

    pub fn start_3d(&self) {
        unsafe {
            self.gl.UseProgram(self.program);
        }
    }

    pub fn draw_model(&self, vao: GLuint, vertex_buffer_length: i32, world: [f32; 16], view: [f32; 16], proj: [f32; 16], view_position: [f32; 3], light: [f32; 15]) {
        let gl = &self.gl;
        unsafe {
            gl.Enable(gl::DEPTH_TEST);
            gl.BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            // gl.UseProgram(self.program);

            gl.UniformMatrix4fv(self.uniforms.world, 1, gl::FALSE, world.as_ptr());
            gl.UniformMatrix4fv(self.uniforms.view, 1, gl::FALSE, view.as_ptr());
            gl.UniformMatrix4fv(self.uniforms.proj, 1, gl::FALSE, proj.as_ptr());
            gl.Uniform3f(self.uniforms.view_position, view_position[0], view_position[1], view_position[2]);
            gl.Uniform3f(self.uniforms.light_position, light[0], light[1], light[2]);
            gl.Uniform3f(self.uniforms.light_direction, light[3], light[4], light[5]);
            gl.Uniform3f(self.uniforms.light_ambient, light[6], light[7], light[8]);
            gl.Uniform3f(self.uniforms.light_diffuse, light[9], light[10], light[11]);
            gl.Uniform3f(self.uniforms.light_specular, light[12], light[13], light[14]);

            gl.BindVertexArray(vao);
            gl.DrawArrays(gl::TRIANGLES, 0, vertex_buffer_length as GLsizei);
            // gl.BindVertexArray(0);
            gl.Disable(gl::DEPTH_TEST);
        }
    }

    pub fn draw_text(&self, text: &str, x: i32, y: i32, scale: f32, color: [f32; 4]) {
        let gl = &self.gl;

        let x = x as f32 * 2.0 / self.window_width as f32 - 1.0;
        let y = y as f32 * 2.0 / self.window_height as f32 - 1.0;

        // Draw the font data into a buffer
        let font_scale = Scale::uniform(scale);
        let v_metrics = self.font.v_metrics(font_scale);
        let glyphs: Vec<_> = self.font
            .layout(text, font_scale, point(x, y + v_metrics.ascent))
            .collect();

        let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as usize;
        let glyphs_width = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;

        let mut buffer: Vec<f32> = vec![0.0; glyphs_width * glyphs_height];

        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {

                let min_x = bounding_box.min.x as usize;
                let min_y = bounding_box.min.y as usize;

                glyph.draw(|x, y, v| {
                    let x = x as usize + min_x - 1;
                    let y = y as usize + min_y - 1;
                    let index = y * glyphs_width + x;
                    buffer[index] = v;
                });
            }
        }

        // Load the texture from the buffer
        let (program, uniform, id) = unsafe {
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            let mut id: u32 = 0;
            gl.GenTextures(1, &mut id);
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, id);

            // TODO Decide what these should be.
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as GLint,
                glyphs_width as GLint,
                glyphs_height as GLint,
                0,
                gl::RED,
                gl::FLOAT,
                buffer.as_ptr() as *const _
            );
            let program = create_program(gl, VS_SRC_2D_TEXTURE, FS_SRC_2D_TEXTURE);
            let uniform = gl.GetUniformLocation(program, b"tex\0".as_ptr() as *const _);
            (program, uniform, id)
        };

        let height = glyphs_height as f32 * 2.0 / self.window_height as f32;
        let width = glyphs_width as f32 / self.window_width as f32;
        let vertices = [
            x, y, 0.0, 1.0, color[0], color[1], color[2], color[3],
            x + width, y, 1.0, 1.0, color[0], color[1], color[2], color[3],
            x + width, y + height, 1.0, 0.0, color[0], color[1], color[2], color[3],
            x, y, 0.0, 1.0, color[0], color[1], color[2], color[3],
            x + width, y + height, 1.0, 0.0, color[0], color[1], color[2], color[3],
            x, y + height, 0.0, 0.0, color[0], color[1], color[2], color[3],
        ];

        let (mut vao, mut vbo) = (0, 0);
        unsafe {
            gl.GenVertexArrays(1, &mut vao);
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );
            gl.BindVertexArray(vao);
            let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;

            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, id);
            gl.Uniform1i(uniform, 0);

            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const _);
            gl.EnableVertexAttribArray(2);
            gl.VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (4 * mem::size_of::<GLfloat>()) as *const _);

            gl.UseProgram(program);

            gl.DrawArrays(gl::TRIANGLES, 0, vertices.len() as GLsizei);

            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);
        }
    }
}
