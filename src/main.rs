mod app;
use app::{self as g, Key, Color, Rect, Vec2};

mod layer;
use layer::{Image, Layer, ImageRect};

mod ui;
use ui::{Ui, Layout, StyleInfo};

// use std::path::Path;
// use nfd::Response as FileDialogResponse;

// mod util;
// use util::{Rect, Color};

// mod gui;
// use gui::{Widget, Button, ColorSelector, ToolSelector, NewDialog, OpenDialog, SaveDialog, ConfirmationDialog, Dialog};

struct State {
    image: Image,
    active_layer_idx: usize,
    canvas: Rect,
    canvas_scale: f32,
    canvas_offset: Vec2,
    canvas_offset_baseline: Vec2,
    active_color: Color,
    active_tool: String,
    currently_drawing: bool,
    showing_new_dialog: bool,
    showing_open_dialog: bool,
    showing_save_dialog: bool,
    error_text: String,
    screen_width: f32,
    screen_height: f32,
    mouse_old: Vec2,
}

impl State {
    fn new() -> Self {
        Self {
            image: Image::new(800, 600),
            active_layer_idx: 0,
            canvas: rect!(100, 100, 800, 600),
            canvas_scale: 2.0,
            canvas_offset: vec2!(0, 0),
            canvas_offset_baseline: vec2!(0, 0),
            active_color: g::BLACK,
            active_tool: "Paintbrush".into(),
            currently_drawing: false,
            showing_new_dialog: false,
            showing_open_dialog: false,
            showing_save_dialog: false,
            error_text: "".into(),
            screen_width: 0.0,
            screen_height: 0.0,
            mouse_old: vec2!(0, 0),
        }
    }

     fn screen_to_canvas(&mut self, p: Vec2) -> (i32, i32) {
         (
             ((p.x - self.canvas.x + (self.image.rect.w as f32 * self.canvas_scale / 2.0)) / self.canvas_scale - 0.5).round() as i32 - self.active_layer().rect.x,
             ((p.y - self.canvas.y + (self.image.rect.h as f32 * self.canvas_scale / 2.0)) / self.canvas_scale - 0.5).round() as i32 - self.active_layer().rect.y,
         )
     }

     fn center_canvas(&mut self) {
         self.canvas.x = self.screen_width / 2.0;
         self.canvas.y = self.screen_height / 2.0;
     }

     fn update_canvas_position(&mut self) {
         self.canvas.x += self.canvas_offset.x;
         self.canvas.y += self.canvas_offset.y;
     }

     fn active_layer(&mut self) -> &mut Layer {
         &mut self.image.layers[self.active_layer_idx]
     }
}

fn draw_tool_pane(ui: &mut Ui, state: &mut State) {
    ui.push_window("Tool Pane", rect!(50, 50, 100, 300));
    // // ui.push_layout("Tool Pane", Layout::Floating);
    ui.push_layout("Tool columns", Layout::ToolColumn);

    let tools = [
        "Pencil",
        "Paintbrush",
        "Color Picker",
        "Paint Bucket",
        "Spray Can",
    ];
    for tool in &tools {
        if state.active_tool == *tool {
            temp_style!(ui, background_color: color!(255, 255, 0));
        }
        if ui.button(tool).clicked {
            state.active_tool = String::from(*tool);
        }
    }
}

fn draw_color_selector(ui: &mut Ui, state: &mut State) {
    ui.push_window("Color Selector", rect!(200, 50, 100, 300));
    ui.push_layout("Color columns", Layout::ToolColumn);

    let colors = [
        color!(0, 0, 0),
        color!(70, 70, 70),
        color!(120, 120, 120),
        color!(153, 0, 48),
        color!(237, 28, 36),
        color!(255, 126, 0),
        color!(255, 194, 14),
        color!(255, 242, 0),
        color!(168, 230, 29),
        color!(34, 177, 76),
        color!(0, 183, 239),
        color!(77, 109, 243),
        color!(47, 54, 153),
        color!(111, 49, 152),
        color!(255, 255, 255),
        color!(220, 220, 220),
        color!(180, 180, 180),
        color!(156, 90, 60),
        color!(255, 163, 177),
        color!(229, 170, 122),
        color!(145, 228, 156),
        color!(255, 249, 189),
        color!(211, 249, 188),
        color!(157, 187, 97),
        color!(153, 217, 234),
        color!(112, 154, 209),
        color!(84, 109, 142),
        color!(181, 165, 213),
    ];

    for color in &colors {
        temp_style!(ui, background_color: *color);
        if state.active_color == *color {
            temp_style!(ui, border_color: color!(255, 255, 0));
        }
        let hash = format!("##{:?}", color);
        if ui.button(&hash).clicked {
            state.active_color = *color;
        }
    }
}

#[macroquad::main("Pixel Editor")]
async fn main() {
//     g::simulate_mouse_with_touch(false);


    let mut ui = Ui::new();

    let mut has_updated = false;
    let mut state = State::new();
    state.image.layers.push(Layer::new(ImageRect::new(0, 0, 800, 600)));
    state.active_layer_idx = 1;
    state.active_layer().fill(0, 0, color!(255, 0, 0));
    state.image.layers.push(Layer::new(ImageRect::new(0, 0, 800, 600)));
    state.active_layer_idx = 2;
    state.active_layer().fill(0, 0, color!(0, 255, 0));
    state.image.layers.push(Layer::new(ImageRect::new(0, 0, 800, 600)));
    state.active_layer_idx = 3;
    state.active_layer().fill(0, 0, color!(0, 0, 255));

    let mut texture = g::Texture2D::from_rgba8(state.image.rect.w as u16, state.image.rect.h as u16, &state.image.raw_data());
    let mut click_intercepted = false;

    loop {
        if g::is_key_pressed(Key::Q) || g::is_key_pressed(Key::Enter) {
            break;
        }

        ui.push_layout("Main Window", Layout::Vertical);

        ui.push_layout("Toolbar", Layout::ToolRow);
        if ui.button("Open").clicked {
            println!("Open");
        }
        if ui.button("Save").clicked {
            println!("Save");
        }
        ui.spacer("toolbar_spacer");
        if ui.button("Close").clicked {
            println!("Close");
        }
        ui.pop_layout();


        ui.spacer("viewport_spacer");
        // ui.push_layout("Viewport", Layout::Vertical);
        // ui.pop_layout();

        ui.push_layout("Status bar", Layout::ToolRow);

        push_style!(ui,
            background_color: color!(0, 128, 0),
        );
        ui.label("File:");
        if ui.text_box("File Input 1").clicked {
            println!("File input 1");
        }
        if ui.text_box("File Input 2").clicked {
            println!("File input 2");
        }
        ui.pop_style();

        ui.spacer("toolbar_spacer");
        if ui.button("Close2").clicked {
            println!("Close2");
        }
        ui.pop_layout();

        ui.pop_layout();

        ui.pop_layout();
        ui.pop_layout();

        draw_tool_pane(&mut ui, &mut state);
        draw_color_selector(&mut ui, &mut state);

        //////////////

        g::clear_background(color!(50, 50, 50));
        let rect = rect!(0, 0, state.image.rect.w, state.image.rect.h);
        let src_rect = rect;
        let dest_rect = rect!(
            state.canvas.x - (rect.w * state.canvas_scale as f32 / 2.0).round(),
            state.canvas.y - (rect.h * state.canvas_scale as f32 / 2.0).round(),
            rect.w * state.canvas_scale as f32,
            rect.h * state.canvas_scale as f32,
        );
        // TODO Texture2D only supports u16, determine if we need to find an
        // alternative or go with it and do bounds checking
        let dirty_rect = state.image.dirty_rect();
        // let dirty_rect = state.image.rect;
        let dirty_data = state.image.partial_data(dirty_rect);
        let dirty_image = g::Image {
            bytes: dirty_data,
            width: dirty_rect.w as u16,
            height: dirty_rect.h as u16,
        };
        if dirty_rect.w != 0 && dirty_rect.h != 0 {
            texture.update_part(&dirty_image, dirty_rect.x, dirty_rect.y, dirty_rect.w as i32, dirty_rect.h as i32);
        }
        state.image.clear_dirty();

        texture.set_filter(g::FilterMode::Nearest);
        g::draw_texture_ex(&texture, dest_rect.x, dest_rect.y, g::WHITE, g::DrawTextureParams {
            dest_size: Some(vec2!(dest_rect.w, dest_rect.h)),
            source: Some(src_rect),
            ..Default::default()
        });

        //////////////

        g::draw_text("Hello", 100.0, 100.0, 30.0, g::DARKGRAY);

        if !has_updated || g::is_key_pressed(Key::Space) {
            has_updated = true;
        }

        if g::is_key_pressed(Key::Q) {
            break;
        }
        if g::is_key_pressed(Key::Left) {
            state.active_layer().rect.x -= 100;
        }
        if g::is_key_pressed(Key::Right) {
            state.active_layer().rect.x += 100;
        }
        if g::is_key_pressed(Key::Up) {
            state.active_layer().rect.y -= 100;
        }
        if g::is_key_pressed(Key::Down) {
            state.active_layer().rect.y += 100;
        }
        if g::is_key_pressed(Key::Tab) {
            state.active_layer_idx += 1;
            state.active_layer_idx %= state.image.layers.len();
            println!("Active Layer Index: {}", state.active_layer_idx);
        }
        if g::is_mouse_middle_pressed() {
            let (mouse_x, mouse_y) = g::mouse_position();
            state.canvas_offset_baseline.x = mouse_x;
            state.canvas_offset_baseline.y = mouse_y;
        }
        if g::is_mouse_middle_down() {
            let (mouse_x, mouse_y) = g::mouse_position();
            state.canvas_offset.x = mouse_x - state.canvas_offset_baseline.x;
            state.canvas_offset.y = mouse_y - state.canvas_offset_baseline.y;
            state.update_canvas_position();
            state.canvas_offset.x = 0.0;
            state.canvas_offset.y = 0.0;
            state.canvas_offset_baseline.x = mouse_x;
            state.canvas_offset_baseline.y = mouse_y;
        }

        if !g::is_mouse_left_down() {
            state.currently_drawing = false;
        }

        let new_screen_width = g::screen_width();
        let new_screen_height = g::screen_height();
        if state.screen_width != new_screen_width || state.screen_height != new_screen_height {
            state.screen_width = new_screen_width;
            state.screen_height = new_screen_height;
            state.center_canvas();
        }

        let (_wheel_x, wheel_y) = g::mouse_wheel();
        if wheel_y != 0.0 {
            state.canvas_scale *= (10.0 + wheel_y) / 10.0;
            state.update_canvas_position();
        }
        if (g::is_mouse_left_down() && !ui.mouse_intercepted) || state.currently_drawing {
            state.currently_drawing = true;
            let color = state.active_color;

            let (mouse_x, mouse_y) = g::mouse_position();
            let (x, y) = state.screen_to_canvas(vec2!(mouse_x, mouse_y));
            let (old_x, old_y) = state.screen_to_canvas(state.mouse_old);

            match state.active_tool.as_str() {
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
                        state.active_color = color;
                    }
                }
                "Paint Bucket" => {
                    state.active_layer().fill(x, y, color);
                }
                "Spray Can" => {
                    for _ in 0..100 {
                        let dx = macroquad::rand::rand() as i32 % 100 - 50;
                        let dy = macroquad::rand::rand() as i32 % 100 - 50;
                        if (dx as f32 * dx as f32 + dy as f32 * dy as f32).sqrt() < 50.0 {
                            state.active_layer().draw_pixel(x + dx, y + dy, color);
                        }
                        state.active_layer().add_dirty_rect(ImageRect::new(x - 51, y - 51, 102, 102));
                    }
                }
                _ => {}
            }
        }

        // let mut test_dialog = Dialog::new("Test Dialog");
        //let mut layer_dialog = Dialog::new("Layers");
        //
        //    let mut new_dialog = NewDialog::new(500.0, 500.0, 800.0, 600.0);
        //    let mut open_dialog = OpenDialog::new(500.0, 500.0, "test_image.png".to_string());
        //    let mut save_dialog = SaveDialog::new(500.0, 500.0, "test_image.png".to_string());
        //
        //    let mut confirm_overwrite_dialog = ConfirmationDialog::new(
        //        400.0,
        //        400.0,
        //        format!("Are you sure you want to overwrite {}?", "image.png"),
        //        vec![
        //        "Yes".into(),
        //        "No".into(),
        //        "Cancel".into(),
        //        ],
        //    );
        //    confirm_overwrite_dialog.showing = false;
        //    let mut confirm_overwrite_path: Option<String> = None;
        //
        //    loop {
        //        let mut click_intercepted = false;
        //        // test_dialog.update(&mut click_intercepted);
        //        // layer_dialog.update(&mut click_intercepted);
        //        if !click_intercepted {
        //
        //            if new_dialog.should_close {
        //                new_dialog.should_close = false;
        //                state.showing_new_dialog = false;
        //            }
        //            if open_dialog.should_close {
        //                open_dialog.should_close = false;
        //                state.showing_open_dialog = false;
        //            }
        //            if save_dialog.should_close {
        //                save_dialog.should_close = false;
        //                state.showing_save_dialog = false;
        //            }
        //            if new_button.update(&mut click_intercepted) {
        //                state.showing_new_dialog = true;
        //            }
        //            if open_button.update(&mut click_intercepted) {
        //                state.showing_open_dialog = true;
        //            }
        //            if save_button.update(&mut click_intercepted) {
        //                state.showing_save_dialog = true;
        //            }
        //        }
        //
        //
        //        if state.showing_new_dialog {
        //            if let Some(layer) = new_dialog.update(&mut click_intercepted) {
        //                // TODO safeguards!
        //                let image = Image::new(layer.rect.w, layer.rect.h);
        //                state.image = image;
        //                texture = g::Texture2D::from_rgba8(state.image.rect.w as u16, state.image.rect.h as u16, &state.image.raw_data());
        //                state.active_layer_idx = 0;
        //                state.showing_new_dialog = false;
        //            }
        //        }
        //
        //        if state.showing_open_dialog {
        //            if let Some(path) = open_dialog.update(&mut click_intercepted) {
        //                if let Ok(image) = Image::from_path(&path) {
        //                    texture = g::Texture2D::from_rgba8(state.image.rect.w as u16, state.image.rect.h as u16, &state.image.raw_data());
        //                    state.image = image;
        //                } else {
        //                    state.error_text = "Failed to load file.".into();
        //                }
        //                state.showing_open_dialog = false;
        //            }
        //        }
        //
        //         if state.showing_save_dialog {
        //
        //             let mut write_file = false;
        //
        //             let mut path = None;
        //
        //             if let Some(path) = save_dialog.update(&mut click_intercepted) {
        //                 let path = Path::new(&path);
        //                 if path.unwrap().exists() {
        //                     confirm_overwrite_dialog.showing = true;
        //                     confirm_overwrite_path = Some(path.to_string());
        //                     // if let Some(text) = confirm_overwrite_dialog.update(&mut click_intercepted) {
        //                     //     println!("text received form confrm: {}", text);
        //                     //     match &text[..] {
        //                     //         "Yes" => {
        //                     //             println!("we got a yes");
        //                     //             confirm_overwrite_dialog.showing = false;
        //                     //             write_file = true;
        //                     //         }
        //                     //         _ => {
        //                     //             confirm_overwrite_dialog.showing = false;
        //                     //         }
        //                     //     }
        //                     // }
        //                 } else {
        //                     write_file = true;
        //                 }
        //
        //                 if let Some(text) = confirm_overwrite_dialog.update(&mut click_intercepted) {
        //                     println!("text received form confrm: {}", text);
        //                     match &text[..] {
        //                         "Yes" => {
        //                             println!("we got a yes");
        //                             confirm_overwrite_dialog.showing = false;
        //                             write_file = true;
        //                             path = confirm_overwrite_path;
        //                         }
        //                         _ => {
        //                             confirm_overwrite_dialog.showing = false;
        //                         }
        //                     }
        //                 }
        //
        //                 if write_file {
        //                     // TODO do I have to create a new path here?
        //                     match state.image.save(Path::new(&path.unwrap())) {
        //                         Ok(_) => println!("Saved to {}", path.unwrap().display()),
        //                         Err(_) => state.error_text = "Failed to save file!".into(),
        //                     }
        //                 }
        //                 state.showing_save_dialog = false;
        //             }
        //         }
        //
        //
        //        if let Some(text) = confirm_overwrite_dialog.update(&mut click_intercepted) {
        //            println!("text received form confrm: {}", text);
        //            match &text[..] {
        //                "Yes" => {
        //                    println!("we got a yes");
        //                    // confirm_overwrite_dialog.showing = false;
        //                    // match state.image.save(Path::new(&path)) {
        //                    //     Ok(_) => println!("Saved to {}", path.display()),
        //                    //     Err(_) => state.error_text = "Failed to save file!".into(),
        //                    // }
        //                }
        //                _ => {
        //                    confirm_overwrite_dialog.showing = false;
        //                }
        //            };
        //            state.showing_save_dialog = false;
        //        }
        //
        //
        //g::draw_text(&state.error_text, 5.0, g::screen_height() - 30.0, 20.0,
        //color!(255, 0, 0));

        let (x, y) = g::mouse_position();
        state.mouse_old.x = x;
        state.mouse_old.y = y;

        ui.update();
        g::next_frame().await
    }
}
