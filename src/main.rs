extern crate chrono;
extern crate qrcode;
extern crate mosquitto_client as mosq;
extern crate serde;
extern crate serde_json;
extern crate linux_embedded_hal as hal;
extern crate embedded_graphics;
extern crate ssd1306;
extern crate machine_ip;
extern crate wiringpi;

use hal::I2cdev;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rect, Line};
use embedded_graphics::pixelcolor::PixelColorU8;
use embedded_graphics::fonts::Font12x16;
use ssd1306::{prelude::*, mode::GraphicsMode, Builder};
use mosq::Mosquitto;
use serde::{Serialize, Deserialize};
use std::thread;
use std::process::Command;
use std::error::Error;
use std::time::Duration;
use chrono::prelude::*;
use sqlite::State;
use qrcode::QrCode;
use wiringpi::pin::Value::{High, Low};

#[derive(Serialize, Deserialize, Debug)]
struct ScheduleArray {
    data: Vec<Schedule>
}

#[derive(Serialize, Deserialize, Debug)]
struct Schedule {
    id: i64,
    hour: i64,
    minute: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct WateringTime {
    minute: i64,
    second: i64
}

fn main() -> Result<(), Box <dyn Error>> {
    // Create database file if not exists
    if let Ok(_) = sqlite::open("./data.db") {
        println!("Database opening OK!");
    }
    else {
        println!("Database doesn't exist! Creating..");
        Command::new("echo > data.db")
            .output()
            .expect("Creating data.db failed!");
    }

    // Database and digital pin normalization 
    let pi = wiringpi::setup();
    let pin = pi.output_pin(6);

    pin.digital_write(Low);

    let connection = sqlite::open("./data.db").unwrap();
    
    // create schedule table
    if let Err(e) = connection.execute(
    "
        create table schedule(
            id integer primary key autoincrement,
            hour tinyint,
            minute tinyint
        );                   
    "
    ) { println!("{}", e) }

    // create watering_time table
    if let Err(e) = connection.execute(
    "
        create table watering_time(
            id integer primary key autoincrement,
            minute tinyint,
            second tinyint
        );                   
    "
    ) { println!("{}", e) }

    // connection.execute("insert into schedule(hour, minute) values(1, 5)");

    // Check if watering_time exists
    let mut wt_exist_stmt = connection.prepare("select * from watering_time where id=1").unwrap();

    let mut wt_time_counter = 0;
    while let State::Row = wt_exist_stmt.next().unwrap() {
        wt_time_counter += 1;
        println!("minute: {}, second: {}", wt_exist_stmt.read::<i64>(1).unwrap(), wt_exist_stmt.read::<i64>(2).unwrap());
    }

    println!("Num of watering_time = {}", wt_time_counter);
    if wt_time_counter == 0 {
        println!("Watering time is empty! Inserting....");
        connection.execute("insert into watering_time(id, minute, second) values(1, 0, 10)");
    }
    else {
        println!("Watering time is not empty!");
    }


    // MQTT Listener thread
    let listener = thread::spawn(|| {
        println!("MQTT thread started!");
        let m = Mosquitto::new("client-1");
        
        m.connect("0.0.0.0", 1883);

        let control = m.subscribe("control", 0).expect("Cannot subscribe to control topic!");
        let schedule = m.subscribe("schedule/#", 0).expect("Cannot subscribe to schedule!");
        let watering = m.subscribe("watering/#", 0).expect("Cannot subscribe to watering!");
        let power = m.subscribe("power", 0).expect("Cannot subscribe to power!");

        let mut mc = m.callbacks(());
        mc.on_message(|_,msg| {
            let topic = msg.topic().to_string();
            let message = msg.text().to_string();

            match &topic[..] {
                "schedule/req" => {
                    println!("Schedule req topic detected!");
                    
                    let connection = sqlite::open("./data.db").unwrap();
                    let mut schedule_stmt = connection.prepare("select * from schedule").unwrap();

                    let mut schedule_array = ScheduleArray {
                        data: Vec::new()
                    };
                        
                    while let State::Row = schedule_stmt.next().unwrap() {
                        let id = schedule_stmt.read::<i64>(0).unwrap();
                        let hour = schedule_stmt.read::<i64>(1).unwrap();
                        let minute = schedule_stmt.read::<i64>(2).unwrap();
                        
                        schedule_array.data.push(Schedule {
                            id: id,
                            hour: hour,
                            minute: minute
                        });
                    }

                    let schedule_json = serde_json::to_string(&schedule_array).unwrap();
                    m.publish("schedule/res", &schedule_json[..].as_bytes(), 0, false).unwrap();
                },
                "schedule/add" => {
                    println!("Schedule add topic detected!");
                
                    let data_json: serde_json::Value = serde_json::from_str(&message[..]).unwrap();
                    let hour = &data_json["hour"];
                    let minute = &data_json["minute"];
                    
                    println!("From message, hour: {}, minute: {}", hour, minute);

                    let connection = sqlite::open("./data.db").unwrap();
                    connection.execute(format!("insert into schedule(hour, minute) values({}, {})", hour, minute));
                    println!("Schedule added successfully!");
                },
                "schedule/delete" => {
                    println!("Schedule delete topic detected!");
                
                    let data_json: serde_json::Value = serde_json::from_str(&message[..]).unwrap();
                    let id = &data_json["id"];
                    
                    println!("From message, id: {}", id);

                    let connection = sqlite::open("./data.db").unwrap();
                    connection.execute(format!("delete from schedule where id={}", id));
                    println!("Schedule deleted successfully!");
                },
                
                "watering/req" => {
                    println!("Watering req topic detected!");

                    let connection = sqlite::open("./data.db").unwrap();
                    let mut wt_time_stmt = connection.prepare("select * from watering_time where id=1").unwrap();

                    let mut watering_time = WateringTime {
                        minute: 0,
                        second: 0
                    };

                    while let State::Row = wt_time_stmt.next().unwrap() {
                        let minute = wt_time_stmt.read::<i64>(1).unwrap();
                        let second = wt_time_stmt.read::<i64>(2).unwrap();

                        watering_time.minute = minute;
                        watering_time.second = second;
                    }

                    let watering_time_json = serde_json::to_string(&watering_time).unwrap();
                    m.publish("watering/res", &watering_time_json[..].as_bytes(), 0, false).unwrap();
                },
                "watering/update" => {
                    
                    let data_json: serde_json::Value = serde_json::from_str(&message[..]).unwrap();
                    let minute = &data_json["minute"];
                    let second = &data_json["second"];
                    
                    println!("From message, minute: {}, second: {}", minute, second);

                    let connection = sqlite::open("./data.db").unwrap();
                    connection.execute(format!("update watering_time set minute={}, second={} where id=1", minute, second));
                    println!("Watering updated successfully!");
                },
                "control" => {
                    // TODO: Add control GPIO functionalities
                    println!("Control topic detected!");
                    
                    let data_json: serde_json::Value = serde_json::from_str(&message[..]).unwrap();
                    let control_type = &data_json["control_type"];

                    println!("Control data: {}", control_type);

                    let pi = wiringpi::setup();
                    let pin = pi.output_pin(6);

                    if control_type == 1 {
                        println!("Turning on valve!");
                        pin.digital_write(High);
                    }
                    else if control_type == 0 {
                        println!("Turning off valve!");
                        pin.digital_write(Low);
                    }
                },
                "power" => {
                    println!("Power topic detected!");
                    Command::new("poweroff")
                        .output()
                        .expect("Failed to power off device!");
                },
                _ => println!("Topic irrelevant.")
            }

            println!("Topic: {}, Message: {}", topic, message);
        });

        m.loop_until_disconnect(200);
    });

    
    // Database poller thread
    let db_poller = thread::spawn(|| {
        let mut last_detected_time: DateTime<Local> = Local::now();
        
        loop {
            let current_time = Local::now();
 
            let last_hour = &last_detected_time.hour();
            let last_minute = &last_detected_time.minute();
            let cur_hour = &current_time.hour();
            let cur_minute = &current_time.minute();

            

            println!("Last: {}.{}, Current:{}.{}", last_hour, last_minute, cur_hour, cur_minute);
            
            if last_hour == cur_hour && last_minute == cur_minute {
                
                
                println!("Time is still the same!");
            }
            else {
                last_detected_time = current_time;
                println!("Time has changed!");
            
                let connection = sqlite::open("./data.db").unwrap();
                let mut schedule_stmt = connection.prepare(format!("select * from schedule where hour={} and minute={}", cur_hour, cur_minute)).unwrap();

                let mut schedule_counter = 0;
                
                while let State::Row = schedule_stmt.next().unwrap() {
                    let hour = schedule_stmt.read::<i64>(1).unwrap();
                    let minute = schedule_stmt.read::<i64>(2).unwrap();
                    
                    println!("Found match! {}:{}", hour, minute);
                    schedule_counter += 1;
                }
                
                if schedule_counter != 0 {
                    println!("Match detected! Watering start...");

                    let mut counter = 0;
                    let mut watering_stmt = connection.prepare("select * from watering_time where id=1").unwrap();
                    
                    while let State::Row = watering_stmt.next().unwrap() {
                        let minute = watering_stmt.read::<i64>(1).unwrap();
                        let second = watering_stmt.read::<i64>(2).unwrap();

                        let total_secs = minute * 60 + second;

                        println!("Watering for {} minute and {} seconds, totalling {} seconds.", minute, second, total_secs);
                        counter = total_secs;
                    }

                    let pi = wiringpi::setup();
                    let pin = pi.output_pin(6);

                    pin.digital_write(High);
                    
                    while counter > 0 {
                        println!("{}...", counter);
                        counter -= 1;
                        thread::sleep(Duration::from_secs(1));       
                    }
                    
                    pin.digital_write(Low);

                    println!("All done!");
                }
            }

            thread::sleep(Duration::from_secs(10));
        }
    });
    
    // IP checker thread
    let ip_poller = thread::spawn(|| {
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
            else {
                println!("IP not found!");
                disp.clear();

                disp.draw(
                    Font12x16::render_str(&format!("{}", "NO IP!".to_string()))
                    .into_iter()
                );

                disp.flush().unwrap();
            }
            thread::sleep(Duration::from_secs(10));
        }
    });
 
    // Keep main thread alive
    let handles = vec![listener, db_poller, ip_poller];
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    Ok(())
}
