pub use macroquad::prelude::*;
pub use macroquad::prelude::KeyCode as Key;
pub use macroquad::prelude::MouseButton as Button;

#[macro_export]
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr $(,)?) => (
        Rect::new($x as f32, $y as f32, $w as f32, $h as f32)
    )
);

#[macro_export]
macro_rules! vec2(
    ($x:expr, $y:expr $(,)?) => (
        Vec2::new($x as f32, $y as f32)
    )
);


#[macro_export]
macro_rules! color(
    ($r:expr, $g:expr, $b:expr, $a:expr $(,)?) => (
        Color::new(
            $r as f32 / 255.0,
            $g as f32 / 255.0,
            $b as f32 / 255.0,
            $a as f32 / 255.0,
        )
    );

    ($r:expr, $g:expr, $b:expr $(,)?) => (
        Color::new(
            $r as f32 / 255.0,
            $g as f32 / 255.0,
            $b as f32 / 255.0,
            1.0,
        )
    );
);

pub fn draw_rect(rect: Rect, color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
}

pub fn is_mouse_left_down() -> bool {
    is_mouse_button_down(MouseButton::Left)
}
pub fn is_mouse_right_down() -> bool {
    is_mouse_button_down(MouseButton::Right)
}
pub fn is_mouse_middle_down() -> bool {
    is_mouse_button_down(MouseButton::Middle)
}
pub fn is_mouse_left_pressed() -> bool {
    is_mouse_button_pressed(MouseButton::Left)
}
pub fn is_mouse_right_pressed() -> bool {
    is_mouse_button_pressed(MouseButton::Right)
}
pub fn is_mouse_middle_pressed() -> bool {
    is_mouse_button_pressed(MouseButton::Middle)
}

pub fn is_ctrl_down() -> bool {
    is_key_down(Key::LeftControl) || is_key_down(Key::RightControl)
}

pub fn is_alt_down() -> bool {
    is_key_down(Key::LeftAlt) || is_key_down(Key::RightAlt)
}

pub fn is_shift_down() -> bool {
    is_key_down(Key::LeftShift) || is_key_down(Key::RightShift)
}

pub fn is_super_down() -> bool {
    is_key_down(Key::LeftSuper) || is_key_down(Key::RightSuper)
}

fn is_printable(c: char) -> bool {
    // Check if it's a printable ASCII character
    if c.is_ascii() {
        return c.is_ascii_graphic() || c.is_ascii_whitespace();
    }

    // For non-ASCII characters, we need to check Unicode categories
    match c {
        // Exclude control characters and non-characters
        '\u{0000}'..='\u{001F}' | '\u{007F}'..='\u{009F}' | '\u{FDD0}'..='\u{FDEF}' => false,
        
        // Exclude private use area
        '\u{E000}'..='\u{F8FF}' | '\u{F0000}'..='\u{FFFFD}' | '\u{100000}'..='\u{10FFFD}' => false,
        
        // Exclude non-characters
        '\u{FFF0}'..='\u{FFFF}' | '\u{1FFFE}'..='\u{1FFFF}' | '\u{2FFFE}'..='\u{2FFFF}' |
        '\u{3FFFE}'..='\u{3FFFF}' | '\u{4FFFE}'..='\u{4FFFF}' | '\u{5FFFE}'..='\u{5FFFF}' |
        '\u{6FFFE}'..='\u{6FFFF}' | '\u{7FFFE}'..='\u{7FFFF}' | '\u{8FFFE}'..='\u{8FFFF}' |
        '\u{9FFFE}'..='\u{9FFFF}' | '\u{AFFFE}'..='\u{AFFFF}' | '\u{BFFFE}'..='\u{BFFFF}' |
        '\u{CFFFE}'..='\u{CFFFF}' | '\u{DFFFE}'..='\u{DFFFF}' | '\u{EFFFE}'..='\u{EFFFF}' |
        '\u{FFFFE}'..='\u{FFFFF}' | '\u{10FFFE}'..='\u{10FFFF}' => false,
        
        // All other characters are considered printable
        _ => true,
    }
}

pub fn get_text() -> Option<String> {
    let mut text = None;
    while let Some(c) = get_char_pressed() {
        println!("c = {} ({} as u32)", c, c as u32);
        if is_printable(c) && !is_ctrl_down() && !is_alt_down() && !is_super_down() {
            text.get_or_insert(String::new()).push_str(&c.to_string());
        }
    }
    text
}
