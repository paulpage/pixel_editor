use std::collections::HashMap;
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
    keys_down: HashMap<VirtualKeyCode, ()>,
    keys_pressed: HashMap<VirtualKeyCode, ()>,
    keys_released: HashMap<VirtualKeyCode, ()>,
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
            keys_down: HashMap::new(),
            keys_pressed: HashMap::new(),
            keys_released: HashMap::new(),
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
                            self.keys_down.insert(keycode, ());
                            self.keys_pressed.insert(keycode, ());
                        } else {
                            self.keys_down.remove(&keycode);
                            self.keys_released.insert(keycode, ());
                        }
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
        self.keys_down.contains_key(&key)
    }
    pub fn key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keys_pressed.contains_key(&key)
    }
    pub fn key_released(&self, key: VirtualKeyCode) -> bool {
        self.keys_released.contains_key(&key)
    }
}
