mod ip_poller;
mod init;
mod db_poller;
mod mqtt_listener;
mod router;
mod model;
mod qr_fit;

use std::{thread, time::Duration};
use std::sync::{Arc, Mutex};
use embedded_graphics::prelude::*;
use ssd1306::interface::I2cInterface;
use ssd1306::prelude::*;
use linux_embedded_hal::I2cdev;

fn main() {
    let (conn, relay_pin) = init::init();
    let disp_arc: Arc<Mutex<Option<GraphicsMode<I2cInterface<I2cdev>>>>> = Arc::new(Mutex::new(None));

    let conn_arc = Arc::new(Mutex::new(conn));
    let relay_pin_arc = Arc::new(Mutex::new(relay_pin));
    
    // Arc clones for db poller
    let db_conn_clone = Arc::clone(&conn_arc);
    let db_relay_pin_clone = Arc::clone(&relay_pin_arc);

    // Arc clones for MQTT
    let mqtt_conn_clone = Arc::clone(&conn_arc);
    let mqtt_relay_pin_clone = Arc::clone(&relay_pin_arc);

    let db_handle = thread::spawn(move || {
        db_poller::poll_loop(db_conn_clone, db_relay_pin_clone);
    }); 
    
    let mqtt_handle = thread::spawn(move || {
        mqtt_listener::listen(mqtt_conn_clone, mqtt_relay_pin_clone);
    });

    let ip_poller_handle = thread::spawn(move || {
        // Arc clones for IP poller
        let init_disp_clone = Arc::clone(&disp_arc);
        init::disp(init_disp_clone);
        
        let ip_disp_clone = Arc::clone(&disp_arc);
        ip_poller::poll_loop(ip_disp_clone);
    });

    for handle in vec![db_handle, mqtt_handle, ip_poller_handle] {
        handle.join().unwrap();
    }

        
}
