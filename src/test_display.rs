use ssd1306::prelude::*;
use ssd1306::interface::i2c::I2cInterface;
use linux_embedded_hal::I2cdev;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;

pub fn test_display(disp: &mut GraphicsMode<I2cInterface<I2cdev>>) {
    disp.draw(
        Font6x8::render_str(&"Selamat Idul Fitri".to_string()).into_iter()
    );

    disp.draw(
        Font6x8::render_str(&"1441H, mohon maaf".to_string())
            .translate(Coord::new(0, 10))
            .into_iter()
    );

    disp.draw(
        Font6x8::render_str(&"lahir dan batin!".to_string())
            .translate(Coord::new(0, 20))
            .into_iter()
    );

    disp.draw(
        Font6x8::render_str(&"- MADE IN RUST LANG -".to_string())
            .translate(Coord::new(0, 40))
            .into_iter()
    );
    
    disp.flush().unwrap();
}
