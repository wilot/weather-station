# Weather Station Interfaces

Defines the common interfaces for communication between the Raspberry Pi and the ESP8266. They communicate over MQTT
sending the C struct below:

```
typedef struct __attribute__((packed)) sensor_message_header_t {
  uint32_t magic_value;
} SensorMessageHeader;

typedef struct __attribute__((packed)) sensor_payload_t {
  float bmeTemperature;
  float bmePressure;
  float bmeHumidity;
  float ccs811Temperature;
  uint16_t ccs811eCO2;  // in ppm
  uint16_t ccs811TVOC;  // in ppb
  float DHT22Temperature;
  float DHT22Humidity;
} SensorPayload;

typedef struct __attribute__((packed)) message_t {
  SensorMessageHeader header;
  SensorPayload payload;
} SensorMessage;
```

The SQLite database contains a single table, WeatherStation, which contains

| MeasurementTime | ReceivedTime | TemperatureBME | TemperatureCCS811 | TemperatureDHT22 | PressureBME | HumidityBME | HumidityDHT22 | eCO2CCS811 | TVOCCCS811 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| INTEGER (POSIX time) | INTEGER | INTEGER (*0.1˚C) | INTEGER | INTEGER | INTEGER (Pascal) | INTEGER (%) | INTEGER (%) | INTEGER (ppm) | INTEGER (ppb) |

