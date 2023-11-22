#![allow(dead_code)]

use image::RgbaImage;
use image::error::ImageError;
use std::path::Path;
use std::collections::VecDeque;
use std::cmp::{min, max};

use super::app::{self, Color};

#[derive(Copy, Clone, Debug)]
pub struct ImageRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl ImageRect {
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self {
            x,
            y,
            w,
            h,
        }
    }

    pub fn union(&self, other: ImageRect) -> ImageRect {
        if self.w == 0 || self.h == 0 {
            return other;
        }
        if other.w == 0 || other.h == 0 {
            return *self;
        }
        let r = ImageRect {
            x: min(self.x, other.x),
            y: min(self.y, other.y),
            w: (max(self.x + self.w as i32, other.x + other.w as i32) - min(self.x, other.x)) as u32,
            h: (max(self.y + self.h as i32, other.y + other.h as i32) - min(self.y, other.y)) as u32,
        };
        return r;
    }

    pub fn has_intersection(&self, other: ImageRect) -> bool {
        self.x <= other.x + other.w as i32 && self.x + self.w as i32 >= other.x
         && self.y <= other.y + other.h as i32 && self.y + self.h as i32 >= other.y
    }

    pub fn intersection(&self, other: ImageRect) -> ImageRect {
        if !ImageRect::has_intersection(self, other) {
            return ImageRect::new(0, 0, 0, 0);
        }
        ImageRect {
            x: max(self.x, other.x),
            y: max(self.y, other.y),
            w: (min(self.x + self.w as i32, other.x + other.w as i32) - max(self.x, other.x)) as u32,
            h: (min(self.y + self.h as i32, other.y + other.h as i32) - max(self.y, other.y)) as u32,
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w as i32 && y >= self.y && y < self.y + self.h as i32
    }
}

pub struct Layer {
    pub rect: ImageRect,
    pub data: Vec<Color>,
    pub z_index: i32,
    pub dirty_rect: ImageRect,
}

pub struct Image {
    pub rect: ImageRect,
    pub layers: Vec<Layer>,
}

pub struct ImageHistory {
    snapshots: Vec<Image>,
    idx: i32,
}

impl Layer {
    pub fn new(rect: ImageRect) -> Self {
        let color = app::WHITE;
        let data = vec![color; (rect.w * rect.h) as usize];
        Self {
            rect,
            data,
            z_index: 0,
            dirty_rect: ImageRect::new(0, 0, 0, 0),
        }
    }

    pub fn from_path(x: i32, y: i32, path: &str) -> Result<Self, ImageError> {
        let image = image::open(path)?.to_rgba8();
        let rect = ImageRect::new(x, y, image.width(), image.height());

        let color = app::WHITE;
        let mut data = vec![color; (rect.w * rect.h) as usize];

        for y in 0..rect.h {
            for x in 0..rect.w {
                let c = image.get_pixel(x as u32, y as u32);
                data[(y * rect.w + x) as usize] = Color::new(c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, c[3] as f32 / 255.0);
            }
        }

        Ok(Self {
            rect,
            data,
            z_index: 0,
            dirty_rect: ImageRect::new(0, 0, 0, 0),
        })
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.rect.w as i32 && y >= 0 && y < self.rect.h as i32
    }

    pub fn draw_pixel(&mut self, x: i32, y: i32, color: Color) {
        if self.contains_point(x, y) {
            self.data[y as usize * self.rect.w as usize + x as usize] = color;
        }
    }

    pub fn draw_pixel_unchecked(&mut self, x: i32, y: i32, color: Color) {
        self.data[y as usize * self.rect.w as usize + x as usize] = color;
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
        self.draw_pixel(x1, y1, color);
        self.draw_pixel(x2, y2, color);
        let w = (x2 - x1).abs();
        let h = (y2 - y1).abs();
        let step = std::cmp::max(w, h);
        if step != 0 {
            let dx = (x2 as f64 - x1 as f64) / step as f64;
            let dy = (y2 as f64 - y1 as f64) / step as f64;
            for i in 0..step {
                self.draw_pixel((x1 as f64 + dx * i as f64) as i32, (y1 as f64 + dy * i as f64) as i32, color);
            }
        }
        self.add_dirty_rect(ImageRect {
            x: min(x1, x2) - 1,
            y: min(y1, y2) - 1,
            w: w as u32 + 2,
            h: h as u32 + 2,
        });
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Option<Color> {
        if self.contains_point(x, y) {
            return Some(self.data[y as usize * self.rect.w as usize + x as usize]);
        }
        None
    }

    pub fn get_pixel_unchecked(&self, x: i32, y: i32) -> Color {
        self.data[y as usize * self.rect.w as usize + x as usize]
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
        self.add_dirty_rect(self.rect);
    }

    pub fn blend(&mut self, other: &Layer, clip_rect: ImageRect) -> bool {

        let target_rect = self.rect.intersection(other.rect).intersection(self.dirty_rect.union(other.dirty_rect)).intersection(clip_rect);

        if target_rect.w == 0 || target_rect.h == 0 {
            return false;
        }

        for y in target_rect.y..target_rect.y + target_rect.h as i32 {
            for x in target_rect.x..target_rect.x + target_rect.w as i32 {
                if !(self.rect.contains(x, y) && other.rect.contains(x - other.rect.x, y - other.rect.y)) {
                }
                let base_color = self.get_pixel_unchecked(x, y);
                let other_color = other.get_pixel_unchecked(x - other.rect.x, y - other.rect.y);

                if other_color.a == 0.0 {
                    continue;
                }
                if other_color.a == 1.0 {
                    self.draw_pixel(x, y, other_color);
                    continue;
                }

                let a1 = other_color.a;
                let a2 = base_color.a;
                let factor = a2 * (1.0 - a1);

                let new_color = Color {
                    r: (base_color.r * a1 + other_color.r * factor / (a1 + factor)),
                    g: (base_color.r * a1 + other_color.r * factor / (a1 + factor)),
                    b: (base_color.r * a1 + other_color.r * factor / (a1 + factor)),
                    a: (base_color.r * a1 + other_color.r * factor / (a1 + factor)),
                };
                self.draw_pixel(x, y, new_color);
            }
        }

        return true;
    }

    pub fn add_dirty_rect(&mut self, rect: ImageRect) {
        self.dirty_rect = self.dirty_rect.union(rect);
    }

    pub fn clear_dirty_rect(&mut self) {
        self.dirty_rect = ImageRect::new(0, 0, 0, 0);
    }
}

impl Image {
    pub fn new(w: u32, h: u32) -> Self {
        let mut layers = Vec::new();
        let rect = ImageRect::new(0, 0, w, h);
        layers.push(Layer::new(rect));
        Self {
            rect,
            layers,
        }
    }

    pub fn from_path(path: &str) -> Result<Self, ImageError> {
        let mut layers = Vec::new();
        layers.push(Layer::from_path(0, 0, path)?);
        Ok(Self {
            rect: ImageRect::new(
                0,
                0,
                layers[0].rect.w,
                layers[0].rect.h,
            ),
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

    pub fn blend(&self, clip_rect: ImageRect) -> Layer {
        let mut base = Layer::new(self.rect);
        for layer in &self.layers {
            base.blend(layer, clip_rect);
        }
        base
    }

    pub fn raw_data(&self) -> Vec<u8> {

        let blended = self.blend(self.rect);

        let mut raw_data = vec![0; blended.rect.w as usize * blended.rect.h as usize * 4];
        for y in 0..blended.rect.h {
            for x in 0..blended.rect.w {
                let p = y as usize * blended.rect.w as usize + x as usize;
                let color = blended.data[p];
                raw_data[p * 4 + 0] = (color.r * 255.0) as u8;
                raw_data[p * 4 + 1] = (color.g * 255.0) as u8;
                raw_data[p * 4 + 2] = (color.b * 255.0) as u8;
                raw_data[p * 4 + 3] = (color.a * 255.0) as u8;
            }
        }
        return raw_data;
    }

    pub fn partial_data(&self, rect: ImageRect) -> Vec<u8> {
        let blended = self.blend(rect);
        let mut raw_data = vec![0; rect.w as usize * rect.h as usize * 4];
        for y in rect.y..rect.y+rect.h as i32 {
            for x in rect.x..rect.x+rect.w as i32 {
                let si = (y - rect.y) as usize * rect.w as usize + (x - rect.x) as usize;
                let di = y as usize * blended.rect.w as usize + x as usize;
                let color = blended.data[di];
                raw_data[si * 4 + 0] = (color.r * 255.0) as u8;
                raw_data[si * 4 + 1] = (color.g * 255.0) as u8;
                raw_data[si * 4 + 2] = (color.b * 255.0) as u8;
                raw_data[si * 4 + 3] = (color.a * 255.0) as u8;

            }
        }
        return raw_data;
    }

    pub fn dirty_rect(&self) -> ImageRect {
        let mut dirty_rect = ImageRect::new(0, 0, 0, 0);
        for layer in &self.layers {
            dirty_rect = dirty_rect.union(layer.dirty_rect);
        }
        dirty_rect
    }

    pub fn clear_dirty(&mut self) {
        for layer in &mut self.layers {
            layer.clear_dirty_rect();
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), ()> {

        let blended = self.blend(self.rect);
        let mut image = RgbaImage::from_pixel(blended.rect.w, blended.rect.h, [255, 255, 255, 255].into());
        for y in 0..blended.rect.h {
            for x in 0..blended.rect.w {
                let color = blended.get_pixel(x as i32, y as i32).unwrap();
                image.put_pixel(x as u32, y as u32, [
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                    (color.a * 255.0) as u8,
                ].into());
            }
        }

        match image.save(path) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
