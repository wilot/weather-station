use crate::mqtt_message::SensorPayload;
use rusqlite::{Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

const CREATE_SQL: &str = "CREATE TABLE weather_data (
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

const INSERT_SQL: &str = "INSERT INTO weather_data (
    MeasurementTime, ReceivedTime, TemperatureBME, TemperatureCCS811,
    TemperatureDHT22, PressureBME, HumidityBME, HumidityDHT22, eCO2CCS811,
    TVOCCCS811
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";

const CREATE_SQL_TEST: &str = "CREATE TABLE test_weather_data (
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

const INSERT_SQL_TEST: &str = "INSERT INTO test_weather_data (
    MeasurementTime, ReceivedTime, TemperatureBME, TemperatureCCS811,
    TemperatureDHT22, PressureBME, HumidityBME, HumidityDHT22, eCO2CCS811,
    TVOCCCS811
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";

/// Creates a new table called 'weather_data' in the database.
///
/// Creates `weather_data`, only if it does not already exist. Also creates a test table
/// `test_weather_data`, overwriting if it already exists.
pub fn create_tables(conn: &Connection) -> Result<()> {
    // Check whether `weather_data` exists
    let weather_table_exists = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='weather_data'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);

    // Create only if it doesn't already exist
    if !weather_table_exists {
        conn.execute(CREATE_SQL, [])?;
    }

    // Always drop (if exists) and recreate `test_weather_data`
    conn.execute("DROP TABLE IF EXISTS test_weather_data", [])?;
    conn.execute(CREATE_SQL_TEST, [])?;
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

// Inserts a dummy record into the test table
pub fn insert_test_record(conn: &Connection) -> Result<()> {
    let dummy_payload = SensorPayload::create_dummy();

    let received_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time predates unix epoch???")
        .as_secs() as i64;

    conn.execute(INSERT_SQL_TEST, dummy_payload.to_sql_tuple(received_time))?;
    Ok(())
}
