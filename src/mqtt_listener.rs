extern crate paho_mqtt as mqtt;

use crate::router;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
//use async_std::task;
use std::time::Duration;
use std::process;
use std::thread;
use uuid::Uuid;
use gpio_cdev::LineHandle;

pub fn listen(
    conn: Arc<Mutex<Connection>>,
    _relay_pin: Arc<Mutex<LineHandle>>
) {
    fn on_connect_success(cli: &mqtt::AsyncClient, _msgid: u16) {
        println!("Connection succeeded");
        cli.subscribe_many(&["#"], &[1]);
        println!("Subscribing to topics: {:?}", &["#"]);
    }

    fn on_connect_failure(cli: &mqtt::AsyncClient, _msgid: u16, _rc: i32) {
        thread::sleep(Duration::from_millis(2500));
        cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
    }

    let host = "tcp://localhost:1883".to_string();
    let client_id = Uuid::new_v4().to_hyphenated().to_string();

    
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(client_id.clone())
        .finalize();

    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating client! {:?}", e);
        process::exit(1);
    }); 

    cli.set_connected_callback(|_cli: &mqtt::AsyncClient| {
        println!("Connected!");
    });

    cli.set_connection_lost_callback(|cli: &mqtt::AsyncClient| {
        println!("Connection lost! Attempting reconnect..");
        thread::sleep(Duration::from_millis(2500));
        cli.reconnect_with_callbacks(on_connect_success, on_connect_failure);
    });

    cli.set_message_callback(move |cli, msg| {
        if let Some(msg) = msg {
            let topic = msg.topic();
            let payload_str = msg.payload_str();

            println!("{} - {}", topic, payload_str);
     
            // let conn_clone = Arc::clone(&conn);
            router::route(
                topic.to_string(), 
                payload_str.to_string(), 
                &cli, 
                &conn
            );
        }
    });

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .clean_session(true)
        .finalize();

    println!("Connecting to the MQTT server...");
    println!("{}", format!("Subscriber client id: {}", client_id));
    cli.connect_with_callbacks(conn_opts, on_connect_success, on_connect_failure);

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
