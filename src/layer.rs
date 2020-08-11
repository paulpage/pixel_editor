use image::{ImageBuffer, RgbaImage};
use super::util::Rect;


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
}
