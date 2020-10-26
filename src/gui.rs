use sdl2::keyboard::Keycode as Key;

use super::util::{Color, Rect};
use super::platform::Platform;
use super::layer::{Layer, Image};

pub trait Widget<T> {
    fn draw(&self, p: &mut Platform);
    fn update(&mut self, platform: &mut Platform, click_intercepted: &mut bool) -> T;
}

pub struct ConfirmationDialog {
    message: String,
    buttons: Vec<Button>,
    rect: Rect,
    pub showing: bool,
}

impl ConfirmationDialog {
    pub fn new(p: &mut Platform, x: i32, y: i32, message: String, options: Vec<String>) -> Self {
        let (_, text_width, _) = p.layout_text(&message, 20.0);
        let width = std::cmp::max(
            text_width as u32 + 10,
            options.len() as u32 * 105 + 5
        );
        let rect = Rect::new(x, y, width, 200);
        let mut buttons = Vec::new();
        for (i, option) in options.iter().enumerate() {
            buttons.push(Button::new(Rect::new(x + 5 + i as i32 * 105, y + 50, 100, 50), option.to_string()));
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
    fn draw(&self, p: &mut Platform) {
        if self.showing {
            p.draw_rect(self.rect, Color::GRAY);
            p.draw_text(&self.message, self.rect.x + 5, self.rect.y + 5, 20.0, Color::BLACK);
            for button in &self.buttons {
                button.draw(p);
            }
        }
    }

    fn update(&mut self, p: &mut Platform, mouse_intercepted: &mut bool) -> Option<String> {
        if self.showing {
            // *mouse_intercepted = self.rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) && p.mouse_left_down || *mouse_intercepted;
            for button in &mut self.buttons {
                if button.update(p, mouse_intercepted) {
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
    pub fn new(x: i32, y: i32, default_width: u32, default_height: u32) -> Self {
        let mut dialog = Self {
            width_field: TextBox::new(Rect::new(x + 70, y + 5, 100, 30)),
            height_field: TextBox::new(Rect::new(x + 70, y + 35, 100, 30)),
            ok_button: Button::new(Rect::new(x + 5, y + 70, 100, 30), "Ok".into()),
            cancel_button: Button::new(Rect::new(x + 110, y + 70, 100, 30), "Cancel".into()),
            rect: Rect::new(x, y, 250, 110),
            should_close: false,
        };
        dialog.width_field.text = default_width.to_string();
        dialog.height_field.text = default_height.to_string();
        dialog
    }
}

impl Widget<Option<Layer>> for NewDialog {
    fn draw(&self, p: &mut Platform) {
        p.draw_rect(self.rect, Color::GRAY);
        p.draw_text("Width:", self.rect.x + 5, self.rect.y + 10, 20.0, Color::BLACK);
        p.draw_text("Height:", self.rect.x + 5, self.rect.y + 40, 20.0, Color::BLACK);
        self.width_field.draw(p);
        self.height_field.draw(p);
        self.ok_button.draw(p);
        self.cancel_button.draw(p);
    }

    fn update(&mut self, p: &mut Platform, mouse_intercepted: &mut bool) -> Option<Layer> {
        // TODO
        if self.width_field.update(p, mouse_intercepted) {
            self.height_field.active = false;
        }
        if self.height_field.update(p, mouse_intercepted) {
            self.width_field.active = false;
        }
        if self.ok_button.update(p, mouse_intercepted) {
            if let (Ok(width), Ok(height)) = (self.width_field.text.parse(), self.height_field.text.parse()) {
                self.should_close = true;
                return Some(Layer::new(Rect::new(0, 0, width, height)))
            }
        }
        if self.cancel_button.update(p, mouse_intercepted) {
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
            color: Color::new(0, 150, 0, 255),
            color_hovered: Color::new(0, 200, 0, 255),
            color_pressed: Color::new(0, 100, 0, 255),
            text_color: Color::BLACK,
            text_color_hovered: Color::BLACK,
            text_color_pressed: Color::WHITE,
        }
    }
}

enum ButtonState {
    Released,
    Hovered,
    Pressed,
}

impl Widget<bool> for Button {
    fn draw(&self, p: &mut Platform) {
        let (button_color, text_color) = match self.state {
            ButtonState::Released => (self.color, self.text_color),
            ButtonState::Hovered => (self.color_hovered, self.text_color_hovered),
            ButtonState::Pressed => (self.color_pressed, self.text_color_pressed),
        };
        p.draw_rect(self.rect, button_color);
        p.draw_text(&self.text, self.rect.x + 5, self.rect.y + 5, self.text_size, text_color);
    }

    fn update(&mut self, p: &mut Platform, mouse_intercepted: &mut bool) -> bool {
        *mouse_intercepted = self.rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) && p.mouse_left_down || *mouse_intercepted;
        if self.rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) {
            if p.mouse_left_down {
                self.state = ButtonState::Pressed;
            } else {
                self.state = ButtonState::Hovered;
            }

            if p.mouse_left_pressed {
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
    fn draw(&self, p: &mut Platform) {
        p.draw_rect(self.rect, Color::BLACK);
        p.draw_rect(Rect::new(self.rect.x + 2, self.rect.y + 2, self.rect.width - 4, self.rect.height - 4), Color::WHITE);
        let text_rect = p.draw_text(&self.text, self.rect.x + 4, self.rect.y + 4, 20.0, Color::BLACK);
        if self.active {
            p.draw_rect(Rect::new(text_rect.x + text_rect.width as i32, text_rect.y, 5, text_rect.height), Color::BLACK);
        }
    }

    fn update(&mut self, p: &mut Platform, mouse_intercepted: &mut bool) -> bool {
        *mouse_intercepted = self.rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) && p.mouse_left_down || *mouse_intercepted;
        self.active = (self.rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) && p.mouse_left_pressed) || self.active;
        if self.active {
            self.text.push_str(&p.text_entered());
            for key in &p.keys_pressed {
                match key {
                    Key::Backspace => {
                        self.text.pop();
                    }
                    _ => {}
                }
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
        let mut height = rect.height / colors.len() as u32;
        if height < 5 {
            height = 5;
        }
        let x = rect.x + 2;
        let mut y = rect.y + 2;
        for _ in &colors {
            rects.push(Rect {
                x,
                y,
                width: rect.width - 4,
                height: height - 4,
            });
            y += height as i32;
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
    fn draw(&self, p: &mut Platform) {
        p.draw_rect(self.rect, Color::new(50, 50, 50, 255));
        for (i, rect) in self.rects.iter().enumerate() {
            if i == self.selected_color_idx {
                p.draw_rect(
                    Rect::new(
                        rect.x - 2,
                        rect.y - 2,
                        rect.width + 4,
                        rect.height + 4
                    ),
                    Color::new(255, 255, 0, 255)
                );
            }

            p.draw_rect(*rect, self.colors[i]);
        }
    }

    fn update(&mut self, p: &mut Platform, mouse_intercepted: &mut bool) -> Color {
        *mouse_intercepted = self.rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) && p.mouse_left_down || *mouse_intercepted;
        if self.rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) && p.mouse_left_pressed {
            for (i, rect) in self.rects.iter().enumerate() {
                if rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) {
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
        let mut height = rect.height / labels.len() as u32;
        if height < 5 {
            height = 5;
        }
        let x = rect.x + 2;
        let mut y = rect.y + 2;
        for _ in &labels {
            rects.push(Rect {
                x,
                y,
                width: rect.width - 4,
                height: height - 4,
            });
            y += height as i32;
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

    fn draw(&self, p: &mut Platform) {
        p.draw_rect(self.rect, Color::new(20, 20, 20, 255));
        for (i, rect) in self.rects.iter().enumerate() {
            if i == self.selected_tool_idx {
                p.draw_rect(
                    Rect::new(
                        rect.x - 2,
                        rect.y - 2,
                        rect.width + 4,
                        rect.height + 4
                    ),
                    Color::new(255, 255, 0, 255)
                );
            }

            p.draw_rect(*rect, Color::new(100, 100, 100, 255));
            p.draw_text(&self.labels[i], rect.x + 4, rect.y + 4, 20.0, Color::BLACK);
        }
    }

    fn update(&mut self, p: &mut Platform, mouse_intercepted: &mut bool) -> String {
        *mouse_intercepted = self.rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) && p.mouse_left_down || *mouse_intercepted;
        if self.rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) && p.mouse_left_pressed {
            for (i, rect) in self.rects.iter().enumerate() {
                if rect.contains_point(p.mouse_x as i32, p.mouse_y as i32) {
                    self.selected_tool_idx = i;
                    return self.labels[i].clone()
                }
            }
        }
        self.labels[self.selected_tool_idx].clone()
    }
}
