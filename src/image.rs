use opencv::{core::Size_, prelude::*, Result};
use tui::style::Color;
use tui::widgets::canvas::Shape;

use crate::opencv_wrapper;
use crate::{HEIGHT, WIDTH};

pub struct SharedImage {
    image: Image,
}

impl SharedImage {
    pub fn default(image_height: i32, image_width: i32, image_map: Vec<(f64, f64)>) -> SharedImage {
        let image = Image::default(image_height, image_width, image_map);

        SharedImage { image }
    }
    pub fn update(&mut self, color: Vec<u8>) {
        self.image.update(color);
    }

    pub fn image(&self) -> &Image {
        &self.image
    }
}

pub struct Image {
    color: Vec<u8>,
    size: Size_<i32>,
    image_map: Vec<(f64, f64)>,
}

impl Image {
    pub fn default(image_height: i32, image_width: i32, image_map: Vec<(f64, f64)>) -> Image {
        let pixels: usize = image_height as usize * image_width as usize * 3;
        let mut v = Vec::new();
        for i in 0..pixels {
            v.push(i as u8);
        }
        let size = Size_::new(image_width, image_height);

        Image {
            color: v,
            size,
            image_map,
        }
    }

    pub fn new_from_vec(
        v: Vec<u8>,
        image_map: Vec<(f64, f64)>,
        image_height: i32,
        image_width: i32,
    ) -> Result<Image> {
        let size = Size_::new(image_width, image_height);

        let image = Image {
            color: v,
            size,
            image_map: image_map.clone(),
        };
        Ok(image)
    }

    pub fn new_from_cam(
        cam: &mut impl VideoCaptureTrait,
        image_map: Vec<(f64, f64)>,
        image_height: i32,
        image_width: i32,
    ) -> Result<Image> {
        let mut frame = Mat::default();

        cam.read(&mut frame)?;

        let compressed_frame =
            opencv_wrapper::get_compressed_frame(&frame, image_height, image_width)?;
        let v = compressed_frame.data_bytes()?.to_vec();

        let size = compressed_frame.size()?;

        let image = Image {
            color: v,
            size,
            image_map: image_map.clone(),
        };
        Ok(image)
    }

    pub fn new_from_bytes(bytes: Vec<u8>, image_map: Vec<(f64, f64)>) -> Result<Image> {
        todo!()
    }

    pub fn color(&self) -> &Vec<u8> {
        &self.color
    }

    pub fn image_map(&self) -> &Vec<(f64, f64)> {
        &self.image_map
    }

    pub fn size(&self) -> &Size_<i32> {
        &self.size
    }

    pub fn update(&mut self, color: Vec<u8>) {
        self.color = color;
    }
}

impl Shape for Image {
    fn draw(&self, painter: &mut tui::widgets::canvas::Painter) {
        for (x, y) in self.image_map.iter() {
            let width = self.size.width;

            let pixel = (3.0 * (x + width as f64 * y)) as usize;

            let left = -1.0 * (WIDTH / 2.0);
            let top = HEIGHT / 2.0;

            let color = Color::Rgb(
                self.color[pixel + 2],
                self.color[pixel + 1],
                self.color[pixel],
            );
            if let Some((x, y)) = painter.get_point(*x + left, -*y + top) {
                painter.paint(x, y, color)
            }
        }
    }
}
