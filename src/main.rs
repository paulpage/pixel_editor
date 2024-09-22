mod app;
use app::{self as g, Key, Color, Rect, Vec2};

mod layer;
use layer::{Image, Layer, ImageRect};

mod ui;
use ui::{Ui, Layout};

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
    selected_color: Color,
    selected_tool: String,
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
            canvas: Rect::new(100.0, 100.0, 800.0, 600.0),
            canvas_scale: 2.0,
            canvas_offset: Vec2::new(0.0, 0.0),
            canvas_offset_baseline: Vec2::new(0.0, 0.0),
            selected_color: g::BLACK,
            selected_tool: "Pencil".into(),
            currently_drawing: false,
            showing_new_dialog: false,
            showing_open_dialog: false,
            showing_save_dialog: false,
            error_text: "".into(),
            screen_width: 0.0,
            screen_height: 0.0,
            mouse_old: Vec2::new(0.0, 0.0),
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

#[macroquad::main("Pixel Editor")]
async fn main() {
//     g::simulate_mouse_with_touch(false);


    let mut ui = Ui::new();

    let mut has_updated = false;
    let mut state = State::new();
    state.image.layers.push(Layer::new(ImageRect::new(0, 0, 800, 600)));
    state.active_layer_idx = 1;
    state.active_layer().fill(0, 0, Color::new(1.0, 0.0, 0.0, 1.0));
    state.image.layers.push(Layer::new(ImageRect::new(0, 0, 800, 600)));
    state.active_layer_idx = 2;
    state.active_layer().fill(0, 0, Color::new(0.0, 1.0, 0.0, 1.0));
    state.image.layers.push(Layer::new(ImageRect::new(0, 0, 800, 600)));
    state.active_layer_idx = 3;
    state.active_layer().fill(0, 0, Color::new(0.0, 0.0, 1.0, 1.0));

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
        if ui.button("Open2").clicked {
            println!("Open2");
        }
        if ui.button("Save2").clicked {
            println!("Save2");
        }
        ui.spacer("toolbar_spacer");
        if ui.button("Close2").clicked {
            println!("Close2");
        }
        ui.pop_layout();

        ui.pop_layout();

        ui.pop_layout();
        ui.pop_layout();

        ui.push_window("Tool Pane", Rect::new(50.0, 50.0, 100.0, 300.0));
        // // ui.push_layout("Tool Pane", Layout::Floating);
        ui.push_layout("Tool columns", Layout::ToolColumn);
        ui.button("Pencil");
        ui.button("Paintbrush");
        ui.button("Color Picker");
        ui.button("Paint Bucket");
        ui.button("Spray Can");

        //////////////

        g::clear_background(Color::new(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 255.0 / 255.0));
        let rect = Rect::new(0.0, 0.0, state.image.rect.w as f32, state.image.rect.h as f32);
        let src_rect = rect;
        let dest_rect = Rect::new(
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
            dest_size: Some(Vec2::new(dest_rect.w, dest_rect.h)),
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
        if (g::is_mouse_left_down() && !click_intercepted) || state.currently_drawing {
            state.currently_drawing = true;
            let color = state.selected_color;

            let (mouse_x, mouse_y) = g::mouse_position();
            let (x, y) = state.screen_to_canvas(Vec2::new(mouse_x, mouse_y));
            let (old_x, old_y) = state.screen_to_canvas(state.mouse_old);

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
                        //color_selector.set_selected_color(color);
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
        //let mut color_selector = ColorSelector::new(
        //    Rect::new(5.0, 50.0, 50.0, 1000.0),
        //    vec![
        //        Color::new(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 1.0),
        //        Color::new(70.0 / 255.0, 70.0 / 255.0, 70.0 / 255.0, 1.0),
        //        Color::new(120.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0, 1.0),
        //        Color::new(153.0 / 255.0, 0.0 / 255.0, 48.0 / 255.0, 1.0),
        //        Color::new(237.0 / 255.0, 28.0 / 255.0, 36.0 / 255.0, 1.0),
        //        Color::new(255.0 / 255.0, 126.0 / 255.0, 0.0 / 255.0, 1.0),
        //        Color::new(255.0 / 255.0, 194.0 / 255.0, 14.0 / 255.0, 1.0),
        //        Color::new(255.0 / 255.0, 242.0 / 255.0, 0.0 / 255.0, 1.0),
        //        Color::new(168.0 / 255.0, 230.0 / 255.0, 29.0 / 255.0, 1.0),
        //        Color::new(34.0 / 255.0, 177.0 / 255.0, 76.0 / 255.0, 1.0),
        //        Color::new(0.0 / 255.0, 183.0 / 255.0, 239.0 / 255.0, 1.0),
        //        Color::new(77.0 / 255.0, 109.0 / 255.0, 243.0 / 255.0, 1.0),
        //        Color::new(47.0 / 255.0, 54.0 / 255.0, 153.0 / 255.0, 1.0),
        //        Color::new(111.0 / 255.0, 49.0 / 255.0, 152.0 / 255.0, 1.0),
        //        Color::new(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 1.0),
        //        Color::new(220.0 / 255.0, 220.0 / 255.0, 220.0 / 255.0, 1.0),
        //        Color::new(180.0 / 255.0, 180.0 / 255.0, 180.0 / 255.0, 1.0),
        //        Color::new(156.0 / 255.0, 90.0 / 255.0, 60.0 / 255.0, 1.0),
        //        Color::new(255.0 / 255.0, 163.0 / 255.0, 177.0 / 255.0, 1.0),
        //        Color::new(229.0 / 255.0, 170.0 / 255.0, 122.0 / 255.0, 1.0),
        //        Color::new(145.0 / 255.0, 228.0 / 255.0, 156.0 / 255.0, 1.0),
        //        Color::new(255.0 / 255.0, 249.0 / 255.0, 189.0 / 255.0, 1.0),
        //        Color::new(211.0 / 255.0, 249.0 / 255.0, 188.0 / 255.0, 1.0),
        //        Color::new(157.0 / 255.0, 187.0 / 255.0, 97.0 / 255.0, 1.0),
        //        Color::new(153.0 / 255.0, 217.0 / 255.0, 234.0 / 255.0, 1.0),
        //        Color::new(112.0 / 255.0, 154.0 / 255.0, 209.0 / 255.0, 1.0),
        //        Color::new(84.0 / 255.0, 109.0 / 255.0, 142.0 / 255.0, 1.0),
        //        Color::new(181.0 / 255.0, 165.0 / 255.0, 213.0 / 255.0, 1.0),
        //    ],
        //);
        //
        //    let mut tool_selector = ToolSelector::new(
        //        Rect::new(60.0, 50.0, 120.0, 300.0),
        //        vec![
        //        "Pencil".into(),
        //        "Paintbrush".into(),
        //        "Color Picker".into(),
        //        "Paint Bucket".into(),
        //        "Spray Can".into(),
        //        ],
        //    );
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
        //
        //            state.selected_color = color_selector.update(&mut click_intercepted);
        //            state.selected_tool = tool_selector.update(&mut click_intercepted);
        //
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
        //g::draw_text(&state.error_text, 5.0, g::screen_height() - 30.0, 20.0, Color::new(1.0, 0.0, 0.0, 1.0));

        let (x, y) = g::mouse_position();
        state.mouse_old.x = x;
        state.mouse_old.y = y;

        ui.update();
        g::next_frame().await
    }
}
