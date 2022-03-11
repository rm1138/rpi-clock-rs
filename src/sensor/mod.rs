pub mod apds_9960;

use std::sync::{Arc, RwLock};

pub trait Sensor<Reading> {
    fn init(bus: String) -> Arc<RwLock<Option<Reading>>>;
}
