use std::collections::HashSet;
use glutin::event::{Event, WindowEvent, VirtualKeyCode, ElementState, MouseButton};

pub struct InputState {
    pub mouse_left_down: bool,
    pub mouse_right_down: bool,
    pub mouse_middle_down: bool,
    pub mouse_left_pressed: bool,
    pub mouse_right_pressed: bool,
    pub mouse_middle_pressed: bool,
    pub mouse_left_released: bool,
    pub mouse_right_released: bool,
    pub mouse_middle_released: bool,
    scroll_delta_x: i32,
    scroll_delta_y: i32,
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub mouse_delta_x: f64,
    pub mouse_delta_y: f64,
    pub keys_down: HashSet<VirtualKeyCode>,
    pub keys_pressed: HashSet<VirtualKeyCode>,
    pub keys_released: HashSet<VirtualKeyCode>,
    pub ctrl_down: bool,
    pub alt_down: bool,
    pub shift_down: bool,
    pub super_down: bool,
}

#[allow(dead_code)]
impl InputState {
    pub fn new() -> Self {
        Self {
            mouse_left_down: false,
            mouse_right_down: false,
            mouse_middle_down: false,
            mouse_left_pressed: false,
            mouse_right_pressed: false,
            mouse_middle_pressed: false,
            mouse_left_released: false,
            mouse_right_released: false,
            mouse_middle_released: false,
            scroll_delta_x: 0,
            scroll_delta_y: 0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_delta_x: 0.0,
            mouse_delta_y: 0.0,
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            ctrl_down: false,
            alt_down: false,
            shift_down: false,
            super_down: false,
        }
    }

    pub fn update(&mut self, event: &Event<()>) {
        self.mouse_left_pressed = false;
        self.mouse_left_released = false;
        self.mouse_right_pressed = false;
        self.mouse_right_released = false;
        self.mouse_middle_pressed = false;
        self.mouse_middle_released = false;
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_delta_x = 0.0;
        self.mouse_delta_y = 0.0;
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::MouseInput { button, state, .. } => {
                    match (button, state) {
                        (MouseButton::Left, ElementState::Pressed) => {
                            self.mouse_left_down = true;
                            self.mouse_left_pressed = true;
                        }
                        (MouseButton::Right, ElementState::Pressed) => {
                            self.mouse_right_down = true;
                            self.mouse_right_pressed = true;
                        }
                        (MouseButton::Middle, ElementState::Pressed) => {
                            self.mouse_middle_down = true;
                            self.mouse_middle_pressed = true;
                        }
                        (MouseButton::Left, ElementState::Released) => {
                            self.mouse_left_down = false;
                            self.mouse_left_released = true;
                        }
                        (MouseButton::Right, ElementState::Released) => {
                            self.mouse_right_down = false;
                            self.mouse_right_released = true;
                        }
                        (MouseButton::Middle, ElementState::Released) => {
                            self.mouse_middle_down = false;
                            self.mouse_middle_released = true;
                        }
                        _ => {}
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            self.keys_down.insert(keycode);
                            self.keys_pressed.insert(keycode);
                        } else {
                            self.keys_down.remove(&keycode);
                            self.keys_released.insert(keycode);
                        }
                    }
                    // I'll use the new API when it's supported on web
                    #[allow(deprecated)]
                    {
                        self.ctrl_down = input.modifiers.ctrl();
                        self.alt_down = input.modifiers.alt();
                        self.shift_down = input.modifiers.shift();
                        self.super_down = input.modifiers.logo();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_delta_x = position.x - self.mouse_x;
                    self.mouse_delta_y = position.y - self.mouse_y;
                    self.mouse_x = position.x;
                    self.mouse_y = position.y;
                }
                _ => {}
            }
        }
    }

    pub fn get_scroll_delta_x(&mut self) -> i32 {
        let delta = self.scroll_delta_x;
        self.scroll_delta_x = 0;
        return delta;
    }
    pub fn get_scroll_delta_y(&mut self) -> i32 {
        let delta = self.scroll_delta_y;
        self.scroll_delta_y = 0;
        return delta;
    }
    pub fn key_down(&self, key: VirtualKeyCode) -> bool {
        self.keys_down.contains(&key)
    }
    pub fn key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }
    pub fn key_released(&self, key: VirtualKeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    pub fn text_entered(&self) -> String {
        let mut s = String::new();
        for key in &self.keys_pressed {
            let new_char = match key {
                VirtualKeyCode::Key1 => "1",
                VirtualKeyCode::Key2 => "2",
                VirtualKeyCode::Key3 => "3",
                VirtualKeyCode::Key4 => "4",
                VirtualKeyCode::Key5 => "5",
                VirtualKeyCode::Key6 => "6",
                VirtualKeyCode::Key7 => "7",
                VirtualKeyCode::Key8 => "8",
                VirtualKeyCode::Key9 => "9",
                VirtualKeyCode::Key0 => "0",
                VirtualKeyCode::A => if self.shift_down { "A" } else { "a" },
                VirtualKeyCode::B => if self.shift_down { "B" } else { "b" },
                VirtualKeyCode::C => if self.shift_down { "C" } else { "c" },
                VirtualKeyCode::D => if self.shift_down { "D" } else { "d" },
                VirtualKeyCode::E => if self.shift_down { "E" } else { "e" },
                VirtualKeyCode::F => if self.shift_down { "F" } else { "f" },
                VirtualKeyCode::G => if self.shift_down { "G" } else { "g" },
                VirtualKeyCode::H => if self.shift_down { "H" } else { "h" },
                VirtualKeyCode::I => if self.shift_down { "I" } else { "i" },
                VirtualKeyCode::J => if self.shift_down { "J" } else { "j" },
                VirtualKeyCode::K => if self.shift_down { "K" } else { "k" },
                VirtualKeyCode::L => if self.shift_down { "L" } else { "l" },
                VirtualKeyCode::M => if self.shift_down { "M" } else { "m" },
                VirtualKeyCode::N => if self.shift_down { "N" } else { "n" },
                VirtualKeyCode::O => if self.shift_down { "O" } else { "o" },
                VirtualKeyCode::P => if self.shift_down { "P" } else { "p" },
                VirtualKeyCode::Q => if self.shift_down { "Q" } else { "q" },
                VirtualKeyCode::R => if self.shift_down { "R" } else { "r" },
                VirtualKeyCode::S => if self.shift_down { "S" } else { "s" },
                VirtualKeyCode::T => if self.shift_down { "T" } else { "t" },
                VirtualKeyCode::U => if self.shift_down { "U" } else { "u" },
                VirtualKeyCode::V => if self.shift_down { "V" } else { "v" },
                VirtualKeyCode::W => if self.shift_down { "W" } else { "w" },
                VirtualKeyCode::X => if self.shift_down { "X" } else { "x" },
                VirtualKeyCode::Y => if self.shift_down { "Y" } else { "y" },
                VirtualKeyCode::Z => if self.shift_down { "Z" } else { "z" },
                VirtualKeyCode::Space => " ",
                VirtualKeyCode::Caret => "^",
                VirtualKeyCode::Numlock => "",
                VirtualKeyCode::Numpad0 => "0",
                VirtualKeyCode::Numpad1 => "1",
                VirtualKeyCode::Numpad2 => "2",
                VirtualKeyCode::Numpad3 => "3",
                VirtualKeyCode::Numpad4 => "4",
                VirtualKeyCode::Numpad5 => "5",
                VirtualKeyCode::Numpad6 => "6",
                VirtualKeyCode::Numpad7 => "7",
                VirtualKeyCode::Numpad8 => "8",
                VirtualKeyCode::Numpad9 => "9",
                VirtualKeyCode::Add => "+",
                VirtualKeyCode::Apostrophe => if self.shift_down { "\"" } else { "'" },
                VirtualKeyCode::At => "@",
                VirtualKeyCode::Backslash => if self.shift_down { "|" } else { "\\" },
                VirtualKeyCode::Colon => ":",
                VirtualKeyCode::Comma => if self.shift_down { "<" } else { "," },
                VirtualKeyCode::Decimal => ".",
                VirtualKeyCode::Divide => "/",
                VirtualKeyCode::Equals => "=",
                VirtualKeyCode::Grave => if self.shift_down { "~" } else { "`" },
                VirtualKeyCode::LBracket => if self.shift_down { "" } else { "" },
                VirtualKeyCode::Minus => if self.shift_down { "_" } else { "-" },
                VirtualKeyCode::Multiply => "*",
                VirtualKeyCode::NumpadComma => ",",
                VirtualKeyCode::NumpadEnter => "\n",
                VirtualKeyCode::NumpadEquals => "=",
                VirtualKeyCode::Period => if self.shift_down { ">" } else { "." },
                VirtualKeyCode::RBracket => if self.shift_down { "}" } else { "]" },
                VirtualKeyCode::Semicolon => if self.shift_down { ":" } else { ";" },
                VirtualKeyCode::Slash => "",
                VirtualKeyCode::Subtract => "-",
                VirtualKeyCode::Tab => "    ",
                _ => "",
            };
            s.push_str(new_char);
        }
        s
    }
}
