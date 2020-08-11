use glutin::event::{Event, WindowEvent, ElementState, MouseScrollDelta};
use glutin::event::VirtualKeyCode as Key;
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use cgmath::{Matrix4, Deg, Vector3, Point3, SquareMatrix};
use std::time::Instant;

mod layer;
use layer::Layer;

mod graphics;
use graphics::{BoundingBox, Camera, Graphics, Model};

mod input;
use input::InputState;

mod util;
use util::{Rect, Color};

struct State {
    layers: Vec<Layer>,
}

impl State {
    fn new() -> Self {
        Self {
            layers: Vec::new(),
        }
    }
}

fn main() {

    let event_loop = EventLoop::new();
    let mut gl = graphics::init(&event_loop);
    let mut input_state = InputState::new();

    let mut state = State::new();
    // state.layers.push(Layer::new(Rect::new(0, 0, 32, 32)));
    state.layers.push(Layer::from_path(0, 0, "/home/paul/Pictures/420.png"));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        input_state.update(&event);

        if input_state.key_down(Key::Q) {
            *control_flow = ControlFlow::Exit;
        }

        match event {
            Event::LoopDestroyed => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    gl.resize(physical_size);
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    let pressed = input.state == ElementState::Pressed;
                    match input.virtual_keycode {
                        // Some(Key::A) => state.left_pressed = pressed,
                        _ => {}
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    match delta {
                        MouseScrollDelta::LineDelta(_x, y) => {
                            // state.camera.distance *= (10.0 - y as f32) / 10.0;
                        }
                        MouseScrollDelta::PixelDelta(d) => {
                            // state.camera.distance *= (100.0 - d.y as f32) / 100.0;
                        }
                    }
                }
                // WindowEvent::MouseInput { button, state: mouse_state, .. } => {
                //     let pressed = mouse_state == ElementState::Pressed;
                //     match button {
                //         MouseButton::Middle => state.middle_pressed = pressed,
                //         _ => {}
                //     }
                // }
                WindowEvent::CursorMoved { position, .. } => {
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                gl.clear(Color::WHITE);
                gl.draw_rect(Rect::new(500, 500, 100, 100), Color::BLACK);
                gl.draw_texture(state.layers[0].rect, state.layers[0].data.clone().into_raw());
                gl.draw_text(
                    "Hello, World!",
                    20, 20, 100.0, Color::new(255, 0, 0, 255));
                gl.swap();
            },
            _ => (),
        }
    });
}
