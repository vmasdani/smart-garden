use std::sync::{Arc, Mutex};
use rusqlite::Connection;

pub fn route(topic: String, msg: String, conn: &Arc<Mutex<Connection>>) {
    match topic.as_str() {
        "schedule" => {
            println!("Schedule topic")
        },
        "watering_time" => {
            println!("Watering time topic");
        },
        "sensor" => {
            println!("Sensor topic");
        },
        "report" => {
            println!("Report topic");
        },
         
        _ => {
            println!("No topic");
        }
    }
}
