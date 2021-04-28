use crate::sensor::Sensor;
use apds9960::Apds9960;
use linux_embedded_hal::I2cdev;
use nb::block;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Debug)]
pub struct ApdsReading {
    light: f32,
}

impl ApdsReading {
    pub fn get_light(&self) -> f32 {
        self.light as f32
    }
}

pub struct ApdsSensor {}

fn init_and_calibrate(apds: &mut Apds9960<I2cdev>, light_sensing_freq: u8) {
    apds.enable().unwrap();
    apds.enable_light().unwrap();
    apds.set_light_integration_time(light_sensing_freq).unwrap();
}

impl Sensor<ApdsReading> for ApdsSensor {
    fn init(bus: String) -> Arc<RwLock<Option<ApdsReading>>> {
        let bus = I2cdev::new(bus).unwrap();
        let mut apds = Apds9960::new(bus);
        let reading = Arc::new(RwLock::new(None));
        let reading_clone = reading.clone();
        let light_sensing_freq: u8 = std::env::var("LIGHT_SENSING_FREQ")
            .ok()
            .and_then(|val| u8::from_str(&val).ok())
            .unwrap_or(200);

        init_and_calibrate(&mut apds, light_sensing_freq);

        std::thread::spawn(move || loop {
            let light = block!(apds.read_light()).unwrap();
            if let Ok(mut reading) = reading.write() {
                *reading = Some(ApdsReading {
                    light: light.clear as f32 / 3000 as f32,
                })
            }
            std::thread::sleep(Duration::from_millis(500))
        });

        reading_clone
    }
}
