use crate::mqtt_message::SensorMessagePayload;
use rusqlite::{types::Value, Connection, Result};
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

const SELECT_SQL_TEST: &str = "SELECT * FROM test_weather_data";

pub struct WeatherDatabase {
    conn: Connection,
}

impl WeatherDatabase {
    /// Establishes connection to the database, or creates it if necessary
    pub fn new(database_path: &str) -> Result<Self> {
        let conn = Connection::open(database_path)?;
        Ok(Self { conn })
    }

    /// Creates a new table called 'weather_data' in the database.
    ///
    /// Creates `weather_data`, only if it does not already exist. Also creates a test table
    /// `test_weather_data`, overwriting if it already exists.
    pub fn create_tables(&self) -> Result<()> {
        // Check whether `weather_data` exists
        let weather_table_exists = self
            .conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='weather_data'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);

        // Create only if it doesn't already exist
        if !weather_table_exists {
            println!("No weather_data table found, creating");
            self.conn.execute(CREATE_SQL, [])?;
        }

        // Always drop (if exists) and recreate `test_weather_data`
        self.conn
            .execute("DROP TABLE IF EXISTS test_weather_data", [])?;
        self.conn.execute(CREATE_SQL_TEST, [])?;
        Ok(())
    }

    /// Inserts sensor data into the 'weather_data' table.
    ///
    /// Reformats the SensorPayload and adds the current POSIX time.
    pub fn insert_sensor_data(&self, payload: &SensorMessagePayload) -> Result<()> {
        let received_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time predates unix epoch???")
            .as_secs() as i64;

        self.conn
            .execute(INSERT_SQL, payload.to_sql_tuple(received_time))?;
        Ok(())
    }

    /// Inserts a dummy payload, prints the result
    pub fn test_sqlite(&self) -> Result<()> {
        let dummy_payload = SensorMessagePayload::create_dummy();

        let received_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time predates unix epoch???")
            .as_secs() as i64;

        self.conn
            .execute(INSERT_SQL_TEST, dummy_payload.to_sql_tuple(received_time))?;

        let mut stmt = self.conn.prepare(SELECT_SQL_TEST)?;
        let col_count = stmt.column_count();
        let rows = stmt.query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..col_count {
                let value: Value = row.get(i)?;
                values.push(value);
            }
            Ok(values)
        })?;
        for row in rows {
            let values = row?;
            let row_str: Vec<String> = values.iter().map(|v| format!("{:?}", v)).collect();
            println!("{}", row_str.join(" | "));
        }
        Ok(())
    }
}
