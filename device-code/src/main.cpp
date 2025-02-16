#include <Arduino.h>

#include <Wire.h>
#include <SPI.h>
#include <ESP8266WiFi.h>
#include <time.h>
#include <coredecls.h>

#include <Adafruit_BME280.h>
#include <Adafruit_CCS811.h>
#include <DHT.h>
#include <DHT_U.h>

#include <PubSubClient.h>


const char* STASSID = "";
const char* STAPSK = "";
const char* NTP_SERVER = "uk.pool.ntp.org";
const char* NTP_TZ = "BST0GMT,M3.2.0/2:00:00,M11.1.0/2:00:00";
const char* MQTT_SERVER = "rpi.local";

//time_t now;
//tm time_struct;

Adafruit_BME280 bme;  // Use I2C interface
//Adafruit_BME280_Humidity bme_humidity(&bme);  // Unified Sensor way of accessing humidity
Adafruit_CCS811 ccs;
DHT_Unified dht(4, DHT22);  // Use GPIO pin 4

WiFiClient mqttSocket;
PubSubClient mqttClient(mqttSocket);
String mqttCLientId = "WeatherStation";

typedef struct SensorPayload {
  float bmeTemperature;
  float bmePressure;
  float bmeHumidity;
  float ccs811Temperature;
  uint16 ccs811eCO2;  // in ppm
  uint16 ccs811TVOC;  // in ppb
  float DHT22Temperature;
  float DHT22Humidity;
};

// Set the update timer for NTP
uint32_t sntp_update_delay_MS_rfc_not_less_than_15000 () {
  return 5 * 60 * 1000UL; // 5 min
}

// Print the time
void print_time() {
  time_t now;
  tm time_struct;
  time(&now);                       // read the current time
  localtime_r(&now, &time_struct);           // update the structure tm with the current time
  Serial.print("year:");
  Serial.print(time_struct.tm_year + 1900);  // years since 1900
  Serial.print("\tmonth:");
  Serial.print(time_struct.tm_mon + 1);      // January = 0 (!)
  Serial.print("\tday:");
  Serial.print(time_struct.tm_mday);         // day of month
  Serial.print("\thour:");
  Serial.print(time_struct.tm_hour);         // hours since midnight  0-23
  Serial.print("\tmin:");
  Serial.print(time_struct.tm_min);          // minutes after the hour  0-59
  Serial.print("\tsec:");
  Serial.print(time_struct.tm_sec);          // seconds after the minute  0-61*
  Serial.print("\twday");
  Serial.print(time_struct.tm_wday);         // days since Sunday 0-6
  if (time_struct.tm_isdst == 1)             // Daylight Saving Time flag
    Serial.print("\tDST");
  else
    Serial.print("\tstandard");
  Serial.println();
}

// Print if the time was set (a callback)
void time_set(bool from_sntp){
  Serial.print(F("Time was set from_sntp=")); Serial.println(from_sntp);
}


/*!
 * @brief Takes measurements from every sensor and prints to serial
*/
void measure_print_sensors() {

  // measure & print BME280 sensor values
  Serial.print("Temperature = ");
  Serial.print(bme.readTemperature());
  Serial.println(" °C");
  Serial.print("Pressure = ");
  Serial.print(bme.readPressure() / 100.0F);
  Serial.println(" hPa");
  Serial.print("Humidity = ");
  Serial.print(bme.readHumidity());
  Serial.println(" %");

  // measure & print CCS811 sensor balues
  float temp = ccs.calculateTemperature();
  if(!ccs.readData()){
    float eCO2 = ccs.geteCO2();
    float TVOC = ccs.getTVOC();

    Serial.print("eCO2: ");
    Serial.print(eCO2);
    Serial.print(" ppm, TVOC: ");
    Serial.print(TVOC);
    Serial.print(" ppb, Temp: ");
    Serial.println(temp);
  } else {
    Serial.println("Error reading measurements from CCS811");
  }

  sensors_event_t dht_event;
  dht.temperature().getEvent(&dht_event);
  if (isnan(dht_event.temperature)) {
    Serial.println(F("Error reading temperature!"));
  }
  else {
    Serial.print(F("Temperature: "));
    Serial.print(dht_event.temperature);
    Serial.println(F("°C"));
  }
  // Get humidity event and print its value.
  dht.humidity().getEvent(&dht_event);
  if (isnan(dht_event.relative_humidity)) {
    Serial.println(F("Error reading humidity!"));
  }
  else {
    Serial.print(F("Humidity: "));
    Serial.print(dht_event.relative_humidity);
    Serial.println(F("%"));
  }
}


bool connectMQTT() {
  Serial.print("Connecting to MQTT server... ");
  if (mqttClient.connect(mqttCLientId.c_str())) {
    Serial.println("connected.");
  } else {
    Serial.print("connection failed; rc=");
    Serial.println(mqttClient.state());
  }
  return mqttClient.connected();
}


/*!
 * @brief Sets up Serial, NTP, WiFi, randomSeed and connects to MQTT
 * @returns True on success, false otherwise
 */
bool initialise_esp() {

  // Initialise Serial & LED
  pinMode(LED_BUILTIN, OUTPUT);
  Serial.begin(115200);
  Serial.println("NTP Attempt");

  // Initialise NTP
  settimeofday_cb(time_set);
  configTime(NTP_TZ, NTP_SERVER);
  yield();  // settimeofday_cb is always deferred to next yield/delay, it's not immediate!

  // Initialise WiFi
  WiFi.persistent(false);
  WiFi.mode(WIFI_STA);
  WiFi.begin(STASSID, STAPSK);
  while (WiFi.status() != WL_CONNECTED){
    delay(200);
    Serial.print(".");
  }
  Serial.println("\nWiFi connected\n");

  randomSeed(micros());  // Use the time to connect as a source of entropy

  return connectMQTT();
}


/*!
 * @brief Initialises each sensor, printing debug to Serial
 * @returns True if successful
*/
bool initialise_sensors() {

  // initialise BMP280
  if (!bme.begin(BME280_ADDRESS_ALTERNATE)) {
    Serial.println("Could not find BME280 sensor!");
    return false;
  }
  Serial.println("Using BME280 weather station presets");
  Serial.println("forced mode, 1x temperature / 1x humidity / 1x pressure oversampling,");
  Serial.println("filter off");
  bme.setSampling(
    Adafruit_BME280::MODE_FORCED,
    Adafruit_BME280::SAMPLING_X1, // temperature
    Adafruit_BME280::SAMPLING_X1, // pressure
    Adafruit_BME280::SAMPLING_X1, // humidity
    Adafruit_BME280::FILTER_OFF
  );
  Serial.println();

  // initialise the DHT22
  dht.begin();
  Serial.println("Initialised DHT22");
  sensors_event_t dht_temperature;
  dht.temperature().getEvent(&dht_temperature);

  // initialise the CCS811
  if (!ccs.begin(CCS811_ADDRESS)){
    Serial.println("Failed to start CCS811 sensor!");
    return false;
  }
  while(!ccs.available());  // Wait
  float measured_temperature = ccs.calculateTemperature();
  ccs.setTempOffset(measured_temperature - dht_temperature.temperature);  // Calibrate CCS811 with current temperature from DHT22

  return true;
}


/*! 
 * @brief Reads each sensor and populates a SensorPayload
 * @returns SensorPayload A struct containing all the sensor measurements
*/
SensorPayload measure_sensors() {

  SensorPayload payload = {};

  // BME measurement
  bme.takeForcedMeasurement();
  payload.bmeTemperature = bme.readTemperature();  // in °C
  payload.bmeHumidity = bme.readHumidity();  // in %
  payload.bmePressure = bme.readPressure();  // in Pa

  // CCS811 measurement
  payload.ccs811Temperature = ccs.calculateTemperature();
  if (!ccs.readData()) {
    payload.ccs811eCO2 = ccs.geteCO2();
    payload.ccs811TVOC = ccs.getTVOC();
  }

  // DHT22 measurement
  sensors_event_t dht_measurement;
  dht.temperature().getEvent(&dht_measurement);
  if(!isnan(dht_measurement.temperature)) {
    payload.DHT22Temperature = dht_measurement.temperature;
  }
  dht.humidity().getEvent(&dht_measurement);
  if(!isnan(dht_measurement.relative_humidity)) {
    payload.DHT22Humidity = dht_measurement.relative_humidity;
  }

  return payload;
}


void setup() {
  initialise_esp();
  if (!initialise_sensors()) {
    Serial.println(F("ERROR INITIALISING SENSORS"));
    while(true);
  } 
}

void loop() {

  digitalWrite(LED_BUILTIN, !digitalRead(LED_BUILTIN));  // Flip the LED

  Serial.println("Estimated time:");
  print_time();

  measure_print_sensors();

  SensorPayload sensor_values = measure_sensors();
  if (!mqttClient.connected()) {
    Serial.println("MQTT client not connected");
    connectMQTT();  // Just assumeit worked this time I guess...
  }
  mqttClient.publish("TEST", (byte*)&sensor_values, sizeof(sensor_values));

  Serial.println(F("\n------------------------------------\n"));
  mqttClient.loop();
  delay(10000);
}
