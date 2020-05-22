use linux_embedded_hal::I2cdev;
use embedded_graphics::prelude::*;
//use embedded_graphics::primitives::{Rect, Line};
use embedded_graphics::primitives::{Rect};
use embedded_graphics::pixelcolor::PixelColorU8;
//use embedded_graphics::fonts::Font12x16;
//use ssd1306::{prelude::*, mode::GraphicsMode, Builder};
use ssd1306::{mode::GraphicsMode, Builder};
use qrcode::QrCode;
use async_std::task;
use std::time::Duration;

pub async fn poll_loop() {
    println!("IP poller thread started!");

    let i2c = I2cdev::new("/dev/i2c-1").unwrap();

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    disp.init().unwrap();
    disp.clear();

    // disp.flush().unwrap();

    loop {
        disp.clear();

        disp.draw(Rect::new(Coord::new(35, 0), Coord::new(87, 52)).with_stroke(Some(PixelColorU8(1u8))).into_iter());

        if let Some(ip) = machine_ip::get() {
            let detected_ip = format!("http://{}", ip.to_string());
            // let detected_ip = String::from("http://192.168.1.1");
            println!("IP detected: {}", detected_ip);

            let code = QrCode::new(detected_ip).unwrap();
            
            let ip_qr = code.render::<char>()
                .quiet_zone(false)
                .module_dimensions(2, 2)
                .build();

            let mut y_point = 1;
            let mut x_point = 36;    

            for i in ip_qr.chars() {
                if i == '\n' {
                    x_point = 36;
                    y_point += 1;
                }

                if i == ' ' {
                    // print!("{}", " ");
                    disp.set_pixel(x_point, y_point, 1);
                }
                else {
                    // print!("{}", i);
                }
                x_point += 1;
                
            }
            disp.flush().unwrap();

            // println!("{}", ip_qr);
        }
    
        task::sleep(Duration::from_secs(5)).await;
    }
}
