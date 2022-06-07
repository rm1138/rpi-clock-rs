use crate::renderer::{Color, Frame};
use crate::sensor::Sensor;
use chrono::Local;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::time::SystemTime;

mod bitmap;
mod mqtt;
mod renderer;
mod sensor;

struct RenderState {
    state: State,
    remain_tick: usize,
    temperature: Option<f32>,
    humidity: Option<f32>,
    brightness: f32,
    last_update: Option<SystemTime>,
    color: Color,
}

enum State {
    Clock,
    Date,
    Temperature,
    Humidity,
    Empty,
}

impl RenderState {
    fn init() -> RenderState {
        RenderState {
            state: State::Empty,
            remain_tick: 0,
            temperature: None,
            humidity: None,
            brightness: 0.1f32,
            last_update: None,
            color: Color::RGB,
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
            State::Clock => (State::Date, 1),
            State::Date => {
                if self.is_temperature_humidity_stale() {
                    (State::Clock, 5)
                } else {
                    (State::Temperature, 1)
                }
            }
            State::Temperature => {
                if self.is_temperature_humidity_stale() {
                    (State::Clock, 5)
                } else {
                    (State::Humidity, 1)
                }
            }
            State::Humidity => (State::Clock, 15),
            State::Empty => (State::Clock, 10),
        };

        self.state = next_state;
        self.remain_tick = next_tick_remain;
    }

    fn set_temperature(&mut self, value: &str) {
        if let Ok(value) = value.parse() {
            self.temperature = Some(value);
            self.last_update = Some(SystemTime::now());
        }
    }

    fn set_humidity(&mut self, value: &str) {
        if let Ok(value) = value.parse() {
            self.humidity = Some(value);
            self.last_update = Some(SystemTime::now());
        }
    }

    fn set_color(&mut self, value: &str) {
        if let Ok(value) = value.parse() {
            self.color = value;
            self.last_update = Some(SystemTime::now());
        }
    }

    fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness;
    }

    fn get_state(&self) -> &State {
        &self.state
    }

    fn get_render_text(&self) -> String {
        match self.state {
            State::Clock => RenderState::format_time(),
            State::Date => RenderState::format_date(),
            State::Temperature => self
                .temperature
                .as_ref()
                .and_then(|value| Some(format!("{:.2}c", value)))
                .unwrap_or("".to_string()),
            State::Humidity => self
                .humidity
                .as_ref()
                .and_then(|value| Some(format!("{:.2}%", value)))
                .unwrap_or("".to_string()),
            State::Empty => "".to_string(),
        }
    }

    fn is_temperature_humidity_stale(&self) -> bool {
        match self.last_update {
            Some(last_update) => last_update.elapsed().unwrap().as_secs() > 300,
            None => true,
        }
    }

    fn is_temperature_humidity_just_updated(&self) -> bool {
        match self.last_update {
            Some(last_update) => last_update.elapsed().unwrap().as_secs() < 1,
            None => false,
        }
    }
}

fn main() {
    println!("Started");
    let state = Arc::new(RwLock::new(RenderState::init()));
    let state_read = state.clone();
    let adps_reading = sensor::apds_9960::ApdsSensor::init("/dev/i2c-1".to_string());

    if let Ok(mut mqtt) = mqtt::Mqtt::connect() {
        let state_mqtt = state.clone();
        // move the mqtt to new thread to prevent it to be dropped
        std::thread::spawn(move || {
            let mqtt_channel = mqtt.consume();
            mqtt.subscribe("temperature");
            mqtt.subscribe("humidity");
            mqtt.subscribe("color");

            mqtt_channel.iter().for_each(|msg| {
                if let Some(msg) = msg {
                    let topic = msg.topic();
                    if let Ok(mut state) = state_mqtt.write() {
                        if topic.contains("temperature") {
                            (*state).set_temperature(&msg.payload_str());
                        } else if topic.contains("humidity") {
                            (*state).set_humidity(&msg.payload_str());
                        } else if topic.contains("color") {
                            (*state).set_color(&msg.payload_str());
                        }
                    }
                } else {
                    println!("None message");
                    mqtt.reconnect();
                    mqtt.subscribe("temperature");
                    mqtt.subscribe("humidity");
                }
            })
        });
    };

    std::thread::spawn(move || loop {
        if let Ok(mut state) = state.write() {
            (*state).next();
            if let Ok(reading) = adps_reading.read() {
                if let Some(reading) = reading.deref() {
                    (*state).set_brightness(reading.get_light())
                }
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    });

    std::thread::sleep(Duration::from_secs(1));

    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 15_600_000, Mode::Mode0).unwrap();
    let mut frame = Frame::new();
    loop {
        frame.clear();
        if let Ok(state) = state_read.read() {
            frame.set_brightness(state.brightness);
            match state.get_state() {
                State::Clock => frame.draw_text(&state.get_render_text(), &state.color, 2, 1),
                State::Date => frame.draw_text(&state.get_render_text(), &state.color, 1, 1),
                State::Temperature => frame.draw_text(&state.get_render_text(), &state.color, 6, 1),
                State::Humidity => frame.draw_text(&state.get_render_text(), &state.color, 6, 1),
                _ => {}
            }

            if state.is_temperature_humidity_stale() {
                frame.draw_pixel(&Color::Raw(64f32, 0f32, 0f32), 1, 7);
            }

            if state.is_temperature_humidity_just_updated() {
                frame.draw_pixel(&Color::RGB, 30, 7);
            }
        }
        spi.write(&frame.get_spi_data()).unwrap();
        std::thread::sleep(Duration::from_micros(1_000_000 / 60));
    }
}
