use crate::database::{create_table, insert_sensor_data};
use crate::mqtt_message::SensorPayload;
use core::ptr::read;
use rumqttc::{Client, MqttOptions, QoS};
use rusqlite::{Connection, Result};
use std::path::Path;

mod database;
mod mqtt_message;

const DATABASE_PATH: &str = "database.db";

/// Callback run when an MQTT message is received.
///
/// Reads the reformats the payload for the database and inserts it to the database.
fn on_message(payload: &[u8]) -> Result<()> {
    let sensor_payload: SensorPayload = unsafe { read(payload.as_ptr() as *const SensorPayload) };
    let conn = connect_or_create_database(DATABASE_PATH).expect("Failed to access database");
    insert_sensor_data(&conn, &sensor_payload)?;
    Ok(())
}

/// Connects to the database or creates if necessary
fn connect_or_create_database(db_path: &str) -> Result<Connection> {
    let db_exists = Path::new(db_path).exists();
    let conn = Connection::open(db_path)?;

    if !db_exists {
        println!("Created database");
        create_table(&conn).expect("Failed to create table");
    } else {
        println!("Connected to database");
    }
    Ok(conn)
}

fn main() {
    println!("Hello, world!");

    let mut options = MqttOptions::("RpiServer", "localhost", 1883);
    let (mut mqtt_client, mut mqtt_connection) = Client::new(options, 10);
    mqtt_client.subscribe("WeatherStation", QoS::AtMostOnce);

    // TODO: Call on_message() during the polling for received messages
    for (i, notification) in mqtt_connection.iter().enumerate() {
        println!("Notification = {:?}", notification);
    }
}
