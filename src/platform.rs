use sdl2::rect::Rect as SdlRect;
use sdl2::pixels::Color as SdlColor;
use sdl2::pixels::PixelFormatEnum;
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::render::WindowCanvas;
use sdl2::{EventPump, Sdl, VideoSubsystem};
use sdl2::keyboard::Mod;
use rusttype::{point, Scale, PositionedGlyph, Font};
use std::collections::HashSet;

use super::util::{Rect, Color};

pub enum PlatformMessage {
    Quit,
    NoMessage,
}

pub struct Platform {
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    canvas: WindowCanvas,
    event_pump: EventPump,
    draw_color: SdlColor,
    font: Font<'static>,

    pub screen_width: i32,
    pub screen_height: i32,
    pub screen_size_changed: bool,

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
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_delta_x: i32,
    pub mouse_delta_y: i32,
    pub keys_down: HashSet<Keycode>,
    pub keys_pressed: HashSet<Keycode>,
    pub keys_released: HashSet<Keycode>,
    pub ctrl_down: bool,
    pub alt_down: bool,
    pub shift_down: bool,
    pub super_down: bool,
}

impl Platform {
    pub fn new() -> Result<Self, String> {

        let draw_color = SdlColor::RGB(0, 0, 0);
        let screen_width = 800;
        let screen_height = 600;

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("rust-sdl2 demo: Game of Life",
                screen_width as u32,
                screen_height as u32)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas()
            .target_texture()
            // .present_vsync()
            .build().map_err(|e| e.to_string())?;

        let event_pump = sdl_context.event_pump()?;

        let font = Font::try_from_bytes(include_bytes!("/usr/share/fonts/TTF/DejaVuSans.ttf") as &[u8]).unwrap();

        canvas.set_draw_color(draw_color);

        Ok(Self {
            sdl_context,
            video_subsystem,
            canvas,
            event_pump,
            draw_color,
            font,

            screen_width,
            screen_height,
            screen_size_changed: false,

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
            mouse_x: 0,
            mouse_y: 0,
            mouse_delta_x: 0,
            mouse_delta_y: 0,
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            ctrl_down: false,
            alt_down: false,
            shift_down: false,
            super_down: false,
        })
    }

    pub fn process_events(&mut self) -> PlatformMessage {

        self.screen_size_changed = false;
        self.mouse_left_pressed = false;
        self.mouse_left_released = false;
        self.mouse_right_pressed = false;
        self.mouse_right_released = false;
        self.mouse_middle_pressed = false;
        self.mouse_middle_released = false;
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_delta_x = 0;
        self.mouse_delta_y = 0;

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return PlatformMessage::Quit;
                }
                Event::KeyDown { keycode: Some(kc), keymod, .. } => {
                    self.keys_down.insert(kc);
                    self.keys_pressed.insert(kc);

                    self.ctrl_down = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD);
                    self.alt_down = keymod.contains(Mod::LALTMOD) || keymod.contains(Mod::RALTMOD);
                    self.shift_down = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);
                    self.super_down = keymod.contains(Mod::LGUIMOD) || keymod.contains(Mod::RGUIMOD);
                }
                Event::KeyUp { keycode: Some(kc), keymod, .. } => {
                    self.keys_down.remove(&kc);
                    self.keys_released.insert(kc);
                    self.ctrl_down = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD);
                    self.alt_down = keymod.contains(Mod::LALTMOD) || keymod.contains(Mod::RALTMOD);
                    self.shift_down = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);
                    self.super_down = keymod.contains(Mod::LGUIMOD) || keymod.contains(Mod::RGUIMOD);
                }
                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    // We need to set these separately from the mouse motion event to handle deltas
                    // properly on touchscreens
                    self.mouse_x = x;
                    self.mouse_y = y;
                    self.mouse_delta_x = 0;
                    self.mouse_delta_y = 0;
                    match mouse_btn {
                        MouseButton::Left => {
                            self.mouse_left_down = true;
                            self.mouse_left_pressed = true;
                        }
                        MouseButton::Middle => {
                            self.mouse_middle_down = true;
                            self.mouse_middle_pressed = true;
                        }
                        MouseButton::Right => {
                            self.mouse_right_down = true;
                            self.mouse_right_pressed = true;
                        }
                        _ => {}
                    }
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    match mouse_btn {
                        MouseButton::Left => {
                            self.mouse_left_down = false;
                            self.mouse_left_released = true;
                        }
                        MouseButton::Middle => {
                            self.mouse_middle_down = false;
                            self.mouse_middle_released = true;
                        }
                        MouseButton::Right => {
                            self.mouse_right_down = false;
                            self.mouse_right_released = true;
                        }
                        _ => {println!("{:?}", mouse_btn);}
                    }
                }
                // Event::FingerUp { .. } => {
                //     self.mouse_left_down = false;
                //     self.mouse_left_released = true;
                // }
                Event::MouseMotion { x, y, .. } => {
                    self.mouse_delta_x = self.mouse_delta_x + x - self.mouse_x;
                    self.mouse_delta_y = self.mouse_delta_y + y - self.mouse_y;
                    self.mouse_x = x;
                    self.mouse_y = y;
                }
                Event::MouseWheel { x, y, .. } => {
                    self.scroll_delta_x += x;
                    self.scroll_delta_y += y;
                }
                Event::Window { win_event, .. } => {
                    if let WindowEvent::Resized(w, h) = win_event {
                        self.screen_size_changed = true;
                        self.screen_width = w;
                        self.screen_height = h;
                    }
                }
                _ => {}
            }
        }
        PlatformMessage::NoMessage
    }

    pub fn get_scroll_delta_x(&mut self) -> i32 {
        let delta = self.scroll_delta_x;
        self.scroll_delta_x = 0;
        delta
    }
    pub fn get_scroll_delta_y(&mut self) -> i32 {
        let delta = self.scroll_delta_y;
        self.scroll_delta_y = 0;
        delta
    }
    pub fn key_down(&self, key: Keycode) -> bool {
        self.keys_down.contains(&key)
    }
    pub fn key_pressed(&self, key: Keycode) -> bool {
        self.keys_pressed.contains(&key)
    }
    pub fn key_released(&self, key: Keycode) -> bool {
        self.keys_released.contains(&key)
    }

    pub fn text_entered(&self) -> String {
        // TODO this would probably be a better fit for SDLs text entry event, I'll just have to
        // make sure we still handle keyboard shortcuts properly
        let mut s = String::new();
        for key in &self.keys_pressed {
            let new_char = match key {
                Keycode::Num1 => "1",
                Keycode::Num2 => "2",
                Keycode::Num3 => "3",
                Keycode::Num4 => "4",
                Keycode::Num5 => "5",
                Keycode::Num6 => "6",
                Keycode::Num7 => "7",
                Keycode::Num8 => "8",
                Keycode::Num9 => "9",
                Keycode::Num0 => "0",
                Keycode::A => if self.shift_down { "A" } else { "a" },
                Keycode::B => if self.shift_down { "B" } else { "b" },
                Keycode::C => if self.shift_down { "C" } else { "c" },
                Keycode::D => if self.shift_down { "D" } else { "d" },
                Keycode::E => if self.shift_down { "E" } else { "e" },
                Keycode::F => if self.shift_down { "F" } else { "f" },
                Keycode::G => if self.shift_down { "G" } else { "g" },
                Keycode::H => if self.shift_down { "H" } else { "h" },
                Keycode::I => if self.shift_down { "I" } else { "i" },
                Keycode::J => if self.shift_down { "J" } else { "j" },
                Keycode::K => if self.shift_down { "K" } else { "k" },
                Keycode::L => if self.shift_down { "L" } else { "l" },
                Keycode::M => if self.shift_down { "M" } else { "m" },
                Keycode::N => if self.shift_down { "N" } else { "n" },
                Keycode::O => if self.shift_down { "O" } else { "o" },
                Keycode::P => if self.shift_down { "P" } else { "p" },
                Keycode::Q => if self.shift_down { "Q" } else { "q" },
                Keycode::R => if self.shift_down { "R" } else { "r" },
                Keycode::S => if self.shift_down { "S" } else { "s" },
                Keycode::T => if self.shift_down { "T" } else { "t" },
                Keycode::U => if self.shift_down { "U" } else { "u" },
                Keycode::V => if self.shift_down { "V" } else { "v" },
                Keycode::W => if self.shift_down { "W" } else { "w" },
                Keycode::X => if self.shift_down { "X" } else { "x" },
                Keycode::Y => if self.shift_down { "Y" } else { "y" },
                Keycode::Z => if self.shift_down { "Z" } else { "z" },
                Keycode::Space => " ",
                Keycode::Caret => "^",
                Keycode::Kp0 => "0",
                Keycode::Kp1 => "1",
                Keycode::Kp2 => "2",
                Keycode::Kp3 => "3",
                Keycode::Kp4 => "4",
                Keycode::Kp5 => "5",
                Keycode::Kp6 => "6",
                Keycode::Kp7 => "7",
                Keycode::Kp8 => "8",
                Keycode::Kp9 => "9",
                Keycode::KpPlus => "+",
                // Keycode::Apostrophe => if self.shift_down { "\"" } else { "'" },
                Keycode::At => "@",
                Keycode::Backslash => if self.shift_down { "|" } else { "\\" },
                Keycode::Colon => ":",
                Keycode::Comma => if self.shift_down { "<" } else { "," },
                // Keycode::Decimal => ".",
                Keycode::KpDivide => "/",
                Keycode::Equals => "=",
                // Keycode::Grave => if self.shift_down { "~" } else { "`" },
                Keycode::LeftBracket => if self.shift_down { "{" } else { "[" },
                Keycode::Minus => if self.shift_down { "_" } else { "-" },
                Keycode::KpMultiply => "*",
                Keycode::KpComma => ",",
                Keycode::KpEnter => "\n",
                Keycode::KpEquals => "=",
                Keycode::Period => if self.shift_down { ">" } else { "." },
                Keycode::RightBracket => if self.shift_down { "}" } else { "]" },
                Keycode::Semicolon => if self.shift_down { ":" } else { ";" },
                Keycode::Slash => "",
                Keycode::KpMinus => "-",
                Keycode::Tab => "    ",
                _ => "",
            };
            s.push_str(new_char);
        }
        s
    }

    pub fn clear(&mut self, color: Color) {
        let color = SdlColor::RGBA(color.r, color.g, color.b, color.a);
        if self.draw_color != color {
            self.canvas.set_draw_color(color);
            self.draw_color = color;
        }
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        if rect.width > 0 && rect.height > 0 {
            let rect = SdlRect::new(rect.x, rect.y, rect.width, rect.height);
            let color = SdlColor::RGBA(color.r, color.g, color.b, color.a);
            if self.draw_color != color {
                self.canvas.set_draw_color(color);
                self.draw_color = color;
            }
            self.canvas.fill_rect(rect).unwrap();
        }
    }

    pub fn draw_texture(&mut self, buffer: &mut [u8], src_rect: Rect, dest_rect: Rect) {
        // TODO this doesn't actually take a subset of the buffer, so if we want a partial
        // texture this'll have to change. For now src_rect just tells you how big the data
        // is.
        let surface = Surface::from_data(buffer, src_rect.width, src_rect.height, 4 * src_rect.width, PixelFormatEnum::ABGR8888).unwrap();
        let creator = self.canvas.texture_creator();
        let texture = creator.create_texture_from_surface(&surface).unwrap();
        let dest_rect = SdlRect::new(dest_rect.x, dest_rect.y, dest_rect.width, dest_rect.height);
        self.canvas.copy(&texture, None, Some(dest_rect)).unwrap();
    }

    pub fn layout_text(&mut self, text: &str, scale: f32) -> (Vec<PositionedGlyph<'_>>, usize, usize) {
        let font_scale = Scale::uniform(scale);
        let v_metrics = self.font.v_metrics(font_scale);
        let glyphs: Vec<_> = self.font
            .layout(text, font_scale, point(0.0, 0.0 + v_metrics.ascent))
            .collect();

        let height = (v_metrics.ascent - v_metrics.descent).ceil() as usize;
        let width = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;
        (glyphs, width, height)
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, scale: f32, color: Color) -> Rect {
        // Save the original parameters to return in the rect
        if text.is_empty() {
            return Rect::new(x, y, 0, 0);
        }

        let input_x = x;
        let input_y = y;

        let (glyphs, glyphs_width, glyphs_height) = self.layout_text(text, scale);
        
        let mut buffer: Vec<f32> = vec![0.0; glyphs_width * glyphs_height];

        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {

                let min_x = bounding_box.min.x;
                let min_y = bounding_box.min.y;

                glyph.draw(|x, y, v| {
                    let x = std::cmp::max(x as i32 + min_x, 1) as usize - 1;
                    let y = std::cmp::max(y as i32 + min_y, 1) as usize - 1;
                    let index = y * glyphs_width + x;
                    buffer[index] = v;
                });
            }
        }

        let rect = Rect::new(input_x, input_y, glyphs_width as u32, glyphs_height as u32);
        let mut texture_buffer = Vec::new();
        for a in buffer {
            texture_buffer.push(color.r);
            texture_buffer.push(color.g);
            texture_buffer.push(color.b);
            texture_buffer.push((color.a as f32 * a) as u8);
        }
        self.draw_texture(&mut texture_buffer, Rect::new(0, 0, rect.width, rect.height), rect);

        rect
    }
}
