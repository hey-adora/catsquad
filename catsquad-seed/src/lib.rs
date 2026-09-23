use std::path::Path;

use ab_glyph::{FontRef, PxScale};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text;
use rand::Rng;

pub fn create_img(save_path: impl AsRef<Path>, img_text: impl AsRef<str>) {
    let img_text = img_text.as_ref();
    let path = save_path.as_ref();
    let mut rng = rand::rng();
    let mut image = RgbImage::new(200, 200);
    let r = rng.random_range(200u8..255);
    let g = rng.random_range(200u8..255);
    let b = rng.random_range(200u8..255);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        *pixel = image::Rgb([r, g, b]);
    }
    let height = 64.0;
    let scale = PxScale {
        x: height * 2.0,
        y: height,
    };
    let font = FontRef::try_from_slice(include_bytes!("../../assets/noto_sans.ttf")).unwrap();
    let img = draw_text(
        &mut image,
        Rgb([0u8, 0u8, 0u8]),
        0,
        50,
        scale,
        &font,
        img_text,
    );
    img.save(path).unwrap();
}
