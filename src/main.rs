use std::path::Path;
use glutin::event::{Event, WindowEvent, MouseScrollDelta};
use glutin::event::VirtualKeyCode as Key;
use glutin::event_loop::{ControlFlow, EventLoop};
// use nfd::Response as FileDialogResponse;

mod layer;
use layer::{Image, ImageHistory, Layer};

mod graphics;

mod input;
use input::InputState;

mod util;
use util::{Rect, Color};

mod gui;
use gui::{Widget, Button, ColorSelector, ToolSelector, NewDialog, OpenDialog, SaveDialog, ConfirmationDialog};

struct State {
    image: Image,
    active_layer_idx: usize,
    canvas: Rect,
    canvas_scale: f64,
    canvas_offset_x: i32,
    canvas_offset_y: i32,
    canvas_offset_baseline_x: i32,
    canvas_offset_baseline_y: i32,
    selected_color: Color,
    selected_tool: String,
    currently_drawing: bool,
    // TODO better way to handle dialog boxes please
    showing_new_dialog: bool,
    showing_open_dialog: bool,
    showing_save_dialog: bool,
    error_text: String,
}

impl State {
    fn new() -> Self {
        Self {
            image: Image::new(800, 600),
            active_layer_idx: 0,
            canvas: Rect::new(100, 100, 800, 600),
            canvas_scale: 2.0,
            canvas_offset_x: 0,
            canvas_offset_y: 0,
            canvas_offset_baseline_x: 0,
            canvas_offset_baseline_y: 0,
            selected_color: Color::BLACK,
            selected_tool: "Pencil".into(),
            currently_drawing: false,
            showing_new_dialog: false,
            showing_open_dialog: false,
            showing_save_dialog: false,
            error_text: "".into(),
        }
    }
    
    fn screen_to_canvas(&self, x: f64, y: f64) -> (i32, i32) {
        let layer_x = ((x - self.canvas.x as f64 + (self.image.width as f64 * self.canvas_scale / 2.0)) / self.canvas_scale - 0.5).round() as i32;
        let layer_y = ((y - self.canvas.y as f64 + (self.image.height as f64 * self.canvas_scale / 2.0)) / self.canvas_scale - 0.5).round() as i32;
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

    fn active_layer(&mut self) -> &mut Layer {
        &mut self.image.layers[self.active_layer_idx]
    }
}

fn main() {

    let event_loop = EventLoop::new();
    let mut gl = graphics::init(&event_loop, "Pixel Editor");
    let mut input = InputState::new();

    let mut save_button = Button::new(Rect::new(5, 5, 100, 30), "Save".into());

    let mut new_button = Button::new(Rect::new(110, 5, 100, 30), "New".into());
    let mut open_button = Button::new(Rect::new(215, 5, 100, 30), "Open".into());
    let mut color_selector = ColorSelector::new(
        Rect::new(5, 50, 50, 1000),
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
        Rect::new(60, 50, 120, 300),
        vec![
            "Pencil".into(),
            "Paintbrush".into(),
            "Color Picker".into(),
            "Paint Bucket".into(),
            "Spray Can".into(),
        ],
    );
    let mut new_dialog = NewDialog::new(500, 500, 800, 600);
    let mut open_dialog = OpenDialog::new(500, 500, "C:\\dev\\test_image.png".to_string());
    let mut save_dialog = SaveDialog::new(500, 500, "C:\\dev\\test_image.png".to_string());

    let mut confirm_overwrite_dialog = ConfirmationDialog::new(
        &gl,
        400,
        400,
        format!("Are you sure you want to overwrite {}?", "image.png"),
        vec![
            "Yes".into(),
            "No".into(),
            "Cancel".into(),
        ],
    );
    confirm_overwrite_dialog.showing = false;

    let mut state = State::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        input.update(&event);

        let mut click_intercepted = false;

        if new_dialog.should_close {
            new_dialog.should_close = false;
            state.showing_new_dialog = false;
        }
        if open_dialog.should_close {
            open_dialog.should_close = false;
            state.showing_open_dialog = false;
        }
        if save_dialog.should_close {
            save_dialog.should_close = false;
            state.showing_save_dialog = false;
        }
        if new_button.update(&input, &mut click_intercepted) {
            state.showing_new_dialog = true;
        }
        if open_button.update(&input, &mut click_intercepted) {
            state.showing_open_dialog = true;
        }
        if save_button.update(&input, &mut click_intercepted) {
            state.showing_save_dialog = true;
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

        if state.showing_new_dialog {
            if let Some(layer) = new_dialog.update(&input, &mut click_intercepted) {
                // TODO safeguards!
                let image = Image::new(layer.rect.width, layer.rect.height);
                state.image = image;
                state.active_layer_idx = 0;
                state.showing_new_dialog = false;
            }
        }

        if state.showing_open_dialog {
            if let Some(path) = open_dialog.update(&input, &mut click_intercepted) {
                if let Ok(image) = Image::from_path(&path) {
                    state.image = image;
                } else {
                    state.error_text = "Failed to load file.".into();
                }
                state.showing_open_dialog = false;
            }
        }

        if state.showing_save_dialog {
            if let Some(path) = save_dialog.update(&input, &mut click_intercepted) {
                let path = Path::new(&path);
                let mut write_file = false;
                if path.exists() {
                    confirm_overwrite_dialog.showing = true;
                    if let Some(text) = confirm_overwrite_dialog.update(&input, &mut click_intercepted) {
                        match &text[..] {
                            "Yes" => {
                                confirm_overwrite_dialog.showing = false;
                                write_file = true;
                            }
                            _ => {
                                confirm_overwrite_dialog.showing = false;
                            }
                        }
                    }
                } else {
                    write_file = true;
                }
                if write_file {
                    // TODO do I have to create a new path here?
                    match state.image.save(Path::new(&path)) {
                        Ok(_) => println!("Saved to {}", path.display()),
                        Err(_) => state.error_text = "Failed to save file!".into(),
                    }
                }
                state.showing_save_dialog = false;
            }
        }

        if (input.mouse_left_pressed && !click_intercepted) || state.currently_drawing {
            state.currently_drawing = true;
            let color = state.selected_color;

            let (x, y) = state.screen_to_canvas(input.mouse_x, input.mouse_y);
            let old_x = x - (input.mouse_delta_x / state.canvas_scale) as i32;
            let old_y = y - (input.mouse_delta_y / state.canvas_scale) as i32;

            match state.selected_tool.as_str() {
                "Pencil" => state.active_layer().draw_line(old_x, old_y, x, y, color),
                "Paintbrush" => {
                    for dx in -10..=10 {
                        for dy in -10..=10 {
                            if (dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt() < 10.0 {
                                state.active_layer().draw_line(old_x + dx, old_y + dy, x + dx, y + dy, color);
                            }
                        }
                    }
                }
                "Color Picker" => {
                    if let Some(color) = state.active_layer().get_pixel(x, y) {
                        state.selected_color = color;
                        color_selector.set_selected_color(color);
                    }
                }
                "Paint Bucket" => {
                    state.active_layer().fill(x, y, color);
                }
                "Spray Can" => {
                    for _ in 0..10 {
                        let dx = rand::random::<i32>() % 100 - 50;
                        let dy = rand::random::<i32>() % 100 - 50;
                        if (dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt() < 50.0 {
                            state.active_layer().draw_pixel(x + dx, y + dy, color);
                        }
                    }
                }
                _ => {}
            }
        }

        if !input.mouse_left_down {
            state.currently_drawing = false;
        }

        confirm_overwrite_dialog.update(&input, &mut click_intercepted);

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
                gl.clear(Color::new(50, 50, 50, 255));
                let rect = Rect::new(0, 0, state.image.width, state.image.height);
                let src_rect = rect;
                let dest_rect = Rect::new(
                    state.canvas.x - (rect.width as f64 * state.canvas_scale / 2.0).round() as i32,
                    state.canvas.y - (rect.height as f64 * state.canvas_scale / 2.0).round() as i32,
                    (rect.width as f64 * state.canvas_scale) as u32,
                    (rect.height as f64 * state.canvas_scale) as u32,
                );
                gl.draw_texture(src_rect, dest_rect, state.image.raw_data());
                save_button.draw(&gl);
                new_button.draw(&gl);
                open_button.draw(&gl);
                color_selector.draw(&gl);
                tool_selector.draw(&gl);
                gl.draw_text(&state.error_text, 5, gl.window_height - 30, 20.0, Color::new(255, 0, 0, 255));
                if state.showing_new_dialog {
                    new_dialog.draw(&gl);
                }
                if state.showing_open_dialog {
                    open_dialog.draw(&gl);
                }
                if state.showing_save_dialog {
                    save_dialog.draw(&gl);
                }
                confirm_overwrite_dialog.draw(&gl);
                gl.swap();
            },
            _ => (),
        }
    });
}
