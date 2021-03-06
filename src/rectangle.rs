use crate::shape::{Shape, RandomShape};
use crate::point::PrimitivePoint;
use crate::primitive_image::PrimitiveImage;
use image::Rgba;
use std::cmp::max;
use rand;
use rand::Rng;
use image::ImageBuffer;
use imageproc::drawing::draw_filled_rect;
use imageproc::rect::Rect;
use imageproc::affine::rotate;
use imageproc::affine::Interpolation::Nearest;
use image::imageops::overlay;
use crate::utilities::{get_rng, clamp, radians, rgb_to_hex, rotate_point};
use rand::distributions::Distribution;
use rand::distributions::Normal;

const MAXIMUM_MUTATION_ATTEMPTS: u32 = 100_000;

#[derive(Debug, Copy, Clone)]
pub struct Rectangle {
    pub color: image::Rgba<u8>,
    center: PrimitivePoint,
    width: u32,
    height: u32,
    angle: u32 // In degrees
}

impl Rectangle {

    ///
    /// Determine if this rectangle is valid
    ///
    fn is_valid(&self) -> bool {
        true
    }
}

impl RandomShape for Rectangle {

    ///
    /// Generate a random Triangle within the bounds given
    /// `border_extension` is the maximum distance outside of the border a triangle is allowed to go
    ///     It must be >= 1
    ///
    fn random(width: u32, height: u32, _border_extension: i32, seed: u64) -> Box<Shape> {
        let mut rng = get_rng(seed);

        let center = PrimitivePoint::random_point(width, height, seed);
        let width = rng.gen_range(5, max(width, height) / 2);
        let height = rng.gen_range(5, max(width, height) / 2);
        let angle = rng.gen_range(0, 180);

        let mut rect = Rectangle{center, width, height, angle, color: Rgba([0, 0, 0, 128])};
        rect.mutate(width, height, seed);

        Box::new(rect)
    }
}

impl Shape for Rectangle {

    fn mutate(&mut self, width: u32, height: u32, seed: u64) {
        let mut rng = get_rng(seed);
        let normal = Normal::new(0.0, 16.0);


        let mut i = 0;
        loop {
            i += 1;
            let r = rng.gen_range(0, 4);

            match r {
                0 => self.center.mutate(width, height, seed),
                1 => self.width = clamp(self.width as i32 + (normal.sample(&mut rng) as i32), 5, max(width, height) as i32) as u32,
                2 => self.height = clamp(self.height as i32 + (normal.sample(&mut rng) as i32), 5, max(width, height) as i32) as u32,
                3 => self.angle = rng.gen_range(0, 180),
                _ => {}
            }

            if self.is_valid() {
                break;
            }
            if i > MAXIMUM_MUTATION_ATTEMPTS {
                panic!("Rectangle: Too many mutation loops!");
            }
        }
    }

    fn get_pixels(&self) -> Vec<PrimitivePoint> {
        let min_x = self.center.x - (self.width as i32 / 2);
        let min_y = self.center.y - (self.height as i32 / 2);
        let max_x = self.center.x + (self.width as i32 / 2);
        let max_y = self.center.y + (self.height as i32 / 2);

        let mut pixels = vec![];

        for x in min_x..max_x {
            for y in min_y..max_y {
                pixels.push(PrimitivePoint::new(x, y));
            }
        }

        for i in 0..pixels.len() {
            rotate_point(&mut pixels[i], self.center, self.angle);
        }

        pixels
    }

    fn as_svg(&self, scale: f64) -> String {
        let new_center = PrimitivePoint::new((self.center.x as f64 * scale) as i32, (self.center.y as f64 * scale) as i32);

        let min_x = new_center.x - ((self.width as f64 * scale) as i32 / 2);
        let min_y = new_center.y - ((self.height as f64 * scale) as i32 / 2);

        let p1 = PrimitivePoint::new(min_x, min_y);

        format!("<rect fill=\"{}\" fill-opacity=\"{:.5}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" transform=\"rotate({} {} {})\"/>",
                rgb_to_hex(self.color),
                self.color.data[3] as f64 / 255.0,
                p1.x, p1.y,
                self.width as f64 * scale, self.height as f64 * scale,
                self.angle, p1.x as f64 + self.width as f64 * scale / 2.0, p1.y as f64 + self.height as f64 * scale / 2.0)
    }

    // Suppress intellij inspection for E0308 (false positive)
    //noinspection RsTypeCheck
    fn paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut rect_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        rect_image = draw_filled_rect(&rect_image, Rect::at(self.center.x, self.center.y).of_size(self.width, self.height), self.color);
        rect_image = rotate(&rect_image, (self.center.x as f32, self.center.y as f32), -1.0*radians(self.angle as f64) as f32, Nearest);

        overlay(&mut output, &rect_image, 0, 0);

        output
    }

    // Suppress intellij inspection for E0308 (false positive)
    //noinspection RsTypeCheck
    fn scaled_paint_on(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>, scale: f64) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (width, height) = image.dimensions();

        let mut rect_image: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_pixel(width as u32, height as u32, image::Rgba([0, 0, 0, 0]));
        let mut output = image.clone();

        rect_image = draw_filled_rect(&rect_image, Rect::at((self.center.x as f64 * scale) as i32, (self.center.y as f64 * scale) as i32).of_size(((self.width as f64) * scale) as u32, ((self.height as f64) * scale) as u32), self.color);
        rect_image = rotate(&rect_image, ((self.center.x as f64 * scale) as f32, (self.center.y as f64 * scale) as f32), -1.0*radians(self.angle as f64) as f32, Nearest);

        overlay(&mut output, &rect_image, 0, 0);

        output
    }

    fn set_color_using(&mut self, image: &PrimitiveImage) {
        self.color = image.target_average_color_in_shape(&Box::new(*self));
    }
}