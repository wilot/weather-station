use crate::database::WeatherDatabase;
use crate::mqtt_message::SensorMessage;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
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
            eprintln!("Error parsing message bytes: {}", error);
            return;
        }
    };

    let database_conn = match WeatherDatabase::new(DATABASE_PATH) {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("Error connecting to database `on_message`: {}", error);
            return;
        }
    };

    database_conn
        .insert_sensor_data(&sensor_message.payload)
        .unwrap_or_else(|err| eprintln!("Failed to insert sensor payload into database: {}", err));
}

fn test_database() {
    let database_conn = match WeatherDatabase::new(DATABASE_PATH) {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("Error connecting to database for test: {}", error);
            return;
        }
    };
    database_conn
        .test_sqlite()
        .unwrap_or_else(|err| eprintln!("Database test failed: {}", err));
}

fn main() {
    // Connect to the MQTT server
    let options = MqttOptions::new("RpiServer", "localhost", 1883);
    let (mqtt_client, mut mqtt_connection) = Client::new(options, 10);

    println!("Database connection initialised and MQTT connected.");

    // Verify presence of tables or create if necessary
    WeatherDatabase::new(DATABASE_PATH)
        .expect("Could not connect to database")
        .create_tables()
        .expect("Could not create or verify tables");

    // If passed the `--setup-test` argument, setup the database and test it!
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--setup-test" {
        test_database();
        println!("Tests successful");
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
            Ok(_) => {}
            Err(conn_err) => println!("Connection error: {:?}", conn_err),
        };
    }
}
