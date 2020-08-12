use super::util::{Color, Rect};
use super::graphics::Graphics;
use super::input::InputState;

pub trait Widget<T> {
    fn draw(&self, graphics: &Graphics);
    fn update(&mut self, input: &InputState) -> T;
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
    fn draw(&self, graphics: &Graphics) {
        let (button_color, text_color) = match self.state {
            ButtonState::Released => (self.color, self.text_color),
            ButtonState::Hovered => (self.color_hovered, self.text_color_hovered),
            ButtonState::Pressed => (self.color_pressed, self.text_color_pressed),
        };
        graphics.draw_rect(self.rect, button_color);
        graphics.draw_text(&self.text, self.rect.x + 5, self.rect.y + 5, self.text_size, text_color);
    }

    fn update(&mut self, input: &InputState) -> bool {
        if self.rect.contains_point(input.mouse_x as i32, input.mouse_y as i32) {
            if input.mouse_left_down {
                self.state = ButtonState::Pressed;
            } else {
                self.state = ButtonState::Hovered;
            }

            if input.mouse_left_pressed {
                return true;
            }
        } else {
            self.state = ButtonState::Released;
        }
        false
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
}

impl Widget<Color> for ColorSelector {
    fn draw(&self, graphics: &Graphics) {
        graphics.draw_rect(self.rect, Color::new(50, 50, 50, 255));
        for (i, rect) in self.rects.iter().enumerate() {
            if i == self.selected_color_idx {
                graphics.draw_rect(
                    Rect::new(
                        rect.x - 2,
                        rect.y - 2,
                        rect.width + 4,
                        rect.height + 4
                    ),
                    Color::new(255, 255, 0, 255)
                );
            }

            graphics.draw_rect(*rect, self.colors[i]);
        }
    }

    fn update(&mut self, input: &InputState) -> Color {
        if self.rect.contains_point(input.mouse_x as i32, input.mouse_y as i32) && input.mouse_left_pressed {
            for (i, rect) in self.rects.iter().enumerate() {
                if rect.contains_point(input.mouse_x as i32, input.mouse_y as i32) {
                    self.selected_color_idx = i;
                    return self.colors[i];
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

    fn draw(&self, graphics: &Graphics) {
        graphics.draw_rect(self.rect, Color::new(20, 20, 20, 255));
        for (i, rect) in self.rects.iter().enumerate() {
            if i == self.selected_tool_idx {
                graphics.draw_rect(
                    Rect::new(
                        rect.x - 2,
                        rect.y - 2,
                        rect.width + 4,
                        rect.height + 4
                    ),
                    Color::new(255, 255, 0, 255)
                );
            }

            graphics.draw_rect(*rect, Color::new(100, 100, 100, 255));
            graphics.draw_text(&self.labels[i], rect.x + 4, rect.y + 4, 20.0, Color::BLACK);
        }
    }

    fn update(&mut self, input: &InputState) -> String {
        if self.rect.contains_point(input.mouse_x as i32, input.mouse_y as i32) && input.mouse_left_pressed {
            for (i, rect) in self.rects.iter().enumerate() {
                if rect.contains_point(input.mouse_x as i32, input.mouse_y as i32) {
                    self.selected_tool_idx = i;
                    return self.labels[i].clone();
                }
            }
        }
        self.labels[self.selected_tool_idx].clone()
    }
}
