use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use crate::model::*;

pub fn poll_loop(conn: Arc<Mutex<Connection>>) {
    loop {
        println!("Polling...");

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
                    println!("{:?}", schedule.unwrap());    
                }   
            },
            Err(_) => {
                println!("Fetching schedule error.");
            }
        }

        thread::sleep(Duration::from_secs(10));
    }
    
}
