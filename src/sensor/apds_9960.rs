use crate::sensor::Sensor;
use apds9960::Apds9960;
use linux_embedded_hal::I2cdev;
use nb::block;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Debug)]
pub enum Gesture {
    Left,
    Right,
}
#[derive(Debug)]
pub struct ApdsReading {
    light: u16,
    gesture: Option<Gesture>,
}

impl ApdsReading {
    pub fn get_light(&self) -> f32 {
        self.light as f32
    }
}

pub struct ApdsSensor {}

fn init_and_calibrate(apds: &mut Apds9960<I2cdev>) {
    apds.enable().unwrap();
    apds.enable_light().unwrap();
    apds.set_light_integration_time(128).unwrap();
}

impl Sensor<ApdsReading> for ApdsSensor {
    fn init(bus: String) -> Arc<RwLock<Option<ApdsReading>>> {
        let bus = I2cdev::new(bus).unwrap();
        let mut apds = Apds9960::new(bus);
        let reading = Arc::new(RwLock::new(None));
        let reading_clone = reading.clone();

        init_and_calibrate(&mut apds);

        std::thread::spawn(move || loop {
            let light = block!(apds.read_light()).unwrap();
            if let Ok(mut reading) = reading.write() {
                *reading = Some(ApdsReading {
                    light: light.clear,
                    gesture: None,
                })
            }
            std::thread::sleep(Duration::from_secs(1))
        });

        reading_clone
    }
}
