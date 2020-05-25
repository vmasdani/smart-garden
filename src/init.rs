use rusqlite::{params, Connection};
use ssd1306::prelude::*;
use ssd1306::Builder;
use ssd1306::interface::i2c::I2cInterface;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
//use std::process;
use linux_embedded_hal::I2cdev;


pub fn init() -> (
    Connection, 
    GraphicsMode<I2cInterface<I2cdev>>, 
    LineHandle
) {
    // Inititate GPIO
    let relay_pin = if let Ok(relay_pin) = gpio() {
        relay_pin
    } else {
        panic!("Error opening relay pin!");
    };
    
    // Initiate Display
    let disp = disp();

    // Initiate DB
    let db = if let Ok(db) = db() {
        println!("Success opening db!");
        db
    } else {
        panic!("Error opening db!");
    };

    (db, disp, relay_pin)
}

pub fn gpio() -> gpio_cdev::errors::Result<LineHandle> {
    // Initiate GPIO
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let output = chip.get_line(7)?;
    let output_handle = output.request(LineRequestFlags::OUTPUT, 0, "relay")?;
    Ok(output_handle)
}

pub fn disp() -> GraphicsMode<I2cInterface<I2cdev>> {
    let i2c = if let Ok(i2c) = I2cdev::new("/dev/i2c-1") {
        println!("Success opening i2c!");
        i2c
    } else {
        panic!("Error opening i2c!");
    };

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
    disp.init().unwrap();
    disp.flush().unwrap();
    disp
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
