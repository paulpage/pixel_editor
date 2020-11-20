use sdl2::keyboard::Keycode as Key;
use std::path::Path;
use std::cmp::{min, max};
use nfd::Response as FileDialogResponse;

use std::time::Duration;

mod layer;
use layer::{Image, ImageHistory, Layer};

mod util;
use util::{Rect, Color};

mod gui;
use gui::{Widget, Button, ColorSelector, ToolSelector, NewDialog, ConfirmationDialog, LayerSelector};

mod platform;
use platform::{Platform, PlatformMessage};

macro_rules! active_layer {
    ($state:expr) => {
        $state.image.layers[$state.active_layer_idx]
    }
}

enum FillMode {
    Fill,
    Outline,
}

struct State {
    image: Image,
    active_layer_idx: usize,
    temp_layer: Layer,
    history: ImageHistory,
    canvas: Rect,
    canvas_scale: f64,
    canvas_offset_x: i32,
    canvas_offset_y: i32,
    canvas_offset_baseline_x: i32,
    canvas_offset_baseline_y: i32,
    selected_color: Color,
    selected_tool: String,
    brush_size: i32,
    currently_drawing: bool,
    error_text: String,
    cached_blended_layer: Layer,
    dirty_region: Rect,
    last_mousedown_x: i32,
    last_mousedown_y: i32,
    fill_mode: FillMode,
}

impl State {
    fn new() -> Self {
        let (width, height) = (800, 600);
        Self {
            image: Image::new(width, height),
            active_layer_idx: 0,
            temp_layer: Layer::new(Rect::new(0, 0, width, height)),
            history: ImageHistory::new(),
            canvas: Rect::new(100, 100, width, height),
            canvas_scale: 1.0,
            canvas_offset_x: 0,
            canvas_offset_y: 0,
            canvas_offset_baseline_x: 0,
            canvas_offset_baseline_y: 0,
            selected_color: Color::BLACK,
            selected_tool: "Pencil".into(),
            brush_size: 20,
            currently_drawing: false,
            error_text: "".into(),
            cached_blended_layer: Layer::new(Rect::new(0, 0, width, height)),
            dirty_region: Rect::new(0, 0, width, height),
            last_mousedown_x: 0,
            last_mousedown_y: 0,
            fill_mode: FillMode::Outline,
        }
    }
    
    fn screen_to_canvas(&self, x: i32, y: i32) -> (i32, i32) {
        let layer_x = ((x as f64 - self.canvas.x as f64 + (self.image.width as f64 * self.canvas_scale / 2.0)) / self.canvas_scale - 0.5).round() as i32;
        let layer_y = ((y as f64 - self.canvas.y as f64 + (self.image.height as f64 * self.canvas_scale / 2.0)) / self.canvas_scale - 0.5).round() as i32;
        (layer_x, layer_y)
    }

    fn center_canvas(&mut self, screen_width: i32, screen_height: i32) {
        self.canvas.x = screen_width / 2;
        self.canvas.y = screen_height / 2;
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
    let mut p = Platform::new().unwrap();
    let mut state = State::new();

    let mut save_button = Button::new(Rect::new(5, 5, 100, 30), "Save".into());
    let mut new_button = Button::new(Rect::new(110, 5, 100, 30), "New".into());
    let mut open_button = Button::new(Rect::new(215, 5, 100, 30), "Open".into());
    let mut undo_button = Button::new(Rect::new(320, 5, 100, 30), "Undo".into());
    let mut redo_button = Button::new(Rect::new(425, 5, 100, 30), "Redo".into());
    let mut new_layer_button = Button::new(Rect::new(530, 5, 100, 30), "Add Layer".into());

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
            "Eraser".into(),
            "Line".into(),
            "Rectangle".into(),
        ],
    );
    let mut layer_selector = LayerSelector::new(Rect::new(400, 400, 200, 200));

    let mut new_dialog = NewDialog::new(500, 500, 800, 600);

    let mut confirm_overwrite_dialog = ConfirmationDialog::new(
        &mut p,
        400,
        400,
        format!("Are you sure you want to overwrite {}?", "image.png"),
        vec![
            "Yes".into(),
            "No".into(),
            "Cancel".into(),
        ],
    );

    // TODO do we want to make the background a pattern (like white or checkers) instead of or in
    // addition to making the first layer solid white?
    state.image.layers[0].fill(0, 0, Color::WHITE);
    state.history.take_snapshot(&state.image);
    layer_selector.refresh(&state.image, state.active_layer_idx);

    'running: loop {

        if let PlatformMessage::Quit = p.process_events() {
            // TODO prompt to save image
            break 'running;
        }

        if p.screen_size_changed {
            state.center_canvas(p.screen_width, p.screen_height);
        }

        let mut click_intercepted = false;
        if save_button.update(&mut p, &mut click_intercepted) {
            let result = nfd::open_save_dialog(None, None).unwrap();
            match result {
                FileDialogResponse::Okay(file_path) => {
                    let path = Path::new(&file_path);
                    let mut write_file = false;
                    if path.exists() {
                        confirm_overwrite_dialog.show();
                        if let Some(text) = confirm_overwrite_dialog.update(&mut p, &mut click_intercepted) {
                            if let "Yes" = &text[..] {
                                write_file =true;
                            }
                        }
                    } else {
                        write_file = true;
                    }
                    if write_file {
                        match state.image.save(Path::new(&file_path)) {
                            Ok(_) => println!("Saved to {}", file_path),
                            Err(_) => state.error_text = "Failed to save file!".into(),
                        }
                    }
                }
                FileDialogResponse::OkayMultiple(_) => {
                    state.error_text = "Can't save to multiple files.".into();
                }
                FileDialogResponse::Cancel => {}
            }
        }

        if new_button.update(&mut p, &mut click_intercepted) {
            new_dialog.show();
        }
        if open_button.update(&mut p, &mut click_intercepted) {
            let result = nfd::open_file_dialog(None, None).unwrap();
            match result {
                FileDialogResponse::Okay(file_path) => {
                    if let Ok(image) = Image::from_path(&file_path) {
                        state.image = image;
                    } else {
                        state.error_text = "Failed to load file.".into();
                    }
                }
                FileDialogResponse::OkayMultiple(_) => {
                    state.error_text = "Can't open multiple files.".into();
                }
                FileDialogResponse::Cancel => {}
            }
        }
        if undo_button.update(&mut p, &mut click_intercepted) {
            state.image = state.history.undo(state.image);
        }
        if redo_button.update(&mut p, &mut click_intercepted) {
            state.image = state.history.redo(state.image);
        }
        if new_layer_button.update(&mut p, &mut click_intercepted) {
            state.image.layers.push(Layer::new(Rect::new(0, 0, state.image.width, state.image.height)));
            state.active_layer_idx = state.image.layers.len() - 1;
            layer_selector.refresh(&state.image, state.active_layer_idx);
        }

        state.selected_color = color_selector.update(&mut p, &mut click_intercepted);
        state.selected_tool = tool_selector.update(&mut p, &mut click_intercepted);
        if let Some(idx) = layer_selector.update(&mut p, &mut click_intercepted) {
            state.active_layer_idx = idx;
            layer_selector.refresh(&state.image, state.active_layer_idx);
        }

        if p.key_down(Key::Q) {
            // TODO ask the user if they really want to quit, and/or save a recovery file
            break 'running;
        }

        if p.mouse_middle_pressed {
            state.canvas_offset_baseline_x = p.mouse_x as i32;
            state.canvas_offset_baseline_y = p.mouse_y as i32;
        }
        if p.mouse_middle_down {
            state.canvas_offset_x = p.mouse_x as i32 - state.canvas_offset_baseline_x;
            state.canvas_offset_y = p.mouse_y as i32 - state.canvas_offset_baseline_y;
            state.update_canvas_position();
            state.canvas_offset_x = 0;
            state.canvas_offset_y = 0;
            state.canvas_offset_baseline_x = p.mouse_x as i32;
            state.canvas_offset_baseline_y = p.mouse_y as i32;
        }

        if (p.mouse_left_pressed && !click_intercepted) || state.currently_drawing {
            state.currently_drawing = true;
            let color = state.selected_color;

            let (x, y) = state.screen_to_canvas(p.mouse_x, p.mouse_y);
            let old_x = x - (p.mouse_delta_x as f64 / state.canvas_scale) as i32;
            let old_y = y - (p.mouse_delta_y as f64 / state.canvas_scale) as i32;

            if p.mouse_left_pressed {
                state.last_mousedown_x = x;
                state.last_mousedown_y = y;
            }

            state.dirty_region = Rect {
                x: min(x, old_x) - state.brush_size - 1,
                y: min(y, old_y) - state.brush_size - 1,
                width: (x - old_x).abs() as u32 + state.brush_size as u32 * 2 + 2,
                height: (y - old_y).abs() as u32 + state.brush_size as u32 * 2 + 2,
            };

            match state.selected_tool.as_str() {
                "Pencil" => active_layer!(state).draw_line(old_x, old_y, x, y, color),
                "Paintbrush" => {
                    for dx in -state.brush_size / 2..=state.brush_size / 2 {
                        for dy in -state.brush_size / 2..=state.brush_size / 2 {
                            if (dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt() < 10.0 {
                                active_layer!(state).draw_line(old_x + dx, old_y + dy, x + dx, y + dy, color);
                            }
                        }
                    }
                }
                "Color Picker" => {
                    if let Some(color) = active_layer!(state).get_pixel(x, y) {
                        state.selected_color = color;
                        color_selector.set_selected_color(color);
                    }
                    state.dirty_region = Rect::new(0, 0, 0, 0);
                }
                "Paint Bucket" => {
                    active_layer!(state).fill(x, y, color);
                    state.dirty_region = Rect::new(0, 0, state.canvas.width, state.canvas.height);
                }
                "Spray Can" => {
                    for _ in 0..100 {
                        let dx = rand::random::<i32>() % state.brush_size - (state.brush_size / 2);
                        let dy = rand::random::<i32>() % state.brush_size - (state.brush_size / 2);
                        if (dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt() < (state.brush_size as f64 / 2.0) {
                            active_layer!(state).draw_pixel(x + dx, y + dy, color);
                        }
                    }
                }
                "Eraser" => {
                    for dx in -state.brush_size / 2..=state.brush_size / 2 {
                        for dy in -state.brush_size / 2..=state.brush_size / 2 {
                            if (dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt() < 10.0 {
                                active_layer!(state).draw_line(old_x + dx, old_y + dy, x + dx, y + dy, Color::new(0, 0, 0, 0));
                            }
                        }
                    }
                }
                "Line" => {
                    state.temp_layer.clear();
                    state.temp_layer.draw_line(state.last_mousedown_x, state.last_mousedown_y, x, y, color);
                    state.dirty_region = Rect::new(0, 0, state.canvas.width, state.canvas.height);
                    // state.dirty_region = Rect {
                    //     x: min(x, state.last_mousedown_x) - state.brush_size - 1,
                    //     y: min(y, state.last_mousedown_y) - state.brush_size - 1,
                    //     width: (x - state.last_mousedown_x).abs() as u32 + state.brush_size as u32 * 2 + 2,
                    //     height: (y - state.last_mousedown_y).abs() as u32 + state.brush_size as u32 * 2 + 2,
                    // };
                }
                "Rectangle" => {
                    state.temp_layer.clear();

                    let x1 = min(state.last_mousedown_x, x);
                    let x2 = max(state.last_mousedown_x, x);
                    let y1 = min(state.last_mousedown_y, y);
                    let y2 = max(state.last_mousedown_y, y);
                    match state.fill_mode {
                        FillMode::Fill => {
                            for i in y1..y2 {
                                state.temp_layer.draw_line(x1, i, x2, i, color);
                            }
                        }
                        FillMode::Outline => {
                            for i in 0..state.brush_size {
                                if x2 - i > x1 && y2 - i > y1 {
                                    state.temp_layer.draw_line(x1 + i, y1 + i, x2 - i, y1 + i, color);
                                    state.temp_layer.draw_line(x1 + i, y2 - i, x2 - i, y2 - i, color);
                                    state.temp_layer.draw_line(x1 + i, y1 + i, x1 + i, y2 - i, color);
                                    state.temp_layer.draw_line(x2 - i, y1 + i, x2 - i, y2 - i, color);
                                }
                            }
                        }
                    }
                    state.dirty_region = Rect::new(0, 0, state.canvas.width, state.canvas.height);
                }
                _ => {}
            }
        }

        if p.mouse_left_released && state.currently_drawing {
            active_layer!(state).blend(&state.temp_layer);
            state.history.take_snapshot(&state.image);
            state.temp_layer.clear();
            state.dirty_region = Rect::new(0, 0, 0, 0);
        }

        if let Some(layer) = new_dialog.update(&mut p, &mut click_intercepted) {
            // TODO safeguards!
            let image = Image::new(layer.rect.width, layer.rect.height);
            state.image = image;
            state.active_layer_idx = 0;
            state.history = ImageHistory::new();
            state.canvas = Rect::new(state.canvas.x, state.canvas.y, layer.rect.width, layer.rect.height);
            state.cached_blended_layer = Layer::new(Rect::new(0, 0, layer.rect.width, layer.rect.height));
            state.dirty_region = Rect::new(0, 0, layer.rect.width, layer.rect.height);
            state.temp_layer = Layer::new(Rect::new(0, 0, layer.rect.width, layer.rect.height));
            active_layer!(state).fill(0, 0, Color::WHITE);
        }

        if !p.mouse_left_down {
            state.currently_drawing = false;
        }

        confirm_overwrite_dialog.update(&mut p, &mut click_intercepted);

        let scroll_y = p.get_scroll_delta_y();
        state.canvas_scale *= (10.0 + scroll_y as f64) / 10.0;

        p.clear(Color::new(50, 50, 50, 255));
        let rect = Rect::new(0, 0, state.image.width, state.image.height);
        let src_rect = rect;
        let dest_rect = Rect::new(
            state.canvas.x - (rect.width as f64 * state.canvas_scale / 2.0).round() as i32,
            state.canvas.y - (rect.height as f64 * state.canvas_scale / 2.0).round() as i32,
            (rect.width as f64 * state.canvas_scale) as u32,
            (rect.height as f64 * state.canvas_scale) as u32,
        );
        if state.dirty_region.width > 0 && state.dirty_region.height > 0 {
            let dirty_blended = state.image.blend(state.dirty_region);
            state.cached_blended_layer.blend(&dirty_blended);
            state.cached_blended_layer.blend(&state.temp_layer);
        }
        p.draw_texture(&mut state.cached_blended_layer.data, src_rect, dest_rect);
        save_button.draw(&mut p);
        new_button.draw(&mut p);
        open_button.draw(&mut p);
        undo_button.draw(&mut p);
        redo_button.draw(&mut p);
        new_layer_button.draw(&mut p);
        color_selector.draw(&mut p);
        tool_selector.draw(&mut p);
        layer_selector.draw(&mut p);
        p.draw_text(&state.error_text, 5, p.screen_height - 30, 20.0, Color::new(255, 0, 0, 255));
        new_dialog.draw(&mut p);
        confirm_overwrite_dialog.draw(&mut p);

        p.present();

        state.dirty_region = Rect::new(0, 0, 0, 0);

        std::thread::sleep(Duration::from_millis(1));
    }
}
