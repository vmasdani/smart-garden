mod ip_poller;
mod init;
mod db_poller;
mod mqtt_listener;
mod router;
mod model;
mod test_display;

use std::{thread, time::Duration};
use std::sync::{Arc, Mutex};

fn main() {
    let (conn, relay_pin) = init::init();

    //test_display::test_display(&mut disp);

    let conn_arc = Arc::new(Mutex::new(conn));
    // let _disp_arc = Arc::new(Mutex::new(disp));
    let relay_pin_arc = Arc::new(Mutex::new(relay_pin));
    
    // Arc clones for db poller
    let db_conn_clone = Arc::clone(&conn_arc);
    
    // Arc clones for MQTT
    let mqtt_conn_clone = Arc::clone(&conn_arc);
    let mqtt_relay_pin_clone = Arc::clone(&relay_pin_arc);

    let db_handle = thread::spawn(move || {
        db_poller::poll_loop(db_conn_clone);
    }); 
    
    let mqtt_handle = thread::spawn(move || {
        mqtt_listener::listen(mqtt_conn_clone, mqtt_relay_pin_clone);
    });

    let ip_poller_handle = thread::spawn(move || {
        //ip_poller::poll_loop();
        let mut disp = init::disp();
    });

    for handle in vec![db_handle, mqtt_handle, ip_poller_handle] {
        handle.join().unwrap();
    }

    // task::block_on(main_loop(conn_arc, disp_arc, relay_arc));

    /*
    // MQTT Listener thread
    let conn_mqtt_clone = Arc::clone(&conn);
    let _mqtt_listener_handle = thread::spawn(move || {
        mqtt_listener::listen(conn_mqtt_clone);
    });

    // Database poller thread
    let conn_db_poller_clone = Arc::clone(&conn);    
    let _db_poller_handle = thread::spawn(move || {
        db_poller::poll_loop(conn_db_poller_clone);
    });
    */

    /*
    let ip_poller_handle = thread::spawn(|| {
        ip_poller::poll_loop();
    });

    // Keep main thread alive
    for handle in vec![mqtt_listener_handle, db_poller_handle, ip_poller_handle] {
        handle.join().unwrap();
    }
    */
    
    /*
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
    */
    
}
