#![allow(dead_code)]
use std::cmp::{min, max};

pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width as i32 && y >= self.y && y < self.y + self.height as i32
    }

    pub fn union(&self, other: Rect) -> Rect {
        if self.width == 0 || self.height == 0 {
            return other;
        }
        if other.width == 0 || other.height == 0 {
            return *self;
        }
        let r = Rect {
            x: min(self.x, other.x),
            y: min(self.y, other.y),
            width: (max(self.x + self.width as i32, other.x + other.width as i32) - min(self.x, other.x)) as u32,
            height: (max(self.y + self.height as i32, other.y + other.height as i32) - min(self.y, other.y)) as u32,
        };
        return r;
    }

    pub fn has_intersection(&self, other: Rect) -> bool {
        (self.x <= other.x + other.width as i32 && self.x + self.width as i32 >= other.x
         && self.y <= other.y + other.height as i32 && self.y + self.height as i32 >= other.y)
    }

    pub fn intersection(&self, other: Rect) -> Rect {
        if !Rect::has_intersection(self, other) {
            return Rect::new(0, 0, 0, 0);
        }
        Rect {
            x: max(self.x, other.x),
            y: max(self.y, other.y),
            width: (min(self.x + self.width as i32, other.x + other.width as i32) - max(self.x, other.x)) as u32,
            height: (min(self.y + self.height as i32, other.y + other.height as i32) - max(self.y, other.y)) as u32,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(color_hex_str: &str) -> Result<Self, std::num::ParseIntError> {
        let color = i32::from_str_radix(color_hex_str, 16)?;
        let b = color % 0x100;
        let g = (color - b) / 0x100 % 0x100;
        let r = (color - g) / 0x10000;

        Ok(Self {
            r: r as u8,
            g: g as u8,
            b: b as u8,
            a: 255,
        })
    }

    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const GRAY: Self = Self {
        r: 150,
        g: 150,
        b: 150,
        a: 255,
    };

}
