# Weather Station Interfaces

Defines the common interfaces for communication between the Raspberry Pi and the ESP8266. They communicate over MQTT
sending the C struct below:

```
typedef struct SensorPayload {
    time_t posix_time;  // signed long long
    float bmeTemperature;
    float bmePressure;
    float bmeHumidity;
    float ccs811Temperature;
    uint16 ccs811eCO2;  // in ppm
    uint16 ccs811TVOC;  // in ppb
    float DHT22Temperature;
    float DHT22Humidity;
};
```

The SQLite database contains a single table, WeatherStation, which contains

| MeasurementTime | ReceivedTime | TemperatureBME | TemperatureCCS811 | TemperatureDHT22 | PressureBME | HumidityBME | HumidityDHT22 | eCO2CCS811 | TVOCCCS811 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| INTEGER (POSIX time) | INTEGER | INTEGER (*0.1˚C) | INTEGER | INTEGER | INTEGER (Pascal) | INTEGER (%) | INTEGER (%) | INTEGER (ppm) | INTEGER (ppb) |

