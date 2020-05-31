use rusqlite::{params, Connection};
use ssd1306::prelude::*;
use ssd1306::Builder;
use ssd1306::interface::i2c::I2cInterface;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
//use std::process;
use linux_embedded_hal::I2cdev;
use std::{time::Duration, thread};

pub fn init() -> (
    Connection, 
    // GraphicsMode<I2cInterface<I2cdev>>, 
    LineHandle
) {
    // Inititate GPIO
    let relay_pin = if let Ok(relay_pin) = gpio() {
        relay_pin
    } else {
        panic!("Error opening relay pin!");
    };
    
    // Initiate Display
    // let disp = disp();

    // Initiate DB
    let db = if let Ok(db) = db() {
        println!("Success opening db!");
        db
    } else {
        panic!("Error opening db!");
    };

    //(db, disp, relay_pin)
    (db, relay_pin)
}

pub fn gpio() -> gpio_cdev::errors::Result<LineHandle> {
    // Initiate GPIO
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let output = chip.get_line(7)?;
    let output_handle = output.request(LineRequestFlags::OUTPUT, 0, "relay")?;
    Ok(output_handle)
}

pub fn disp() -> GraphicsMode<I2cInterface<I2cdev>> {
    let mut i2c =
        if let Ok(i2c) = I2cdev::new("/dev/i2c-1") {
            println!("Success opening i2c!");
            Some(i2c)
        } else {
            println!("Error opening i2c!");
            None
        };

    match &i2c {
        Some(i2c) => {
            println!("Success opening i2c!");
        },
        None => {
            println!("Error opening i2c! Retrying in 5 secs..");
            thread::sleep(Duration::from_secs(5));
            return disp()
        }
    }

    let disp_result: Option<GraphicsMode<_>> = 
        if let Some(i2c) = i2c {
            Some(Builder::new().connect_i2c(i2c).into())
        } else {
            None
        };

    match disp_result {
        Some(mut disp_result) => {
            if let Ok(_) = disp_result.init() {
                disp_result.flush().unwrap();
                disp_result
            } else {
                println!("Error connecting to OLED! retrying in 5 secs...");
                thread::sleep(Duration::from_secs(5));
                disp()
            }
        },
        None => {
            println!("Error opening i2c! retrying in 5 secs...");
            thread::sleep(Duration::from_secs(5));
            disp()
        }
    }
}

pub fn db() -> rusqlite::Result<Connection>{
    let conn = Connection::open("./data.db3")?;
 
    conn.execute("
        create table if not exists schedule (
            id integer primary key autoincrement,
            hour integer,
            minute integer,
            watering_minute integer,
            watering_second integer
        )
    ", params![])?;

    conn.execute("
        create table if not exists sensor (
            id integer primary key autoincrement,
            serial_number  integer
        )
    ", params![])?;

    conn.execute("
        create table if not exists sensor_data (
            id integer primary key autoincrement,
            moisture integer
        );
    ", params![])?;
            
    Ok(conn)
}
