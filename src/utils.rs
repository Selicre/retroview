
use image::{GenericImage, GenericImageView, imageops};

pub fn overlay<I: GenericImage, J: GenericImageView<Pixel = I::Pixel>>(target: &mut I, src: &J, x: i32, y: i32)
        where I::Pixel: 'static, J::InnerImageView: GenericImage + 'static {
    if x < -(src.width() as i32) || y < -(src.height() as i32) { return; }
    if x > (target.width() as i32) || y > (target.height() as i32) { return; }
    if x < 0 || y < 0 {
        let cuts = ((-x).max(0) as u32,(-y).max(0) as u32);
        let (w,h) = (src.width(), src.height());
        let cr = src.view(cuts.0, cuts.1, w-cuts.0,h-cuts.1).to_image();
        imageops::overlay(target, &cr, x.max(0) as u32, y.max(0) as u32);
    } else {
        if x < target.width() as i32 && y < target.height() as i32 {
            imageops::overlay(target, src, x as u32, y as u32);
        }
    }
}
