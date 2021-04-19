use std::time::{Duration, Instant};
use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use crate::renderer::Frame;

mod renderer;

fn main() {
    println!("Started");

    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 15_600_000, Mode::Mode0).unwrap();
    let frame = Frame::new();
    loop {
        spi.write(&frame.get_spi_data());
        let now = Instant::now();
        while Instant::now().duration_since(now) < Duration::from_millis(2) {}
    }
}
