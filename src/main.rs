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
    state.layers.push(Layer::new(Rect::new(0, 0, 32, 32)));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        input_state.update(&event);

        // if input_state.key_down(Key::A) {
        //     state.camera.rot_horizontal += 0.02;
        // }
        // if input_state.key_down(Key::D) {
        //     state.camera.rot_horizontal -= 0.02;
        // }
        // if input_state.key_down(Key::W) {
        //     state.camera.rot_vertical -= 0.02;
        //     if state.camera.rot_vertical < 0.001 {
        //         state.camera.rot_vertical = 0.001;
        //     }
        // }
        // if input_state.key_down(Key::S) {
        //     state.camera.rot_vertical += 0.02;
        //     if state.camera.rot_vertical > std::f32::consts::PI {
        //         state.camera.rot_vertical = std::f32::consts::PI - 0.001;
        //     }
        // }

        match event {
            Event::LoopDestroyed => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    // windowed_context.resize(physical_size);
                    gl.resize(physical_size);
                    // gl.set_screen_size(physical_size.width as i32, physical_size.height as i32);
                    // state.aspect_ratio = {
                    //     let size = windowed_context.window().inner_size();
                    //     size.width as f32 / size.height as f32
                    // };
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    let pressed = input.state == ElementState::Pressed;
                    match input.virtual_keycode {
                        // Some(VirtualKeyCode::A) => state.left_pressed = pressed,
                        // Some(VirtualKeyCode::D) => state.right_pressed = pressed,
                        // Some(VirtualKeyCode::W) => state.up_pressed = pressed,
                        // Some(VirtualKeyCode::S) => state.down_pressed = pressed,
                        // Some(Key::T) => {
                        //     if pressed {
                        //         let mut model = load_ldraw_file(&mut gl, &mut parser, "3005.dat", Some([1.0, 0.0, 0.0, 1.0]));
                        //         model.position = new_brick_position;
                        //         new_brick_position.y += 3;
                        //         new_brick_position.z += 1;
                        //         model.set_transform();
                        //         models.push(model);
                        //         state.active_model_idx = models.len() - 1;
                        //     }
                        // }
                        // Some(Key::R) => {
                        //     if pressed {
                        //         models[state.active_model_idx].rotation.y += 1;
                        //         models[state.active_model_idx].rotation_offset.y = 90.0;
                        //         models[state.active_model_idx].set_transform();
                        //     }
                        // }
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
                    // let dx = input_state.mouse_delta_x as f32;
                    // let dy = input_state.mouse_delta_y as f32;
                    // if input_state.mouse_middle_down {
                    //     state.camera.rotate(dx * -0.005, dy * -0.005);
                    // }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                // let p = state.camera.position();
                gl.clear(Color::WHITE);

                gl.draw_rect(state.layers[0].rect, Color::BLACK);
                // gl.draw_rect(input_state.mouse_x as i32, input_state.mouse_y as i32, 100, 100, [0.0, 0.0, 0.0, 1.0]);
                gl.draw_text(
                    "Hello, World!",
                    // gl.window_width - input_state.mouse_x as i32, gl.window_height - input_state.mouse_y as i32, 256.0, [1.0, 0.0, 0.5, 1.0]);
                    20, 20, 20.0, Color::new(255, 0, 0, 255));
                gl.swap();
            },
            _ => (),
        }
    });
}
