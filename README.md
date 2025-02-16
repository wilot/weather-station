# Weather Station Project

A weather station project that measures outside air temperature, pressure etc. using an ESP8266. Weather records are
then uploaded to a server (Raspberry Pi 1B) using MQTT protocol. The server, using Mosquitto MQTT server, parses the
received reports and saves them to a SQLite database. This is then presented with a local instance of Graphana. The
current weather is also presented on an 
[LCD display](https://learn.adafruit.com/adafruit-16x2-character-lcd-plus-keypad-for-raspberry-pi/overview) 
attached to the Pi's GPIO pins. 

## Project Contents

- `device-code`: A PlatformIO project using the ESP8266 arduino core
- `lcd-controller`: A library to control an LCD display. A rewrite of Adafruit's Python 
[library](https://github.com/adafruit/Adafruit_CircuitPython_CharLCD) in Rust, just for fun.
- `message-parser`: Parses received MQTT messages and saves to the SQLite database. Runs on the Pi.
