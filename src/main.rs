use core::ptr::read;
use std::path::Path;
use rusqlite::{Connection, Result};
use crate::mqtt_message::SensorPayload;
use crate::database::{insert_sensor_data, create_table};

mod mqtt_message;
mod database;

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
}
