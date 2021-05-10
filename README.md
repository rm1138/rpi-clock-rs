# rpi-clock-rs

This project is using the LED panel for awtrix and a raspberry pi zero w to build a "smart clock" with the following features.

- Display room temperature
- Display room humidity
- Display room pressure
- Logging the data above to a remote MQTT server.
- Auto brightness

## Hardware

- WS2812B LED Matrix 32x8
- BME280 (temperature, humidity and pressure sensor)
- APSD9960 (Light and gesture sensor)
- Raspberry PI Zero W

## Wiring

- WS2812B

  This led panel has 3pin (5V, GND, DATA), the 5V and GND can connect to the PI directly or connect to an external 5V power supply.
  The data pin connect to the MOSI (PIN 19) pin.

- BME280

  Connect the SDA to PIN 3, CLK to PIN 5, 5V and GND can connect to the PI directly.

- APSD9960

  The SDA, CLK and GND pins can follow the BME280, the 3.3V connects to the 3.3V pin of the PI.
  
## Raspberry PI Config

- Enable I2C
- Enable SPI
- Increase the I2C buffer

## Demo

[![Demo](https://img.youtube.com/vi/e_vwJALaTAY/0.jpg)](https://www.youtube.com/watch?v=e_vwJALaTAY)
