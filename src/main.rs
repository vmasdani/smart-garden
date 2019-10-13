// TODO: add gpio control in fn control()
// add QR code
// add database checking thread
// TODO: Move to Orange Pi and add linux_embedded_hal functionalities
// Add default watering time
// Add update watering_time and delete schedule
// TODO: update index.html. Add timepicker and alert response if button is pushed.

extern crate chrono;
extern crate qrcode;
extern crate rusqlite;

use actix_web::{web, App, HttpResponse, HttpServer};
use actix_files::NamedFile;
use actix_files as fs;
use serde::Deserialize;
use serde::Serialize;
use std::thread;
use std::process::Command;
use std::error::Error;
use std::time::Duration;
use chrono::prelude::*;
use rusqlite::{Connection, NO_PARAMS};
use qrcode::QrCode;

#[derive(Debug, Deserialize)]
struct Schedule {
    id: u32,
    hour: u32,
    minute: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct WateringTime {
    id: u32,
    minute: u32,
    second: u32
}

// Control struct
#[derive(Deserialize)]
struct Control {
    control_type: u8
}

fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./www/index.html")?)
}

fn control(control_json: web::Json<Control>) -> HttpResponse {
    // TODO: Add GPIO control
    HttpResponse::Ok()
        .content_type("plain/text")
        .body(format!("Control success! {}", control_json.control_type))
}

fn add_schedule(schedule_json: web::Json<Schedule>) -> HttpResponse {
    let conn = Connection::open("./db/data.db3").unwrap();
    
    if let Ok(_res) = conn.execute("insert into schedule(hour, minute) values (?1, ?2)",
        &[&schedule_json.hour, &schedule_json.minute]
    ) { HttpResponse::Ok().body("Success adding schedule!") }
    else { HttpResponse::Ok().body("Error adding schedule!") }
}

fn get_schedule() -> HttpResponse {
    let conn = Connection::open("./db/data.db3").unwrap();

    let mut query_get = conn.prepare("select * from schedule").unwrap();
    let schedule_iter = query_get.query_map(NO_PARAMS, |row| Ok(Schedule {
        id: row.get(0).unwrap(),
        hour: row.get(1).unwrap(),
        minute: row.get(2).unwrap()
    })).unwrap();

    let mut resp = String::from("[");

    for schedule in schedule_iter {
        let sched = schedule.unwrap();
        resp = format!("{}{{\"id\":{}, \"hour\":{}, \"minute\":{}}},", resp, sched.id, sched.hour, sched.minute);
        println!("Found sched: {:?}", sched);
    }

    let resp_len = resp.len();
    
    // Strip end comma
    let resp_fmt = format!("{}]", &resp[0..resp_len - 1]);

    HttpResponse::Ok()
        .content_type("application/json")
        // .body(format!("{}", query_get.column_count()))
        .body(resp_fmt)
}

fn delete_schedule(path: web::Path<(u32,)>) -> HttpResponse {
    let conn = Connection::open("./db/data.db3").unwrap();
    
    if let Ok(_res) = conn.execute("delete from schedule where id=?1",
        &[&path.0]
    ) { HttpResponse::Ok().body("Success deleting schedule!") }
    else { HttpResponse::Ok().body("Error deleting schedule!") }
}

fn get_watering_time() -> HttpResponse {
    let conn = Connection::open("./db/data.db3").unwrap();

    let mut query_get = conn.prepare("select * from watering_time where id=1").unwrap();
    let wt_iter = query_get.query_row(NO_PARAMS, |row| Ok(WateringTime {
        id: row.get(0).unwrap(),
        minute: row.get(1).unwrap(),
        second: row.get(2).unwrap()
    })).unwrap();

    let found_wt = wt_iter;

    HttpResponse::Ok().json(WateringTime {
        id: found_wt.id,
        minute: found_wt.minute,
        second: found_wt.second
    })
}

fn update_watering_time(wt_json: web::Json<WateringTime>) -> HttpResponse {
    let conn = Connection::open("./db/data.db3").unwrap();
    
    if let Ok(_res) = conn.execute("update watering_time set minute=?1, second=?2 where id=1",
        &[&wt_json.minute, &wt_json.second]
    ) { HttpResponse::Ok().body("Success updating watering time!") }
    else { HttpResponse::Ok().body("Error updating watering time!") }
}

fn poweroff() {
    println!("Powering off!");
    let res = Command::new("/sbin/poweroff").output().unwrap();
    println!("{}", String::from_utf8_lossy(&res.stdout));
}

fn main() -> rusqlite::Result<()>{
    // Create tables if not exists
    let conn =  Connection::open("./db/data.db3").unwrap(); 

    // Create schedule table
    if let Ok(_res) = conn.execute(
        "
            create table schedule (
                id  integer primary key autoincrement,
                hour integer,
                minute integer
            );
            
        ",
        NO_PARAMS
    ) {
        println!("Create schedule table success!");
    }
    else {
        println!("Create schedule table error! Already exists?");
    }

    // Create watering_time table
    if let Ok(_res) = conn.execute(
        "
            create table watering_time (
                id integer primary key autoincrement,
                minute integer,
                second integer
            )
        ",
        NO_PARAMS
    ) {
        println!("Create watering_time table success!");
    }
    else {
        println!("Create watering_time table error! Already exists?");
    }

    // TODO: Add default watering time
    let mut get_wt_statement = conn.prepare("select * from watering_time where id=1").unwrap();
    let wt_iter = get_wt_statement.query_map(NO_PARAMS, |row| Ok(WateringTime {
        id: row.get(0).unwrap(),
        minute: row.get(1).unwrap(),
        second: row.get(2).unwrap()
    })).unwrap();

    let count = wt_iter.count();
    if count == 0 {
        println!("No watering time found! adding..");
        if let Ok(_res) = conn.execute("insert into watering_time values(1, 0, 30)", NO_PARAMS) {
            println!("Successfully added watering time table!");
        }
        else {
            println!("Error adding watering time table!");
        }
    }
    else {
        println!("Watering time found!");
    }

    // Database poller thread
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

    // IP checker thread
    thread::spawn(|| {
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

    HttpServer::new(|| {
        App::new()
            .route("/control", web::post().to(control))
            .route("/schedule", web::post().to(add_schedule))
            .route("/schedule", web::get().to(get_schedule))
            .route("/schedule/{schedule_id}", web::delete().to(delete_schedule))
            .route("/watering-time", web::get().to(get_watering_time))
            .route("/watering-time", web::put().to(update_watering_time))
            .route("/poweroff", web::post().to(poweroff))
            .service(fs::Files::new("/", "./www").index_file("index.html"))
            //.route("/", web::get().to(index))
            
    })
    .bind("0.0.0.0:8000")
    .unwrap()
    .run()
    .unwrap();

    Ok(())
}
