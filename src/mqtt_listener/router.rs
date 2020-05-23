extern crate paho_mqtt as mqtt;

use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};
use uuid::Uuid;
use std::process;

pub fn route(topic: String, msg: String, cli: &mqtt::AsyncClient, conn: &Arc<Mutex<Connection>>) {
    match topic.as_str() {
        "schedule" => {
            println!("Schedule topic, msg: {}", msg);

            // test add to db
            let conn = conn.lock().unwrap();
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
