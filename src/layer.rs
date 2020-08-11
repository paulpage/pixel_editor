use image::{ImageBuffer, RgbaImage};
use super::util::{Rect, Color};


pub struct Layer {
    pub rect: Rect,
    pub data: image::RgbaImage,
    pub z_index: i32,
}

impl Layer {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            data: RgbaImage::new(rect.width, rect.height),
            z_index: 0,
        }
    }

    pub fn from_path(x: i32, y: i32, path: &str) -> Self {
        let img = image::open(path).unwrap().to_rgba();
        Self {
            rect: Rect::new(x, y, img.width(), img.height()),
            data: img,
            z_index: 0,
        }
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        self.data.put_pixel(x, y, [color.r, color.g, color.b, color.a].into());
    }

    pub fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: Color) {
        self.draw_pixel(x1, y1, color);
        self.draw_pixel(x2, y2, color);
        let width = (x2 as i32 - x1 as i32).abs();
        let height = (y2 as i32 - y1 as i32).abs();
        let step = std::cmp::max(width, height);
        if step != 0 {
            let dx = (x2 as f64 - x1 as f64) / step as f64;
            let dy = (y2 as f64 - y1 as f64) / step as f64;
            for i in 0..step {
                self.draw_pixel((x1 as f64 + dx * i as f64) as u32, (y1 as f64 + dy * i as f64) as u32, color);
            }
        }
    }
}
