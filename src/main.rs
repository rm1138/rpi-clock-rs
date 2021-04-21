use crate::renderer::{Color, Frame};
use crate::sensor::bme_280::BmeReading;
use crate::sensor::Sensor;
use crate::util::spin_wait;
use chrono::Local;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::sync::{Arc, RwLock};
use std::time::Duration;

mod bitmap;
mod renderer;
mod sensor;
mod util;

struct RenderState {
    state: State,
    remain_tick: usize,
    bme_reading: Option<BmeReading>,
}

enum State {
    Clock,
    Date,
    Temperature,
    Humidity,
    Pressure,
    Empty,
}

impl RenderState {
    fn init() -> RenderState {
        RenderState {
            state: State::Empty,
            remain_tick: 0,
            bme_reading: None,
        }
    }

    fn format_time() -> String {
        let now = Local::now();
        if now.timestamp_subsec_millis() < 500 {
            now.format("%H:%M:%S").to_string()
        } else {
            now.format("%H %M %S").to_string()
        }
    }

    fn format_date() -> String {
        let now = Local::now();
        now.format("%m-%d_%a").to_string()
    }

    fn next(&mut self) {
        if self.remain_tick > 0 {
            self.remain_tick -= 1;
            return;
        }

        let (next_state, next_tick_remain) = match self.state {
            State::Clock => (State::Date, 2),
            State::Date => (State::Temperature, 1),
            State::Temperature => (State::Humidity, 1),
            State::Humidity => (State::Pressure, 1),
            State::Pressure | State::Empty => (State::Clock, 8),
        };

        self.state = next_state;
        self.remain_tick = next_tick_remain;
    }

    fn set_bme_reading(&mut self, bme_reading: Option<BmeReading>) {
        self.bme_reading = bme_reading;
    }

    fn get_state(&self) -> &State {
        &self.state
    }

    fn get_render_text(&self) -> String {
        match self.state {
            State::Clock => RenderState::format_time(),
            State::Date => RenderState::format_date(),
            State::Temperature => self
                .bme_reading
                .as_ref()
                .and_then(|bme| Some(format!("{:.2}c", bme.temperature)))
                .unwrap_or("".to_string()),
            State::Humidity => self
                .bme_reading
                .as_ref()
                .and_then(|bme| Some(format!("{:.2}%", bme.humidity)))
                .unwrap_or("".to_string()),
            State::Pressure => self
                .bme_reading
                .as_ref()
                .and_then(|bme| Some(format!("{:.3}ATM", bme.pressure / 101325f32)))
                .unwrap_or("".to_string()),
            State::Empty => "".to_string(),
        }
    }
}

fn main() {
    println!("Started");
    let bme_reading = sensor::bme_280::BmeSensor::init("/dev/i2c-1".to_string());
    let adps_reading = sensor::apds_9960::ApdsSensor::init("/dev/i2c-1".to_string());
    let state = Arc::new(RwLock::new(RenderState::init()));
    let state_read = state.clone();

    std::thread::spawn(move || loop {
        if let Ok(mut state) = state.write() {
            (*state).next();
            if let Ok(bme_reading) = bme_reading.read() {
                (*state).set_bme_reading(*bme_reading);
            }
        }
        std::thread::sleep(Duration::from_millis(1000));
    });

    std::thread::sleep(Duration::from_secs(1));

    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 15_600_000, Mode::Mode0).unwrap();
    let mut frame = Frame::new();
    loop {
        frame.clear();
        if let Ok(state) = state_read.read() {
            match state.get_state() {
                State::Clock => frame.draw_text(&state.get_render_text(), &Color::White, 2, 1),
                State::Date => frame.draw_text(&state.get_render_text(), &Color::White, 1, 1),
                State::Temperature => {
                    frame.draw_text(&state.get_render_text(), &Color::White, 6, 1)
                }
                State::Humidity => frame.draw_text(&state.get_render_text(), &Color::White, 6, 1),
                State::Pressure => frame.draw_text(&state.get_render_text(), &Color::White, 1, 1),
                _ => {}
            }
        }
        spi.write(&frame.get_spi_data()).unwrap();
        spin_wait(2);
    }
}
