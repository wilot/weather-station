use crate::database::{create_table, insert_sensor_data};
use crate::mqtt_message::SensorPayload;
use core::ptr::read;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use rusqlite::{Connection, Result};
use std::path::Path;

mod database;
mod mqtt_message;

const DATABASE_PATH: &str = "database.db";

/// Callback run when an MQTT message is received.
///
/// Reads the reformats the payload for the database and inserts it to the database.
fn on_message(payload: &[u8]) -> Result<()> {
    // TODO: Consider using `bytemuck` to do this safely!
    let sensor_payload: SensorPayload = unsafe { read(payload.as_ptr() as *const SensorPayload) };
    let conn = connect_or_create_database(DATABASE_PATH).expect("Failed to access database");
    insert_sensor_data(&conn, &sensor_payload)?;
    Ok(())
}

/// Connects to the database or creates if necessary
fn connect_or_create_database(db_path: &str) -> Result<Connection> {
    let db_exists = Path::new(db_path).exists();
    let conn = Connection::open(db_path)?; // Implicitly creates if not found

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

    let options = MqttOptions::new("RpiServer", "localhost", 1883);
    let (mqtt_client, mut mqtt_connection) = Client::new(options, 10);
    mqtt_client
        .subscribe("WeatherStation", QoS::AtMostOnce)
        .expect("Couldn't subscribe to 'WeatherStation'");

    // TODO: Call on_message() during the polling for received messages
    for notification in mqtt_connection.iter() {
        match notification {
            Ok(Event::Incoming(Packet::Publish(pub_packet))) => {
                // let topic = pub_packet.topic;  // All topic should be WeatherStation...
                if let Err(e) = on_message(&pub_packet.payload) {
                    println!("Error submitting SQL query {:?}", e);
                } else {
                    println!("Message received successfully");
                }
            }
            Ok(packet) => println!("Misc MQTT event: {:?}", packet),
            Err(conn_err) => println!("Connection error: {:?}", conn_err),
        };
    }
}
