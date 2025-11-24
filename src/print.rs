use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, Resizer};
use fltk::draw::draw_image;
use fltk::enums::ColorDepth;
use fltk::prelude::*;
use fltk::window::Window;
use fltk::{app, printer};

pub fn print_document(images: &Vec<Vec<u8>>, img_width: u32, img_height: u32) {
    let app = app::App::default();
    let mut wind = Window::default();
    wind.end();
    wind.show();
    wind.hide();

    let mut printer = printer::Printer::default();
    printer::Printer::set_dialog_title("Hello, world!");
    if printer.begin_job(images.len() as i32).is_ok() {
        for image in images {
            printer.begin_page().ok();
            // Configure printer
            printer.scale(0.3, 0.3);

            let (width, height) = printer.printable_rect();

            // Downscale image
            // fltk's API wasn't working, so I'll use fast_image_resize
            let src_image =
                Image::from_vec_u8(img_width, img_height, image.to_vec(), PixelType::U8x4)
                    .expect("Failed to parse initial buffer");
            let mut dst_image = Image::new(width as u32, height as u32, PixelType::U8x4);
            let mut resizer = Resizer::new();
            resizer.resize(&src_image, &mut dst_image, None).unwrap();

            draw_image(
                dst_image.buffer(),
                0,
                0,
                dst_image.width() as i32,
                dst_image.height() as i32,
                ColorDepth::Rgba8,
            )
            .expect("PDF print rendering failed");

            printer.set_origin(width / 2, height / 2);
            printer.end_page().ok();
        }

        printer.end_job();
    }

    app.run().unwrap();
}
