use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use crate::model::*;
use gpio_cdev::LineHandle;
use chrono::prelude::*;

pub fn poll_loop(
    conn: Arc<Mutex<Connection>>, 
    relay_pin: Arc<Mutex<LineHandle>>
) {
    let mut last_detected_hour = 0;
    let mut last_detected_minute = 0;

    loop {
        let local: DateTime<Local> = Local::now();
        let cur_hour = local.hour();
        let cur_minute = local.minute();

        println!("Polling...last: {}:{}, cur: {}:{}", last_detected_hour, last_detected_minute, cur_hour, cur_minute);

        let time_changed = last_detected_hour != cur_hour || last_detected_minute != cur_minute;

        if time_changed {
            println!("Time changed!");
        
            let conn = conn.lock().unwrap();
            let mut stmt = conn.prepare("select * from schedule").unwrap();

            let schedule_iter = stmt.query_map(params![], |row| {
                Ok(Schedule {
                    id: row.get(0)?,
                    hour: row.get(1)?,
                    minute: row.get(2)?,
                    watering_minute: row.get(3)?,
                    watering_second: row.get(4)?
                })
            });

            println!("Found schedule:");
     
            match schedule_iter {
                Ok(schedule_iter) => {
                    for schedule in schedule_iter {
                        let schedule = schedule.unwrap();
                        println!("{:?}", schedule);

                        // Turn on the relay pin
                        if schedule.hour == cur_hour as i32 && schedule.minute == cur_minute as i32 {
                            let total_secs = schedule.watering_minute * 60 + schedule.watering_second;

                            println!("Watering for {} secs", total_secs);

                            for i in 0..total_secs {
                                println!("{}...", total_secs - i);
                                {
                                    let mut relay_pin = relay_pin.lock().unwrap();
                                    relay_pin.set_value(1).unwrap();
                                }
                                thread::sleep(Duration::from_secs(1));
                            }

                            let mut relay_pin = relay_pin.lock().unwrap();
                            relay_pin.set_value(0).unwrap();

                            println!("Watering done");
                        }
                    }
                },
                Err(_) => {
                    println!("Fetching schedule error.");
                }
            }
        } else {
            println!("Time has not changed!");
        }

        last_detected_hour = cur_hour;
        last_detected_minute = cur_minute;

        thread::sleep(Duration::from_secs(10));
    }
    
}
