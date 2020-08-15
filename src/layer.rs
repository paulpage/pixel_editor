#![allow(dead_code)]

use image::RgbaImage;
use image::error::ImageError;
use super::util::{Rect, Color};
use std::path::Path;
use std::collections::VecDeque;


pub struct Layer {
    pub rect: Rect,
    pub data: image::RgbaImage,
    pub z_index: i32,
}

impl Layer {
    pub fn new(rect: Rect) -> Self {
        let data = RgbaImage::from_pixel(rect.width, rect.height, [255, 255, 255, 255].into());
        Self {
            rect,
            data,
            z_index: 0,
        }
    }

    pub fn from_path(x: i32, y: i32, path: &str) -> Result<Self, ImageError> {
        let img = image::open(path)?.to_rgba();
        Ok(Self {
            rect: Rect::new(x, y, img.width(), img.height()),
            data: img,
            z_index: 0,
        })
    }

    pub fn draw_pixel(&mut self, x: i32, y: i32, color: Color) {
        if self.rect.contains_point(x, y) {
            self.data.put_pixel(x as u32, y as u32, [color.r, color.g, color.b, color.a].into());
        }
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
        self.draw_pixel(x1, y1, color);
        self.draw_pixel(x2, y2, color);
        let width = (x2 - x1).abs();
        let height = (y2 - y1).abs();
        let step = std::cmp::max(width, height);
        if step != 0 {
            let dx = (x2 as f64 - x1 as f64) / step as f64;
            let dy = (y2 as f64 - y1 as f64) / step as f64;
            for i in 0..step {
                self.draw_pixel((x1 as f64 + dx * i as f64) as i32, (y1 as f64 + dy * i as f64) as i32, color);
            }
        }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Option<Color> {
        if self.rect.contains_point(x, y) {
            let color = self.data.get_pixel(x as u32, y as u32);
            return Some(Color::new(color[0], color[1], color[2], color[3]))
        }
        None
    }

    pub fn fill(&mut self, x: i32, y: i32, color: Color) {
        if let Some(target_color) = self.get_pixel(x, y) {
            if target_color == color {
                return;
            }
            self.draw_pixel(x, y, color);
            let mut queue = VecDeque::new();
            queue.push_back((x, y));
            while !queue.is_empty() {
                if let Some((x, y)) = queue.pop_front() {
                    if let Some(old_color) = self.get_pixel(x - 1, y) {
                        if old_color == target_color {
                            self.draw_pixel(x - 1, y, color);
                            queue.push_back((x - 1, y));
                        }
                    }
                    if let Some(old_color) = self.get_pixel(x + 1, y) {
                        if old_color == target_color {
                            self.draw_pixel(x + 1, y, color);
                            queue.push_back((x + 1, y));
                        }
                    }
                    if let Some(old_color) = self.get_pixel(x, y - 1) {
                        if old_color == target_color {
                            self.draw_pixel(x, y - 1, color);
                            queue.push_back((x, y - 1));
                        }
                    }
                    if let Some(old_color) = self.get_pixel(x, y + 1) {
                        if old_color == target_color {
                            self.draw_pixel(x, y + 1, color);
                            queue.push_back((x, y + 1));
                        }
                    }
                }
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), ()> {
        match self.data.save(path) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
