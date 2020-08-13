use std::path::Path;
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

mod gui;
use gui::{Widget, Button, ColorSelector, ToolSelector};

struct State {
    layers: Vec<Layer>,
    canvas: Rect,
    canvas_scale: f64,
    canvas_offset_x: i32,
    canvas_offset_y: i32,
    canvas_offset_baseline_x: i32,
    canvas_offset_baseline_y: i32,
    selected_color: Color,
    selected_tool: String,
    currently_drawing: bool,
}

impl State {
    fn new() -> Self {
        Self {
            layers: Vec::new(),
            canvas: Rect::new(100, 100, 800, 600),
            canvas_scale: 2.0,
            canvas_offset_x: 0,
            canvas_offset_y: 0,
            canvas_offset_baseline_x: 0,
            canvas_offset_baseline_y: 0,
            selected_color: Color::BLACK,
            selected_tool: "Pencil".into(),
            currently_drawing: false,
        }
    }
    
    fn screen_to_canvas(&self, x: f64, y: f64) -> (i32, i32) {
        let layer_x = ((x - self.canvas.x as f64 + (self.layers[0].rect.width as f64 * self.canvas_scale / 2.0)) / self.canvas_scale - 0.5).round() as i32;
        let layer_y = ((y - self.canvas.y as f64 + (self.layers[0].rect.height as f64 * self.canvas_scale / 2.0)) / self.canvas_scale - 0.5).round() as i32;
        (layer_x, layer_y)
    }

    fn center_canvas(&mut self, window_width: i32, window_height: i32) {
        self.canvas.x = window_width / 2;
        self.canvas.y = window_height / 2;
    }

    fn update_canvas_position(&mut self) {
        self.canvas.x += self.canvas_offset_x;
        self.canvas.y += self.canvas_offset_y;
    }
}

fn main() {

    let event_loop = EventLoop::new();
    let mut gl = graphics::init(&event_loop);
    let mut input = InputState::new();

    let mut save_button = Button::new(Rect::new(5, 5, 100, 50), "Save".into());
    let mut color_selector = ColorSelector::new(
        Rect::new(100, 100, 50, 1000),
        vec![
            Color::new(0, 0, 0, 255),
            Color::new(70, 70, 70, 255),
            Color::new(120, 120, 120, 255),
            Color::new(153, 0, 48, 255),
            Color::new(237, 28, 36, 255),
            Color::new(255, 126, 0, 255),
            Color::new(255, 194, 14, 255),
            Color::new(255, 242, 0, 255),
            Color::new(168, 230, 29, 255),
            Color::new(34, 177, 76, 255),
            Color::new(0, 183, 239, 255),
            Color::new(77, 109, 243, 255),
            Color::new(47, 54, 153, 255),
            Color::new(111, 49, 152, 255),
            Color::new(255, 255, 255, 255),
            Color::new(220, 220, 220, 255),
            Color::new(180, 180, 180, 255),
            Color::new(156, 90, 60, 255),
            Color::new(255, 163, 177, 255),
            Color::new(229, 170, 122, 255),
            Color::new(145, 228, 156, 255),
            Color::new(255, 249, 189, 255),
            Color::new(211, 249, 188, 255),
            Color::new(157, 187, 97, 255),
            Color::new(153, 217, 234, 255),
            Color::new(112, 154, 209, 255),
            Color::new(84, 109, 142, 255),
            Color::new(181, 165, 213, 255),
        ],
    );
    let mut tool_selector = ToolSelector::new(
        Rect::new(200, 100, 200, 500),
        vec![
            "Pencil".into(),
            "Paintbrush".into(),
            "Color Picker".into(),
            "Paint Bucket".into(),
        ],
    );

    let mut state = State::new();
    state.layers.push(Layer::new(Rect::new(0, 0, 800, 600)));
    // state.layers.push(Layer::from_path(0, 0, "/home/paul/Pictures/420.png"));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        input.update(&event);

        let mut click_intercepted = false;
        if save_button.update(&input, &mut click_intercepted) {
            match state.layers[0].save(Path::new("image.png")) {
                Ok(_) => println!("Saved to image.png"),
                Err(_) => println!("Failed to save!"),
            }
        }
        state.selected_color = color_selector.update(&input, &mut click_intercepted);
        state.selected_tool = tool_selector.update(&input, &mut click_intercepted);

        if input.key_down(Key::Q) {
            *control_flow = ControlFlow::Exit;
        }

        if input.mouse_middle_pressed {
            state.canvas_offset_baseline_x = input.mouse_x as i32;
            state.canvas_offset_baseline_y = input.mouse_y as i32;
        }
        if input.mouse_middle_down {
            state.canvas_offset_x = input.mouse_x as i32 - state.canvas_offset_baseline_x;
            state.canvas_offset_y = input.mouse_y as i32 - state.canvas_offset_baseline_y;
            state.update_canvas_position();
            state.canvas_offset_x = 0;
            state.canvas_offset_y = 0;
            state.canvas_offset_baseline_x = input.mouse_x as i32;
            state.canvas_offset_baseline_y = input.mouse_y as i32;
        }

        if (input.mouse_left_pressed && !click_intercepted) || state.currently_drawing {
            state.currently_drawing = true;

            let (x, y) = state.screen_to_canvas(input.mouse_x, input.mouse_y);
            let old_x = x - (input.mouse_delta_x / state.canvas_scale) as i32;
            let old_y = y - (input.mouse_delta_y / state.canvas_scale) as i32;

            match state.selected_tool.as_str() {
                "Pencil" => state.layers[0].draw_line(old_x, old_y, x, y, state.selected_color),
                "Paintbrush" => {
                    for dx in -10..=10 {
                        for dy in -10..=10 {
                            if (dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt() < 10.0 {
                                state.layers[0].draw_line(old_x + dx, old_y + dy, x + dx, y + dy, state.selected_color);
                            }
                        }
                    }
                }
                "Color Picker" => {
                    if let Some(color) = state.layers[0].get_pixel(x, y) {
                        state.selected_color = color;
                        color_selector.set_selected_color(color);
                    }
                }
                "Paint Bucket" => {
                    state.layers[0].fill(x, y, state.selected_color);
                }
                _ => {}
            }
        }

        if !input.mouse_left_down {
            state.currently_drawing = false;
        }

        match event {
            Event::LoopDestroyed => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    gl.resize(physical_size);
                    state.center_canvas(gl.window_width, gl.window_height);
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    match delta {
                        MouseScrollDelta::LineDelta(_x, y) => {
                            state.canvas_scale *= (10.0 + y as f64) / 10.0;
                        }
                        MouseScrollDelta::PixelDelta(d) => {
                            state.canvas_scale *= (100.0 + d.y as f64) / 100.0;
                        }
                    }
                    state.update_canvas_position();
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                gl.clear(Color::GRAY);
                let rect = state.layers[0].rect;
                let src_rect = rect;
                let dest_rect = Rect::new(
                    state.canvas.x - (rect.width as f64 * state.canvas_scale / 2.0).round() as i32,
                    state.canvas.y - (rect.height as f64 * state.canvas_scale / 2.0).round() as i32,
                    (rect.width as f64 * state.canvas_scale) as u32,
                    (rect.height as f64 * state.canvas_scale) as u32,
                );
                gl.draw_texture(src_rect, dest_rect, state.layers[0].data.clone().into_raw());
                save_button.draw(&gl);
                color_selector.draw(&gl);
                tool_selector.draw(&gl);
                gl.swap();
            },
            _ => (),
        }
    });
}
