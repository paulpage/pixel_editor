use super::layer::{Layer, ImageRect};

use super::app::{self, Rect, Color, KeyCode as Key};

pub trait Widget<T> {
    fn draw(&self);
    fn update(&mut self, click_intercepted: &mut bool) -> T;
}

pub struct ConfirmationDialog {
    message: String,
    buttons: Vec<Button>,
    rect: Rect,
    pub showing: bool,
}

pub struct Dialog {
    title: String,
    rect: Rect,
    border_size: f32,
    titlebar_size: f32,
    color: Color,
    border_color: Color,
    text_color: Color,
    is_dragging: bool,
}

impl Dialog {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            rect: Rect::new(0.0, 0.0, 200.0, 200.0),
            border_size: 5.0,
            titlebar_size: 20.0,
            color: app::GRAY,
            border_color: Color::new(0.0, 0.0, 50.0/255.0, 1.0),
            text_color: app::WHITE,
            is_dragging: false,
        }
    }

    pub fn draw(&self) {
        app::draw_rect(self.rect, self.border_color);
        app::draw_rectangle(
            self.rect.x + self.border_size,
            self.rect.y + self.border_size + self.titlebar_size,
            self.rect.w - self.border_size * 2.0,
            self.rect.h - self.border_size * 2.0 - self.titlebar_size,
            self.color,
        );
        app::draw_text(&self.title, self.rect.x + self.border_size, self.rect.y + self.border_size, 20.0, self.text_color);
    }

    pub fn update(&mut self, click_intercepted: &mut bool) {

        let titlebar_rect = Rect {
            x: self.rect.x,
            y: self.rect.y,
            w: self.rect.w,
            h: self.border_size + self.titlebar_size,
        };

        if app::is_mouse_left_pressed() && titlebar_rect.contains(app::mouse_position().into()) && !(*click_intercepted) {
            self.is_dragging = true;
        } else if !app::is_mouse_left_down() {
            self.is_dragging = false;
        }

        if self.is_dragging {
            let mouse_delta = app::mouse_delta_position();
            self.rect.x += mouse_delta.x;
            self.rect.y += mouse_delta.y;
        }

        *click_intercepted = self.rect.contains(app::mouse_position().into()) && app::is_mouse_left_pressed() || *click_intercepted;
    }
}


impl ConfirmationDialog {
    pub fn new(x: f32, y: f32, message: String, options: Vec<String>) -> Self {
        // TODO bring back measuring when I deal with font loading
        // let size = app::measure_text(&message, 20.0);
        // let width = std::cmp::max(
        //     size.text_width + 10.0,
        //     options.len() as f32 * 105.0 + 5.0
        // );
        let width = 800.0;
        let rect = Rect::new(x, y, width, 200.0);
        let mut buttons = Vec::new();
        for (i, option) in options.iter().enumerate() {
            buttons.push(Button::new(Rect::new(x + 5.0 + i as f32 * 105.0, y + 50.0, 100.0, 50.0), option.to_string()));
        }
        Self {
            rect,
            message,
            buttons,
            showing: false,
        }
    }
}

impl Widget<Option<String>> for ConfirmationDialog {
    fn draw(&self) {
        if self.showing {
            app::draw_rect(self.rect, app::GRAY);
            app::draw_text(&self.message, self.rect.x + 5.0, self.rect.y + 5.0, 20.0, app::BLACK);
            for button in &self.buttons {
                button.draw();
            }
        }
    }

    fn update(&mut self, mouse_intercepted: &mut bool) -> Option<String> {
        if self.showing {
            // *mouse_intercepted = self.rect.contains_point(input.mouse_x as i32, input.mouse_y as i32) && input.mouse_left_down || *mouse_intercepted;
            for button in &mut self.buttons {
                if button.update(mouse_intercepted) {
                    println!("yah");
                    return Some(button.text.clone());
                }
            }
        }
        None
    }
}

pub struct NewDialog {
    width_field: TextBox,
    height_field: TextBox,
    ok_button: Button,
    cancel_button: Button,
    rect: Rect,
    pub should_close: bool,
}

impl NewDialog {
    pub fn new(x: f32, y: f32, default_width: f32, default_height: f32) -> Self {
        let mut dialog = Self {
            width_field: TextBox::new(Rect::new(x + 70.0, y + 5.0, 100.0, 30.0)),
            height_field: TextBox::new(Rect::new(x + 70.0, y + 35.0, 100.0, 30.0)),
            ok_button: Button::new(Rect::new(x + 5.0, y + 70.0, 100.0, 30.0), "Ok".into()),
            cancel_button: Button::new(Rect::new(x + 110.0, y + 70.0, 100.0, 30.0), "Cancel".into()),
            rect: Rect::new(x, y, 250.0, 110.0),
            should_close: false,
        };
        dialog.width_field.text = default_width.to_string();
        dialog.height_field.text = default_height.to_string();
        dialog
    }
}

impl Widget<Option<Layer>> for NewDialog {
    fn draw(&self) {
        app::draw_rect(self.rect, app::GRAY);
        app::draw_text("Width:", self.rect.x + 5.0, self.rect.y + 10.0, 20.0, app::BLACK);
        app::draw_text("Height:", self.rect.x + 5.0, self.rect.y + 40.0, 20.0, app::BLACK);
        self.width_field.draw();
        self.height_field.draw();
        self.ok_button.draw();
        self.cancel_button.draw();
    }

    fn update(&mut self, mouse_intercepted: &mut bool) -> Option<Layer> {
        if self.width_field.update(mouse_intercepted) {
            self.height_field.active = false;
        }
        if self.height_field.update(mouse_intercepted) {
            self.width_field.active = false;
        }
        if self.ok_button.update(mouse_intercepted) {
            if let (Ok(width), Ok(height)) = (self.width_field.text.parse(), self.height_field.text.parse()) {
                self.should_close = true;
                return Some(Layer::new(ImageRect::new(0, 0, width, height)))
            }
        }
        if self.cancel_button.update(mouse_intercepted) {
            self.should_close = true;
        }

        None
    }
}

pub struct OpenDialog {
    path_field: TextBox,
    ok_button: Button,
    cancel_button: Button,
    rect: Rect,
    pub should_close: bool,
}

impl OpenDialog {
    pub fn new(x: f32, y: f32, default_path: String) -> Self {
        let mut dialog = Self {
            path_field: TextBox::new(Rect::new(x + 70.0, y + 5.0, 100.0, 30.0)),
            ok_button: Button::new(Rect::new(x + 5.0, y + 70.0, 100.0, 30.0), "Ok".into()),
            cancel_button: Button::new(Rect::new(x + 110.0, y + 70.0, 100.0, 30.0), "Cancel".into()),
            rect: Rect::new(x, y, 250.0, 110.0),
            should_close: false,
        };
        dialog.path_field.text = default_path;
        dialog
    }
}

// TODO parse this into a path right away
impl Widget<Option<String>> for OpenDialog {
    fn draw(&self) {
        app::draw_rect(self.rect, app::GRAY);
        app::draw_text("Open Path:", self.rect.x + 5.0, self.rect.y + 10.0, 20.0, app::BLACK);
        self.path_field.draw();
        self.ok_button.draw();
        self.cancel_button.draw();
    }

    fn update(&mut self, mouse_intercepted: &mut bool) -> Option<String> {
        self.path_field.update(mouse_intercepted);
        if self.ok_button.update(mouse_intercepted) {
            return Some(self.path_field.text.clone());
        }
        if self.cancel_button.update(mouse_intercepted) {
            self.should_close = true;
        }
        None
    }
}

pub struct SaveDialog {
    path_field: TextBox,
    ok_button: Button,
    cancel_button: Button,
    rect: Rect,
    pub should_close: bool,
}

impl SaveDialog {
    pub fn new(x: f32, y: f32, default_path: String) -> Self {
        let mut dialog = Self {
            path_field: TextBox::new(Rect::new(x + 70.0, y + 5.0, 100.0, 30.0)),
            ok_button: Button::new(Rect::new(x + 5.0, y + 70.0, 100.0, 30.0), "Ok".into()),
            cancel_button: Button::new(Rect::new(x + 110.0, y + 70.0, 100.0, 30.0), "Cancel".into()),
            rect: Rect::new(x, y, 250.0, 110.0),
            should_close: false,
        };
        dialog.path_field.text = default_path;
        dialog
    }
}

// TODO parse this into a path right away
impl Widget<Option<String>> for SaveDialog {
    fn draw(&self) {
        app::draw_rect(self.rect, app::GRAY);
        app::draw_text("Save Path:", self.rect.x + 5.0, self.rect.y + 10.0, 20.0, app::BLACK);
        self.path_field.draw();
        self.ok_button.draw();
        self.cancel_button.draw();
    }

    fn update(&mut self, mouse_intercepted: &mut bool) -> Option<String> {
        self.path_field.update(mouse_intercepted);
        if self.ok_button.update(mouse_intercepted) {
            return Some(self.path_field.text.clone());
        }
        if self.cancel_button.update(mouse_intercepted) {
            self.should_close = true;
        }
        None
    }
}

pub struct Button {
    rect: Rect,
    text: String,
    text_size: f32,
    state: ButtonState,

    color: Color,
    color_hovered: Color,
    color_pressed: Color,
    text_color: Color,
    text_color_hovered: Color,
    text_color_pressed: Color,
}

impl Button {
    pub fn new(rect: Rect, text: String) -> Self {
        Self {
            rect,
            text,
            text_size: 20.0,
            state: ButtonState::Released,
            color: Color::new(0.0, 150.0/255.0, 0.0, 1.0),
            color_hovered: Color::new(0.0, 200.0/255.0, 0.0, 1.0),
            color_pressed: Color::new(0.0, 200.0/255.0, 0.0, 1.0),
            text_color: app::BLACK,
            text_color_hovered: app::BLACK,
            text_color_pressed: app::WHITE,
        }
    }
}

enum ButtonState {
    Released,
    Hovered,
    Pressed,
}

impl Widget<bool> for Button {
    fn draw(&self) {
        let (button_color, text_color) = match self.state {
            ButtonState::Released => (self.color, self.text_color),
            ButtonState::Hovered => (self.color_hovered, self.text_color_hovered),
            ButtonState::Pressed => (self.color_pressed, self.text_color_pressed),
        };
        app::draw_rect(self.rect, button_color);
        app::draw_text(&self.text, self.rect.x + 5.0, self.rect.y + 5.0, self.text_size, text_color);
    }

    fn update(&mut self, mouse_intercepted: &mut bool) -> bool {
        *mouse_intercepted = self.rect.contains(app::mouse_position().into()) && app::is_mouse_left_down() || *mouse_intercepted;
        if self.rect.contains(app::mouse_position().into()) {
            if app::is_mouse_left_down() {
                self.state = ButtonState::Pressed;
            } else {
                self.state = ButtonState::Hovered;
            }

            if app::is_mouse_left_pressed() {
                return true;
            }
        } else {
            self.state = ButtonState::Released;
        }
        false
    }
}

pub struct TextBox {
    rect: Rect,
    text: String,
    pub active: bool,
}

impl TextBox {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            text: String::new(),
            active: false,
        }
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl Widget<bool> for TextBox {
    fn draw(&self) {
        app::draw_rect(self.rect, app::BLACK);
        app::draw_rect(Rect::new(self.rect.x + 2.0, self.rect.y + 2.0, self.rect.w - 4.0, self.rect.h - 4.0), app::WHITE);
        // TODO bring this back (this draws the cursor)
        // let text_rect = app::draw_text(&self.text, self.rect.x + 4.0, self.rect.y + 4.0, 20.0, app::BLACK);
        // if self.active {
            // app::draw_rect(Rect::new(text_rect.x + text_rect.w, text_rect.y, 5.0, text_rect.h), app::BLACK);
        // }
    }

    fn update(&mut self, mouse_intercepted: &mut bool) -> bool {
        *mouse_intercepted = self.rect.contains(app::mouse_position().into()) && app::is_mouse_left_down() || *mouse_intercepted;
        self.active = (self.rect.contains(app::mouse_position().into()) && app::is_mouse_left_pressed()) || self.active;
        if self.active {
            if let Some(text) = app::get_text() {
                self.text.push_str(&text);
            }
            if app::is_key_pressed(Key::Backspace) {
                self.text.pop();
            }
        }
        self.active
    }
}

pub struct ColorSelector {
    rect: Rect,
    colors: Vec<Color>,
    rects: Vec<Rect>,
    selected_color_idx: usize,
}

impl ColorSelector {
    pub fn new(rect: Rect, colors: Vec<Color>) -> Self {
        let mut rects = Vec::new();
        let mut h = rect.h / colors.len() as f32;
        if h < 5.0 {
            h = 5.0;
        }
        let x = rect.x + 2.0;
        let mut y = rect.y + 2.0;
        for _ in &colors {
            rects.push(Rect {
                x,
                y,
                w: rect.w - 4.0,
                h: h - 4.0,
            });
            y += h;
        }

        Self {
            rect,
            colors,
            rects,
            selected_color_idx: 0,
        }
    }

    pub fn set_selected_color(&mut self, color: Color) {
        if let Some(i) = self.colors.iter().position(|c| *c == color) {
            self.selected_color_idx = i;
        }
    }
}

impl Widget<Color> for ColorSelector {
    fn draw(&self) {
        app::draw_rect(self.rect, Color::new(50.0/255.0, 50.0/255.0, 50.0/255.0, 1.0));
        for (i, rect) in self.rects.iter().enumerate() {
            if i == self.selected_color_idx {
                app::draw_rect(
                    Rect::new(
                        rect.x - 2.0,
                        rect.y - 2.0,
                        rect.w + 4.0,
                        rect.h + 4.0
                    ),
                    Color::new(1.0, 1.0, 0.0, 1.0)
                );
            }

            app::draw_rect(*rect, self.colors[i]);
        }
    }

    fn update(&mut self, mouse_intercepted: &mut bool) -> Color {
        *mouse_intercepted = self.rect.contains(app::mouse_position().into()) && app::is_mouse_left_down() || *mouse_intercepted;
        if self.rect.contains(app::mouse_position().into()) && app::is_mouse_left_pressed() {
            for (i, rect) in self.rects.iter().enumerate() {
                if rect.contains(app::mouse_position().into()) {
                    self.selected_color_idx = i;
                    return self.colors[i]
                }
            }
        }
        self.colors[self.selected_color_idx]
    }
}

pub struct ToolSelector {
    rect: Rect,
    labels: Vec<String>,
    rects: Vec<Rect>,
    selected_tool_idx: usize,
}

impl ToolSelector {
    pub fn new(rect: Rect, labels: Vec<String>) -> Self {
        let mut rects = Vec::new();
        let mut h = rect.h / labels.len() as f32;
        if h < 5.0 {
            h = 5.0;
        }
        let x = rect.x + 2.0;
        let mut y = rect.y + 2.0;
        for _ in &labels {
            rects.push(Rect {
                x,
                y,
                w: rect.w - 4.0,
                h: h - 4.0,
            });
            y += h;
        }

        Self {
            rect,
            labels,
            rects,
            selected_tool_idx: 0,
        }
    }
}

impl Widget<String> for ToolSelector {

    fn draw(&self) {
        app::draw_rect(self.rect, Color::new(20.0/255.0, 20.0/255.0, 20.0/255.0, 1.0));
        for (i, rect) in self.rects.iter().enumerate() {
            if i == self.selected_tool_idx {
                app::draw_rect(
                    Rect::new(
                        rect.x - 2.0,
                        rect.y - 2.0,
                        rect.w + 4.0,
                        rect.h + 4.0
                    ),
                    Color::new(1.0, 1.0, 0.0, 1.0)
                );
            }

            app::draw_rect(*rect, Color::new(100.0/255.0, 100.0/255.0, 100.0/255.0, 1.0));
            app::draw_text(&self.labels[i], rect.x + 4.0, rect.y + 4.0, 20.0, app::BLACK);
        }
    }

    fn update(&mut self, mouse_intercepted: &mut bool) -> String {
        *mouse_intercepted = self.rect.contains(app::mouse_position().into()) && app::is_mouse_left_down() || *mouse_intercepted;
        if self.rect.contains(app::mouse_position().into()) && app::is_mouse_left_pressed() {
            for (i, rect) in self.rects.iter().enumerate() {
                if rect.contains(app::mouse_position().into()) {
                    self.selected_tool_idx = i;
                    return self.labels[i].clone()
                }
            }
        }
        self.labels[self.selected_tool_idx].clone()
    }
}
