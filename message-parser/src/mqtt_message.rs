use core::ffi;


#[repr(C)]
#[allow(non_snake_case)]
pub struct SensorPayload {
    posix_time: ffi::c_longlong,
    bme_temperature: ffi::c_float,
    bme_pressure: ffi::c_float,
    bme_humidity: ffi::c_float,
    ccs811_temperature: ffi::c_float,
    ccs811_eCO2: ffi::c_ushort,
    ccs811_TVOC: ffi::c_ushort,
    dht22_temperature: ffi::c_float,
    dht22_humidity: ffi::c_float,
}

impl SensorPayload {
    pub fn to_sql_tuple(&self, received_time: i64)
    -> (i64, i64, i32, i32, i32, i32, i32, i32, i32, i32) {
        (
            self.posix_time,
            received_time,
            (self.bme_temperature * 10.0) as i32,
            (self.ccs811_temperature * 10.0) as i32,
            (self.dht22_temperature * 10.0) as i32,
            self.bme_pressure as i32,
            (self.bme_humidity * 100.0) as i32,
            (self.dht22_humidity * 100.0) as i32,
            self.ccs811_eCO2 as i32,
            self.ccs811_TVOC as i32,
        )
    }
}

