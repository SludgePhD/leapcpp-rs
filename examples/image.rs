//! This example acquires raw image frames and displays them in a window.

use leapfrog::{ManagedController, Policy};
use macroquad::prelude::*;

#[macroquad::main("LeapImage")]
async fn main() {
    let controller = ManagedController::new();

    println!("waiting for controller to connect");
    controller.wait_until_device_connected();
    controller.set_policy(Policy::Images);
    println!("starting");

    let mut image = Image::empty();
    let mut imagesize = (0, 0);
    loop {
        clear_background(BLACK);

        let images = controller.images();
        if let Some(leap_image) = images.iter().next() {
            let (w, h) = (leap_image.width() as u16, leap_image.height() as u16);
            if imagesize != (w, h) {
                imagesize = (w, h);
                image = Image::gen_image_color(w, h, RED);
            }

            for y in 0..leap_image.height() {
                for x in 0..leap_image.width() {
                    let val = &leap_image.data().pixel(x, y);
                    image.set_pixel(x as u32, y as u32, Color::from_rgba(*val, *val, *val, 255));
                }
            }
        }

        let tex = Texture2D::from_image(&image);
        draw_texture(tex, 0.0, 0.0, WHITE);

        next_frame().await;
    }
}
