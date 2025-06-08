use crate::database::{create_tables, insert_sensor_data, insert_test_record};
use crate::mqtt_message::SensorMessage;
use core::ptr::read;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use rusqlite::Connection;
use std::env;

mod database;
mod mqtt_message;

const DATABASE_PATH: &str = "database.db";

/// Callback run when an MQTT message is received.
///
/// Reads the reformats the payload for the database and inserts it to the database.
fn on_message(payload: &[u8]) {
    let sensor_message = match SensorMessage::from_bytes(payload) {
        Ok(message) => message,
        Err(error) => {
            println!("Error parsing message bytes: {}", error);
            return;
        }
    };

    let conn = get_database_connection();
    insert_sensor_data(&conn, &(sensor_message.payload))
        .expect("Failed to insert sensor payload into database.");
}

fn get_database_connection() -> Connection {
    let conn = Connection::open(DATABASE_PATH); // Implicitly creates if not found
    conn.expect("Failed to connect to the database!")
}

fn test_database() {
    let conn = get_database_connection();
    insert_test_record(&conn).expect("Database test failed!");
}

fn main() {
    // Set up database
    {
        let conn = get_database_connection();
        create_tables(&conn)
    }
    .expect("Could not initialise database");

    // Connect to the MQTT server
    let options = MqttOptions::new("RpiServer", "localhost", 1883);
    let (mqtt_client, mut mqtt_connection) = Client::new(options, 10);

    println!("Database connection initialised and MQTT connected.");

    // If passed the `--setup-test` argument, setup the database and test it!
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--setup-test" {
        test_database();
    }

    // Subscribe to the MQTT channel
    mqtt_client
        .subscribe("weather/station", QoS::AtMostOnce)
        .expect("Couldn't subscribe to 'WeatherStation'");

    for notification in mqtt_connection.iter() {
        match notification {
            Ok(Event::Incoming(Packet::Publish(pub_packet))) => {
                // let topic = pub_packet.topic;  // All topic should be WeatherStation...
                on_message(&pub_packet.payload);
            }
            Ok(packet) => println!("Misc MQTT event: {:?}", packet),
            Err(conn_err) => println!("Connection error: {:?}", conn_err),
        };
    }
}
