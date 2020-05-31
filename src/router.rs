extern crate paho_mqtt as mqtt;

use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};
//use uuid::Uuid;
//use std::process;
use crate::model::*;
use std::{thread, time::Duration};

pub fn route(
    topic: String, 
    msg: String, 
    cli: &mqtt::AsyncClient, 
    conn: Arc<Mutex<Connection>>
) {
    match topic.as_str() {
        "schedule/add" => {
            println!("Schedule topic, msg: {}", msg);
            
            let schedule: serde_json::Result<Schedule> = serde_json::from_str(msg.as_str());

            match schedule {
                Ok(schedule) => {
                    // test add to db
                    println!("{:?}", schedule);
                    let mut id: String;
                    {
                        let conn = conn.lock().unwrap();
                        conn.execute("insert into schedule(hour, minute, watering_minute, watering_second) values(?, ?, ?, ?)", params![schedule.hour, schedule.minute, schedule.watering_minute, schedule.watering_second]).unwrap();
                        id = conn.last_insert_rowid().to_string();
                    }
                    println!("Inserted id: {}", id);
                    let msg = mqtt::Message::new("schedule/res", id, 0);
                    cli.publish(msg);
                },
                _ => {
                    println!("Parsing schedule JSON failed.");
                }
            }
        },
        "schedule/req" => {
            let conn = conn.lock().unwrap();

            let mut stmt = conn.prepare("select * from schedule").unwrap();
            let rows = stmt.query_map(params![], |row| {
                Ok(Schedule {
                    id: row.get(0)?,
                    hour: row.get(1)?,
                    minute: row.get(2)?,
                    watering_minute: row.get(3)?,
                    watering_second: row.get(4)?
                })
            }).unwrap();

            for schedule in rows {
                println!("{:?}", schedule.unwrap());
            } 
        },
        "schedule/res" => {
        
        },
        "water/on" => {
            println!("Turning on!");
            //thread::sleep(Duration::from_secs(2));
            //println!("Watering kompliet!");
        },
        "water/off" => {
            println!("Turning off!");
        },
        "watering_time" => {
            println!("Watering time topic");
        },
        "sensor/req" => {
            println!("Sensor req topic");
            // Publish data to sensor/res
            let msg = mqtt::Message::new("sensor/res", "MyMessage", 0);
            cli.publish(msg);
        },
        "report" => {
            println!("Report topic");
        },
        "poweroff" => {
            // TODO: power off machine
        },
        _ => {
            println!("Topic irrelevant");
        }
    }

    // cli.disconnect(None).unwrap();
}
