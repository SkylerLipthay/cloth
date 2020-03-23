extern crate cloth;
extern crate image;

use cloth::Cloth;
use image::{RgbaImage, Rgba};

fn main() {
    let mut cloth = Cloth::new(Bridge(RgbaImage::new(250, 310)));

    cloth.set_fill([255, 0, 0, 127]);
    cloth.begin_path();
    cloth.move_to(0.0, 0.0);
    cloth.line_to(200.0, 200.0);
    cloth.line_to(200.0, 0.0);
    cloth.line_to(0.0, 200.0);
    cloth.close_path();
    cloth.fill();

    cloth.set_fill([0, 255, 0, 127]);
    cloth.begin_path();
    cloth.move_to(50.0, 0.0);
    cloth.line_to(250.0, 200.0);
    cloth.line_to(250.0, 0.0);
    cloth.line_to(50.0, 200.0);
    cloth.close_path();
    cloth.fill();

    cloth.set_fill([0, 0, 0, 255]);
    cloth.begin_path();
    cloth.move_to(0.0, 260.5);
    cloth.line_to(50.0, 260.5);
    cloth.line_to(50.0, 310.0);
    cloth.line_to(0.0, 310.0);
    cloth.close_path();
    cloth.fill();

    let image = cloth.into_target().0;
    image.save("examples/basic.png").unwrap();
}

struct Bridge(RgbaImage);

impl cloth::Target for Bridge {
    fn width(&self) -> u32 {
        self.0.width()
    }

    fn height(&self) -> u32 {
        self.0.height()
    }

    fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.0.get_pixel(x, y).0
    }

    fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        self.0.put_pixel(x, y, Rgba(rgba));
    }
}
