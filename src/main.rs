// TODO: add gpio control in fn control()
// add QR code
// add database checking thread
// TODO: Move to Orange Pi and add linux_embedded_hal functionalities
// Add default watering time
// Add update watering_time and delete schedule
// TODO: update index.html. Add timepicker and alert response if button is pushed.

extern crate chrono;
extern crate qrcode;
extern crate mosquitto_client as mosq;
extern crate serde;
extern crate serde_json;

use mosq::Mosquitto;
use serde::{Serialize, Deserialize};
use std::thread;
use std::process::Command;
use std::error::Error;
use std::time::Duration;
use chrono::prelude::*;
use sqlite::State;
use qrcode::QrCode;

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
    // Starting database
    thread::spawn(|| {
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
    });
    

    // MQTT Listener thread
    thread::spawn(|| {
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
    thread::spawn(|| {
        let mut last_detected_time: DateTime<Local> = Local::now();
        
        loop {
            let current_time = Local::now();
 
            let last_hour = &last_detected_time.hour();
            let last_minute = &last_detected_time.minute();
            let cur_hour = &current_time.hour();
            let cur_minute = &current_time.minute();

            

            println!("Last: {}.{}, Current:{}.{}", last_hour, last_minute, cur_hour, cur_minute);
            
            if(last_hour == cur_hour && last_minute == cur_minute) {
                
                
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
                
                if(schedule_counter != 0) {
                    println!("Match detected! Watering start...");

                    let mut counter = 10;
                    
                    while(counter > 0) {
                        println!("{}...", counter);
                        counter -= 1;
                        thread::sleep(Duration::from_secs(1));       
                    }
                    println!("All done!");
                }
            }

            thread::sleep(Duration::from_secs(10));
        }
    });
    /*
    thread::spawn(move || {
        let mut last_detected_time: DateTime<Local> = Local::now();
        let conn_poller = Connection::open("./db/data.db3").unwrap();
            
        loop {
            let current_time = Local::now();

            let last_hour = &last_detected_time.hour();
            let last_minute = &last_detected_time.minute();
            let cur_hour = &current_time.hour();
            let cur_minute = &current_time.minute();

            if last_hour == cur_hour && last_minute == cur_minute {
                println!("Time is still the same! {}:{} vs {}:{}", last_hour, last_minute, cur_hour, cur_minute);
            }
            else {
                last_detected_time = current_time.clone();
                println!("Time has changed! {}:{} vs {}:{}", last_hour, last_minute, cur_hour, cur_minute);
            
                // Poll database
                let mut stmt = conn_poller.prepare("select * from schedule where hour=?1 and minute=?2").unwrap();
                
                let schedule_iter = stmt.query_map(&[&cur_hour, &cur_minute], |row| Ok(Schedule {
                    id: row.get(0).unwrap(),
                    hour: row.get(1).unwrap(),
                    minute: row.get(2).unwrap()
                })).unwrap();

                let count = &schedule_iter.count();
                println!("Match: {}", count);
            }
            
            thread::sleep(Duration::from_secs(10));
        }
    });
    */

    // IP checker thread
    thread::spawn(|| {
        println!("IP poller thread started!");
        loop {
            if let Some(ip) = machine_ip::get() {
                let detected_ip = ip.to_string();
                println!("IP detected: {}", detected_ip);

                let code = QrCode::new(detected_ip).unwrap();
                let ip_qr = code.render::<char>()
                    .quiet_zone(false)
                    .module_dimensions(2, 1)
                    .build();

                println!("{}", ip_qr);
            }
            else {
                println!("IP not found!");
            }
            thread::sleep(Duration::from_secs(60));
        }
    });
 
    // Keep main thread alive
    loop {
    
    }

    Ok(())
}
