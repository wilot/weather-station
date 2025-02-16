# ESP8266 Weather Station Project

Have a battery powered weather station that logs sensor measurements to a server over WiFi every 5mins. Use NTP to get
the time. It will be battery powered so to conserve battery, it should sleep in-between measurements. To further save
power, it should log its measurements to memory and only push them over WiFi every hour or so. While it does this, it
can also re-sync the NTP time, using system clock otherwise.

Will need to use the RTC memory or my own EEPROM in order to store data because the Flash would wear out to fast (only
10k writes vs 100k). RTC memory has 512 bytes (in 4-byte blocks)

All devices run at 3V3, and all use I2C except for the DHT22 which uses a custom interface optimised for longer wires.

NTP server can be found [here](https://www.ntppool.org/en/zone/uk). Maximum ESP8266 deep sleep is abour 3hr20min.

## ESP8266 Board and Connected Sensors

- SparkFun ESP8266 Thing
- DHT22 - temperature & humidity
- BME280 - pressure & humidity
- CCS811 - air quality (tVOC & eCO2)

## Anticipated Current Budget

- ESP8266 running - 70mA
- ESP9266 WiFi tx - <300mA (needs >330uF to smooth WiFi ripples)
- ESP8266 modem sleep - 15mA (may need WiFi.forceSleepBegin();)
- ESP8266 deep sleep - 10uA (may need a 100ms delay after calling deep sleep)
- DHT22 - 1.5mA measuring, 45uA standby (needs 2s to measure)
- BMP280 - 3.6uA, 0.1uA in sleep mode
- CCS811 - 30mA (minimum measurement every 60s)