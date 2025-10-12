use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[allow(non_snake_case)]
struct SensorMessageHeader {
    magic_number: u32,
}

#[allow(non_snake_case)]
pub struct SensorMessagePayload {
    posix_time: i64,
    bme_temperature: f32,
    bme_pressure: f32,
    bme_humidity: f32,
    sgp30_eCO2: u16,
    sgp30_TVOC: u16,
    dht22_temperature: f32,
    dht22_humidity: f32,
}

#[allow(non_snake_case)]
pub struct SensorMessage {
    header: SensorMessageHeader,
    pub payload: SensorMessagePayload,
}

impl SensorMessage {
    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Validate data size first

        let mut cursor = Cursor::new(data);
        let message = SensorMessage {
            header: SensorMessageHeader {
                magic_number: cursor.read_u32::<LittleEndian>()?,
            },
            payload: SensorMessagePayload {
                posix_time: cursor.read_i64::<LittleEndian>()?,
                bme_temperature: cursor.read_f32::<LittleEndian>()?,
                bme_pressure: cursor.read_f32::<LittleEndian>()?,
                bme_humidity: cursor.read_f32::<LittleEndian>()?,
                sgp30_eCO2: cursor.read_u16::<LittleEndian>()?,
                sgp30_TVOC: cursor.read_u16::<LittleEndian>()?,
                dht22_temperature: cursor.read_f32::<LittleEndian>()?,
                dht22_humidity: cursor.read_f32::<LittleEndian>()?,
            },
        };

        match message.header.magic_number {
            0x12345678 => Ok(message),
            0x78563412 => panic!("Magic number error, looks like wrong endiness: 0x78563412"),
            val => panic!("Magic number error: {:#06x}", val),
        }
    }
}

impl SensorMessagePayload {
    pub fn to_sql_tuple(
        &self,
        received_time: i64,
    ) -> (i64, i64, i32, i32, i32, i32, i32, i32, i32) {
        (
            self.posix_time,
            received_time,
            (self.bme_temperature * 10.0) as i32,
            (self.dht22_temperature * 10.0) as i32,
            self.bme_pressure as i32,
            (self.bme_humidity * 100.0) as i32,
            (self.dht22_humidity * 100.0) as i32,
            self.sgp30_eCO2 as i32,
            self.sgp30_TVOC as i32,
        )
    }

    pub const fn create_dummy() -> Self {
        Self {
            posix_time: 1742069972,
            bme_temperature: 100f32,
            bme_pressure: 101325f32,
            bme_humidity: 20f32,
            sgp30_eCO2: 450,
            sgp30_TVOC: 25,
            dht22_temperature: 95f32,
            dht22_humidity: 20f32,
        }
    }
}
