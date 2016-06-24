extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;

extern crate vitral;

use std::path::Path;
use glium::{DisplayBuild, Surface};
use glium::glutin;
use glium::index::PrimitiveType;
use euclid::Point2D;

use vitral::Context;

type GliumTexture = glium::texture::CompressedSrgbTexture2d;

struct Backend {
    display: glium::Display,
    program: glium::Program,
    textures: Vec<GliumTexture>,
}

impl Backend {
    pub fn new(display: glium::Display, program: glium::Program) -> Backend {
        Backend {
            display: display,
            program: program,
            textures: Vec::new(),
        }
    }

    pub fn make_texture(&mut self, img: vitral::ImageBuffer) -> usize {
        let dim = (img.width(), img.height());
        let raw = glium::texture::RawImage2d::from_raw_rgba(img.into_raw(),
                                                            dim);
        let tex = glium::texture::CompressedSrgbTexture2d::new(&self.display,
                                                               raw)
                      .unwrap();
        self.textures.push(tex);
        self.textures.len() - 1
    }

    fn process_events<V>(&self, context: &mut Context<usize, V>) -> bool
        where V: vitral::Vertex
    {
        // polling and handling the events received by the window
        for event in self.display.poll_events() {
            match event {
                glutin::Event::Closed => return false,
                glutin::Event::MouseMoved(x, y) => {
                    context.input_mouse_move(x, y)
                }
                glutin::Event::MouseInput(state, button) => {
                    context.input_mouse_button(match button {
                                                   glutin::MouseButton::Left => {
                                                       vitral::MouseButton::Left
                                                   }
                                                   glutin::MouseButton::Right => {
                                                       vitral::MouseButton::Right
                                                   }
                                                   _ => vitral::MouseButton::Middle,
                                               },
                                               state == glutin::ElementState::Pressed)
                }
                glutin::Event::ReceivedCharacter(c) => context.input_char(c),
                glutin::Event::KeyboardInput(s, _, Some(vk)) => {
                    let is_down = s == glutin::ElementState::Pressed;
                    use glium::glutin::VirtualKeyCode::*;
                    match vk {
                        Tab => {
                            context.input_key_state(vitral::Keycode::Tab,
                                                    is_down)
                        }
                        LShift | RShift => {
                            context.input_key_state(vitral::Keycode::Shift,
                                                    is_down)
                        }
                        LControl | RControl => {
                            context.input_key_state(vitral::Keycode::Ctrl,
                                                    is_down)
                        }
                        NumpadEnter | Return => {
                            context.input_key_state(vitral::Keycode::Enter,
                                                    is_down)
                        }
                        Back => {
                            context.input_key_state(vitral::Keycode::Backspace,
                                                    is_down)
                        }
                        Delete => {
                            context.input_key_state(vitral::Keycode::Del,
                                                    is_down)
                        }
                        Numpad8 | Up => {
                            context.input_key_state(vitral::Keycode::Up,
                                                    is_down)
                        }
                        Numpad2 | Down => {
                            context.input_key_state(vitral::Keycode::Down,
                                                    is_down)
                        }
                        Numpad4 | Left => {
                            context.input_key_state(vitral::Keycode::Left,
                                                    is_down)
                        }
                        Numpad6 | Right => {
                            context.input_key_state(vitral::Keycode::Right,
                                                    is_down)
                        }
                        _ => {}
                    }
                }
                _ => (),
            }
        }

        true
    }

    pub fn update<V>(&self, context: &mut Context<usize, V>) -> bool
        where V: vitral::Vertex + glium::Vertex
    {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        let (w, h) = target.get_dimensions();

        for batch in context.end_frame() {
            // building the uniforms
            let uniforms = uniform! {
                matrix: [
                    [2.0 / w as f32, 0.0, 0.0, -1.0],
                    [0.0, -2.0 / h as f32, 0.0, 1.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ],
                tex: glium::uniforms::Sampler::new(&self.textures[batch.texture])
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };

            let vertex_buffer = {
                glium::VertexBuffer::new(&self.display, &batch.vertices)
                    .unwrap()
            };

            // building the index buffer
            let index_buffer =
                glium::IndexBuffer::new(&self.display,
                                        PrimitiveType::TrianglesList,
                                        &batch.triangle_indices)
                    .unwrap();

            let params = glium::draw_parameters::DrawParameters {
                scissor: batch.clip.map(|clip| {
                    glium::Rect {
                        left: clip.origin.x as u32,
                        bottom: h - (clip.origin.y + clip.size.height) as u32,
                        width: clip.size.width as u32,
                        height: clip.size.height as u32,
                    }
                }),
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            };

            target.draw(&vertex_buffer,
                        &index_buffer,
                        &self.program,
                        &uniforms,
                        &params)
                  .unwrap();
        }

        target.finish().unwrap();

        self.process_events(context)
    }
}


// XXX: An exact copy of Vitral vertex struct, just so that I can derive a
// Glium vertex implementatino for it.
#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
    pub tex: [f32; 2],
}
implement_vertex!(Vertex, pos, color, tex);

impl vitral::Vertex for Vertex {
    fn new(pos: [f32; 2], color: [f32; 4], texcoord: [f32; 2]) -> Self {
        Vertex {
            pos: pos,
            color: color,
            tex: texcoord,
        }
    }
}

fn main() {
    // Construct Glium backend.
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    // building the display, ie. the main object
    let program = program!(
        &display,
        150 => {
        vertex: "
            #version 150 core

            uniform mat4 matrix;

            in vec2 pos;
            in vec4 color;
            in vec2 tex;

            out vec4 vColor;
            out vec2 vTexcoord;

            void main() {
                gl_Position = vec4(pos, 0.0, 1.0) * matrix;
                vColor = color;
                vTexcoord = tex;
            }
        ",

        fragment: "
            #version 150 core
            uniform sampler2D tex;
            in vec4 vColor;
            in vec2 vTexcoord;
            out vec4 f_color;

            void main() {
                f_color = vColor * texture(tex, vTexcoord);
            }
        "})
                      .unwrap();

    let mut backend = Backend::new(display, program);

    // Construct Vitral context.
    let mut context: Context<usize, Vertex>;
    let mut builder = vitral::Builder::new();
    let image = builder.add_image(&image::open(&Path::new("julia.png"))
                                       .unwrap());
    context = builder.build(|img| backend.make_texture(img));

    let font = context.default_font();

    let mut test_input = String::new();

    // Run the program.
    loop {
        context.begin_frame();

        context.draw_image(image,
                           Point2D::new(100.0, 100.0),
                           [1.0, 1.0, 1.0, 1.0]);

        if context.button("Hello, world") {
            println!("Click");
        }

        if context.button("Another button") {
            println!("Clack {}", test_input);
        }

        context.text_input(font,
                           Point2D::new(10.0, 120.0),
                           [0.8, 0.8, 0.8, 1.0],
                           &mut test_input);

        if !backend.update(&mut context) {
            return;
        }
    }
}
