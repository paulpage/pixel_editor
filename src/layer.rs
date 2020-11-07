#![allow(dead_code)]

use image::{self, RgbaImage};
use image::error::ImageError;
use super::util::{Rect, Color};
use std::path::Path;
use std::collections::VecDeque;
use std::cmp::{min, max};

#[derive(Clone)]
pub struct Layer {
    pub rect: Rect,
    pub data: Vec<u8>,
    pub z_index: i32,
    pub name: String,
}

#[derive(Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
}

pub struct ImageHistory {
    idx: i64,
    snapshots: Vec<Image>,
}

impl ImageHistory {
    pub fn new() -> Self {
        Self {
            idx: -1,
            snapshots: Vec::new(),
        }
    }

    pub fn take_snapshot(&mut self, image: &Image) {
        self.snapshots.truncate((self.idx + 1) as usize);
        self.snapshots.push(image.clone());
        self.idx += 1;
    }

    pub fn undo(&mut self, image: Image) -> Image {
        if self.idx > 0 {
            self.idx -= 1;
            return self.snapshots[self.idx as usize].clone();
        }
        image
    }

    pub fn redo(&mut self, image: Image) -> Image {
        if self.idx as usize + 1 < self.snapshots.len() {
            self.idx += 1;
            return self.snapshots[self.idx as usize].clone();
        }
        image
    }
}

impl Layer {
    pub fn new(rect: Rect) -> Self {
        let data = vec![0; (rect.width * rect.height * 4) as usize];
        // let data = RgbaImage::from_pixel(rect.width, rect.height, [255, 255, 255, 255].into());
        Self {
            rect,
            data,
            z_index: 0,
            name: "Unnamed Layer".into(),
        }
    }

    pub fn from_path(x: i32, y: i32, path: &str) -> Result<Self, ImageError> {
        let img = image::open(path)?.to_rgba();
        Ok(Self {
            rect: Rect::new(x, y, img.width(), img.height()),
            data: img.into_raw(),
            z_index: 0,
            name: "Unnamed Layer".into(),
        })
    }

    pub fn draw_pixel(&mut self, x: i32, y: i32, color: Color) {
        if self.rect.contains_point(x, y) {
            let i = (y * self.rect.width as i32 + x) as usize * 4;
            self.data[i] = color.r;
            self.data[i + 1] = color.g;
            self.data[i + 2] = color.b;
            self.data[i + 3] = color.a;
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
            let i = (y * self.rect.width as i32 + x) as usize * 4;
            let c = Color::new(
                self.data[i],
                self.data[i + 1],
                self.data[i + 2],
                self.data[i + 3]
            );
            return Some(c)
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

                    // The cases where alpha is 0 or 255 are the most common, so make those fast
                    // let other_color = other.data.get_pixel((x - other.rect.x) as u32, (y - other.rect.y) as u32);
                    let oi = ((y - other.rect.y) * other.rect.width as i32 + (x - other.rect.x)) as usize * 4;
                    if other.data[oi + 3] == 0 {
                        continue;
                    }
                    if other.data[oi + 3] == 255 {
                        let i = (y * self.rect.width as i32 + x) as usize * 4;
                        self.data[i] = other.data[oi];
                        self.data[i + 1] = other.data[oi + 1];
                        self.data[i + 2] = other.data[oi + 2];
                        self.data[i + 3] = other.data[oi + 3];
                        continue;
                    }

                    let base_color = self.get_pixel(x, y).unwrap();
                    let other_color = other.get_pixel(x - other.rect.x, y - other.rect.y).unwrap();

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

    pub fn clear(&mut self) {
        self.data.iter_mut().map(|i| *i = 0).count();
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

    pub fn blend(&self, rect: Rect) -> Layer {
        let mut base = Layer::new(Rect::new(0, 0, self.width, self.height));
        for layer in self.layers.iter().rev() {
            let width = min(layer.rect.width as i32, base.rect.width as i32 - layer.rect.x);
            let height = min(layer.rect.height as i32, base.rect.height as i32 - layer.rect.y);
            if base.rect.width >= layer.rect.width && base.rect.height >= layer.rect.height {
                for y in max(rect.y, layer.rect.y)..layer.rect.y + height {
                    for x in max(rect.x, layer.rect.x)..min(base.rect.width as i32, layer.rect.x + width) {

                        let i = (y * base.rect.width as i32 + x) as usize * 4;
                        if base.data[i + 3] == 255 {
                            continue;
                        }

                        // The cases where alpha is 0 or 255 are the most common, so make those fast
                        let oi = ((y - layer.rect.y) * layer.rect.width as i32 + (x - layer.rect.x)) as usize * 4;
                        if layer.data[oi + 3] == 0 {
                            continue;
                        }
                        if layer.data[oi + 3] == 255 {
                            base.data[i] = layer.data[oi];
                            base.data[i + 1] = layer.data[oi + 1];
                            base.data[i + 2] = layer.data[oi + 2];
                            base.data[i + 3] = layer.data[oi + 3];
                            continue;
                        }


                        // TODO all this logic has to get inverted since we're going
                        // backwards

                        let base_color = base.get_pixel(x, y).unwrap();
                        let layer_color = layer.get_pixel(x - layer.rect.x, y - layer.rect.y).unwrap();

                        let a1 = layer_color.a as f64 / 255.0;
                        let a2 = base_color.a as f64 / 255.0;
                        let factor = a2 * (1.0 - a1);

                        let new_color = Color {
                            r: (base_color.r as f64 * a1 + layer_color.r as f64 * factor / (a1 + factor)) as u8,
                            g: (base_color.r as f64 * a1 + layer_color.r as f64 * factor / (a1 + factor)) as u8,
                            b: (base_color.r as f64 * a1 + layer_color.r as f64 * factor / (a1 + factor)) as u8,
                            a: (base_color.r as f64 * a1 + layer_color.r as f64 * factor / (a1 + factor)) as u8,
                        };
                        base.draw_pixel(x, y, new_color);
                    }
                }
            }
        }
        base
    }

    pub fn save(&self, path: &Path) -> Result<(), ()> {
        let blended = self.blend(Rect::new(0, 0, self.width, self.height));
        // TODO
        let img = RgbaImage::from_raw(blended.rect.width, blended.rect.height, blended.data).unwrap();
        match img.save(path) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
