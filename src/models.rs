use serde::{Deserialize, Serialize};
//use serde_json::Result;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Schedule {
    pub id: i32,
    pub hour: i32,
    pub minute: i32,
    pub watering_minute: i32,
    pub watering_second: i32
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Sensor {
    pub id: i32,
    pub serial_number: String
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct SensorData {
    pub id: i32,
    pub moisture: i32
}
