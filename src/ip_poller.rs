use linux_embedded_hal::I2cdev;
use embedded_graphics::{
    fonts::{Font12x16, Text, Font6x8, Font8x16},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    style::{PrimitiveStyle, TextStyle, PrimitiveStyleBuilder}
};
use ssd1306::{Builder, interface::I2cInterface};
use ssd1306::prelude::*;
use qrcode::QrCode;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::qr_fit;

pub fn poll_loop(
    disp_arc: Arc<Mutex<Option<GraphicsMode<I2cInterface<I2cdev>>>>>    
) { 
    loop {
        {
            let mut disp_guard = disp_arc.lock().unwrap();

            match *disp_guard {
                Some(ref mut disp) => {
                    disp.clear();

                    if let Some(ip) = machine_ip::get() {
                        let detected_ip = format!("http://{}", ip.to_string());
                        // let detected_ip = "http://192.168.100.62".to_string();
                        println!("IP detected: {}", detected_ip);

                        let code = QrCode::new(detected_ip).unwrap();
                        
                        let ip_qr = code.render::<char>()
                            .quiet_zone(false)
                            .module_dimensions(1, 1)
                            .build();

                        let ip_qr_fit = qr_fit::fit(ip_qr);      
                        
                        for x in 0..62 {
                            for y in 0..62 {
                                match ip_qr_fit[x][y] {
                                    0 => Pixel(Point::new(x as i32 + 1, y as i32 + 1), BinaryColor::On).draw(disp),
                                    _ => Pixel(Point::new(x as i32 + 1, y as i32 + 1), BinaryColor::Off).draw(disp)
                                };
                            }
                        }
                        
                        // Draw guard
                        
                        Rectangle::new(Point::new(0, 0), Point::new(63, 63))
                            .into_styled(
                                PrimitiveStyleBuilder::new()
                                    .stroke_width(1)
                                    .stroke_color(BinaryColor::On)
                                    .build()
                            )
                            .draw(disp);
                         
                        Text::new("192.168", Point::new(66, 0))
                            .into_styled(TextStyle::new(Font8x16, BinaryColor::On))
                            .draw(disp);

                        Text::new("192.168", Point::new(66, 18))
                            .into_styled(TextStyle::new(Font8x16, BinaryColor::On))
                            .draw(disp);

                        disp.flush().unwrap();
                    } else {
                        disp.clear();

                        Text::new("NO IP", Point::new(0, 0))
                            .into_styled(TextStyle::new(Font12x16, BinaryColor::On))
                            .draw(disp);

                        disp.flush().unwrap();
                    }
                },
                _ => {
                    println!("Display not initiated.");
                }
            }
        }
    
        thread::sleep(Duration::from_secs(10));
    }
}
