use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::{Connection, Result};
use crate::mqtt_message::SensorPayload;


const CREATE_SQL: &str = 
"CREATE TABLE weather_data (
MeasurementTime INTEGER,
ReceivedTime INTEGER,
TemperatureBME INTEGER,
TemperatureCCS811 INTEGER,
TemperatureDHT22 INTEGER,
PressureBME INTEGER,
HumidityBME INTEGER,
HumidityDHT22 INTEGER,
eCO2CCS811 INTEGER,
TVOCCCS811 INTEGER
)";

const INSERT_SQL: &str =
"INSERT INTO weather_data (
    MeasurementTime, ReceivedTime, TemperatureBME, TemperatureCCS811,
    TemperatureDHT22, PressureBME, HumidityBME, HumidityDHT22, eCO2CCS811,
    TVOCCCS811
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";


/// Creates a new table called 'weather_data' in the database.
pub fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(CREATE_SQL, ())?;
    Ok(())
}

/// Inserts sensor data into the 'weather_data' table.
///
/// Reformats the SensorPayload and adds the current POSIX time.
pub fn insert_sensor_data(conn: &Connection, payload: &SensorPayload) -> Result<()> {
    let received_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time predates unix epoch???")
        .as_secs() as i64;

    conn.execute(INSERT_SQL, payload.to_sql_tuple(received_time))?;
    Ok(())
}
