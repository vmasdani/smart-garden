use linux_embedded_hal::I2cdev;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rect};
//use embedded_graphics::primitives::{Rectangle};
use embedded_graphics::pixelcolor::PixelColorU8;
//use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::fonts::Font12x16;
//use ssd1306::{prelude::*, mode::GraphicsMode, Builder};
use ssd1306::{Builder, interface::I2cInterface};
use ssd1306::prelude::*;
use qrcode::QrCode;
//use async_std::task;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};

pub fn poll_loop(
    disp_arc: Arc<Mutex<Option<GraphicsMode<I2cInterface<I2cdev>>>>>    
) { 
    loop {
        {
            let mut disp_guard = disp_arc.lock().unwrap();

            match *disp_guard {
                Some(ref mut disp) => {
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
                    } else {
                        disp.draw(
                            Font12x16::render_str(&"NO IP".to_string()).translate(Coord::new(0, 0)).into_iter()    
                        );
                        disp.flush().unwrap();
                    }
                },
                _ => {
                    println!("Display not initiated.");
                }
            }
            // *disp.clear();
            
            /*
            match disp {
                Some(disp) => {
                    /*
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
                    } else {
                        disp.draw(
                            Font12x16::render_str(&"NO IP".to_string()).translate(Coord::new(0, 0)).into_iter()    
                        );
                        disp.flush().unwrap();
                    }
                    */
                },
                _ => {
                    println!("Display not initiated.");
                }
            }
            */
        }
    
        thread::sleep(Duration::from_secs(5));
    }
}
