#![allow(dead_code)]

use image::RgbaImage;
use image::error::ImageError;
use super::util::{Rect, Color};
use std::path::Path;
use std::collections::VecDeque;
use std::cmp::{min, max};

pub struct Layer {
    pub rect: Rect,
    pub data: Vec<Color>,
    pub z_index: i32,
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
}

pub struct ImageHistory {
    snapshots: Vec<Image>,
    idx: i32,
}

impl Layer {
    pub fn new(rect: Rect) -> Self {
        let color = Color::WHITE;
        let data = vec![color; (rect.width * rect.height) as usize];
        Self {
            rect,
            data,
            z_index: 0,
        }
    }

    pub fn from_path(x: i32, y: i32, path: &str) -> Result<Self, ImageError> {
        let image = image::open(path)?.to_rgba();
        let rect = Rect::new(x, y, image.width(), image.height());

        let color = Color::WHITE;
        let mut data = vec![color; (rect.width * rect.height) as usize];

        for y in 0..rect.height {
            for x in 0..rect.width {
                let c = image.get_pixel(x as u32, y as u32);
                data[(y * rect.width + x) as usize] = Color::new(c[0], c[1], c[2], c[3]);
            }
        }

        Ok(Self {
            rect,
            data,
            z_index: 0,
        })
    }

    pub fn draw_pixel(&mut self, x: i32, y: i32, color: Color) {
        if self.rect.contains_point(x, y) {
            self.data[(y as usize * self.rect.width as usize + x as usize)] = color;
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
            return Some(self.data[y as usize * self.rect.width as usize + x as usize]);
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

    pub fn blend(&mut self, other: &Layer) -> bool {
        let width = min(other.rect.width as i32, self.rect.width as i32 - other.rect.x);
        let height = min(other.rect.height as i32, self.rect.height as i32 - other.rect.y);
        if self.rect.width >= other.rect.width && self.rect.height >= other.rect.height {
            for y in max(0, other.rect.y)..other.rect.y + height {
                for x in max(0, other.rect.x)..min(self.rect.width as i32, other.rect.x + width) {

                    let base_color = self.get_pixel(x, y).unwrap();
                    let other_color = other.get_pixel(x - other.rect.x, y - other.rect.y).unwrap();

                    if other_color.a == 0 {
                        continue;
                    }
                    if other_color.a == 255 {
                        self.draw_pixel(x, y, other_color);
                        continue;
                    }

                    let a1 = other_color.a as f64 / 255.0;
                    let a2 = base_color.a as f64 / 255.0;
                    let factor = a2 * (1.0 - a1);

                    
                    let new_color = Color {
                        r: (base_color.r as f64 * a1 + other_color.r as f64 * factor / (a1 + factor)) as u8,
                        g: (base_color.r as f64 * a1 + other_color.r as f64 * factor / (a1 + factor)) as u8,
                        b: (base_color.r as f64 * a1 + other_color.r as f64 * factor / (a1 + factor)) as u8,
                        a: (base_color.r as f64 * a1 + other_color.r as f64 * factor / (a1 + factor)) as u8,
                    };
                    self.draw_pixel(x, y, new_color);
                }
            }
            return true;
        }
        false
    }
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        let mut layers = Vec::new();
        layers.push(Layer::new(Rect::new(0, 0, width, height)));
        Self {
            width,
            height,
            layers,
        }
    }

    pub fn from_path(path: &str) -> Result<Self, ImageError> {
        let mut layers = Vec::new();
        layers.push(Layer::from_path(0, 0, path)?);
        Ok(Self {
            width: layers[0].rect.width,
            height: layers[0].rect.height,
            layers,
        })
    }

    // TODO probably don't even need this since we have clone, but will have to implmement clone
    // pub fn copy(other: &Self) -> Image {
    //     other.cloned()
    // }

    // TODO do I need this?
    pub fn add_layer(&mut self) {
        // TODO
    }

    // TODO do I need this?
    pub fn remove_layer(&mut self) {
        // TODO
    }

    pub fn take_snapshot(&self, history: &mut ImageHistory) {
        // TODO
    }

    pub fn undo(&mut self, history: &mut ImageHistory) {
        // TODO
    }

    pub fn redo(&mut self, history: &mut ImageHistory) {
        // TODO
    }

    pub fn blend(&self) -> Layer {
        let mut base = Layer::new(Rect::new(0, 0, self.width, self.height));
        for layer in &self.layers {
            base.blend(layer);
        }
        base
    }

    pub fn raw_data(&self) -> Vec<u8> {

        let blended = self.blend();

        let mut raw_data = vec![0; blended.rect.width as usize * blended.rect.height as usize * 4];
        for y in 0..blended.rect.height {
            for x in 0..blended.rect.width {
                let p = y as usize * blended.rect.width as usize + x as usize;
                let color = blended.data[p];
                raw_data[p * 4 + 0] = color.r;
                raw_data[p * 4 + 1] = color.g;
                raw_data[p * 4 + 2] = color.b;
                raw_data[p * 4 + 3] = color.a;
            }
        }
        return raw_data;
    }

    pub fn save(&self, path: &Path) -> Result<(), ()> {

        let blended = self.blend();
        let mut image = RgbaImage::from_pixel(blended.rect.width, blended.rect.height, [255, 255, 255, 255].into());
        for y in 0..blended.rect.height {
            for x in 0..blended.rect.width {
                let color = blended.get_pixel(x as i32, y as i32).unwrap();
                image.put_pixel(x as u32, y as u32, [color.r, color.g, color.b, color.a].into());
            }
        }

        match image.save(path) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
