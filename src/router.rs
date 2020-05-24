extern crate paho_mqtt as mqtt;

use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};
use uuid::Uuid;
use std::process;
use crate::models::*;

pub fn route(topic: String, msg: String, cli: &mqtt::AsyncClient, conn: &Arc<Mutex<Connection>>) {
    match topic.as_str() {
        "schedule/add" => {
            println!("Schedule topic, msg: {}", msg);

            // test add to db
            let conn = conn.lock().unwrap();
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
