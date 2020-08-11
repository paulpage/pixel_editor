use glutin::event::{Event, WindowEvent, MouseScrollDelta};
use glutin::event::VirtualKeyCode as Key;
use glutin::event_loop::{ControlFlow, EventLoop};

mod layer;
use layer::Layer;

mod graphics;

mod input;
use input::InputState;

mod util;
use util::{Rect, Color};

struct State {
    layers: Vec<Layer>,
    image_x: i32,
    image_y: i32,
    image_scale: f64,
}

impl State {
    fn new() -> Self {
        Self {
            layers: Vec::new(),
            image_x: 100,
            image_y: 100,
            image_scale: 3.0,
        }
    }
}

fn main() {

    let event_loop = EventLoop::new();
    let mut gl = graphics::init(&event_loop);
    let mut input = InputState::new();

    let mut state = State::new();
    // state.layers.push(Layer::new(Rect::new(0, 0, 32, 32)));
    state.layers.push(Layer::from_path(0, 0, "/home/paul/Pictures/420.png"));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        input.update(&event);

        if input.key_down(Key::Q) {
            *control_flow = ControlFlow::Exit;
        }

        if input.mouse_left_down {
            let x = ((input.mouse_x - state.image_x as f64) / state.image_scale) as i32;
            let y = ((input.mouse_y - state.image_y as f64) / state.image_scale) as i32;
            let old_x = x - (input.mouse_delta_x / state.image_scale) as i32;
            let old_y = y - (input.mouse_delta_y / state.image_scale) as i32;
            state.layers[0].draw_line(old_x, old_y, x, y, Color::new(255, 255, 0, 255));
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
                WindowEvent::MouseWheel { delta, .. } => {
                    match delta {
                        MouseScrollDelta::LineDelta(_x, _y) => {
                            // state.camera.distance *= (10.0 - y as f32) / 10.0;
                        }
                        MouseScrollDelta::PixelDelta(_d) => {
                            // state.camera.distance *= (100.0 - d.y as f32) / 100.0;
                        }
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                gl.clear(Color::WHITE);
                gl.draw_rect(Rect::new(500, 500, 100, 100), Color::BLACK);
                let rect = state.layers[0].rect;
                let src_rect = rect;
                let dest_rect = Rect::new(
                    state.image_x,
                    state.image_y,
                    (rect.width as f64 * state.image_scale) as u32,
                    (rect.height as f64 * state.image_scale) as u32,
                );
                gl.draw_texture(src_rect, dest_rect, state.layers[0].data.clone().into_raw());
                gl.draw_text(
                    "Hello, World!",
                    20, 20, 100.0, Color::new(255, 0, 0, 255));
                gl.swap();
            },
            _ => (),
        }
    });
}
