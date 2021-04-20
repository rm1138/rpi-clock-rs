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

pub struct ApdsSensor {}

fn init_and_calibrate(apds: &mut Apds9960<I2cdev>) {
    apds.enable().unwrap();
    apds.enable_light().unwrap();
    // apds.enable_gesture().unwrap();
    // apds.enable_gesture_mode().unwrap();
    //
    // apds.enable_gesture_mode().unwrap();
    // let mut gesture_data = [0; 4];
    // block!(apds.read_gesture_data(&mut gesture_data)).unwrap();
    // apds.disable_gesture_mode().unwrap();
    //
    // apds.set_gesture_up_offset(gesture_data[0] as i8).unwrap();
    // apds.set_gesture_down_offset(gesture_data[1] as i8).unwrap();
    // apds.set_gesture_left_offset(gesture_data[2] as i8).unwrap();
    // apds.set_gesture_right_offset(gesture_data[3] as i8).unwrap();
}

impl Sensor<ApdsReading> for ApdsSensor {
    fn init(bus: String) -> Arc<RwLock<Option<ApdsReading>>> {
        let bus = I2cdev::new(bus).unwrap();
        let mut apds = Apds9960::new(bus);

        init_and_calibrate(&mut apds);
        // apds.enable_wait().unwrap();

        let reading = Arc::new(RwLock::new(None));
        let reading_clone = reading.clone();

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
