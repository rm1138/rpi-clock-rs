use crate::sensor::Sensor;
use bme280::BME280;
use linux_embedded_hal::{Delay, I2cdev};
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
pub struct BmeReading {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
}

pub struct BmeSensor {}

impl Sensor<BmeReading> for BmeSensor {
    fn init(bus: String) -> Arc<RwLock<Option<BmeReading>>> {
        let bus = I2cdev::new(bus).unwrap();
        let mut bme280 = BME280::new_primary(bus, Delay);
        let reading = Arc::new(RwLock::new(None));
        let reading_clone = reading.clone();
        bme280.init().unwrap();
        std::thread::spawn(move || loop {
            let measure = bme280.measure().unwrap();
            if let Ok(mut reading) = reading.write() {
                *reading = Some(BmeReading {
                    temperature: measure.temperature,
                    humidity: measure.humidity,
                    pressure: measure.pressure,
                });
            }
            std::thread::sleep(Duration::from_secs(60));
        });

        reading_clone
    }
}
