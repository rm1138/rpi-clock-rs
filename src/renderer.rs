use std::{ops::Div, str::FromStr};

use crate::bitmap::Bitmap;
use bitvec::prelude::*;
use palette::{Hsv, LinSrgb};

const ROWS: usize = 8;
const COLS: usize = 32;

pub struct ParseColorErr;

#[derive(Clone)]
#[allow(unused)]
pub enum Color {
    White,
    Black,
    Rainbow,
    RGB,
    Hsv(f32, f32),
    Raw(f32, f32, f32),
}

impl FromStr for Color {
    type Err = ParseColorErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rgb" => Ok(Color::RGB),
            "rainbow" => Ok(Color::Rainbow),
            "black" => Ok(Color::Black),
            _ => {
                let mut arr = s
                    .split(',')
                    .into_iter()
                    .flat_map(|it| it.parse::<f32>())
                    .map(|it| it / 255f32);

                let red: f32 = arr.next().ok_or(ParseColorErr)?;
                let green: f32 = arr.next().ok_or(ParseColorErr)?;
                let blue: f32 = arr.next().ok_or(ParseColorErr)?;

                Ok(Color::Raw(red, green, blue))
            }
        }
    }
}

impl Color {
    fn is_black(&self) -> bool {
        match self {
            Color::Black => true,
            _ => false,
        }
    }
    fn to_rgb(&self, brightness: f32, step: usize, x: usize, y: usize) -> (u8, u8, u8) {
        if brightness <= 0.04f32 && !self.is_black() {
            return (1, 1, 1);
        }
        let rgb: LinSrgb<f32> = match self {
            Color::White => LinSrgb::from_components((1f32, 1f32, 1f32)),
            Color::Black => LinSrgb::from_components((0f32, 0f32, 0f32)),
            Color::Rainbow => {
                let hue_grad = 64f32;
                let step = step / 3;
                let hue = (((x + step) as f32 % hue_grad) / hue_grad) * 360f32;
                let mut row_step = (y) as f32 % 16f32;
                if row_step > 8f32 {
                    row_step = 16f32 - row_step;
                }
                let sat = (row_step / 8f32) * 0.4f32 + 0.6f32; // [0.5 - 0.9]
                LinSrgb::from(Hsv::new(hue, sat, brightness))
            }
            Color::RGB => {
                let hue_grad = 64f32;
                let step = step / 5;
                let hue = (((step) as f32 % hue_grad) / hue_grad) * 360f32;
                let mut row_step = (y) as f32 % 16f32;
                if row_step > 8f32 {
                    row_step = 16f32 - row_step;
                }
                let sat = (row_step / 8f32) * 0.4f32 + 0.6f32; // [0.5 - 0.9]
                LinSrgb::from(Hsv::new(hue, sat, brightness))
            }
            Color::Hsv(hue, sat) => LinSrgb::from(Hsv::new(*hue, *sat, brightness)),
            Color::Raw(r, g, b) => LinSrgb::from_components((*r, *g, *b)),
        };
        let mut hsv: Hsv = rgb.into();
        if let Color::Black = &self {
        } else {
            hsv.value = brightness;
        }
        let rgb: LinSrgb<f32> = hsv.into();
        rgb.into_format().into_components()
    }
}

struct Pixel {
    color: Color,
}

impl Pixel {
    fn to_bytes(&self, brightness: f32, step: usize, x: usize, y: usize) -> [u8; 3] {
        let (r, g, b) = self.color.to_rgb(brightness, step, x, y);
        [g, r, b]
    }

    fn set_color(&mut self, color: &Color) {
        self.color = color.clone();
    }
}

impl Default for Pixel {
    fn default() -> Pixel {
        Pixel {
            color: Color::Black,
        }
    }
}

pub struct Frame {
    pixels: [[Pixel; 8]; 32],
    brightness: f32,
    step: usize,
}

impl Frame {
    pub fn new() -> Frame {
        let pixels: [[Pixel; 8]; 32] = Default::default();
        Frame {
            pixels,
            brightness: 0.1f32,
            step: 0,
        }
    }

    pub fn set_brightness(&mut self, brightness: f32) {
        if brightness > 0.3 {
            self.brightness = 0.3
        } else if brightness < 0.004 {
            self.brightness = 0.004
        } else {
            self.brightness = brightness
        }
    }

    pub fn clear(&mut self) {
        for col in self.pixels.iter_mut() {
            for pixel in col {
                pixel.set_color(&Color::Black);
            }
        }
    }

    fn draw_bitmap(&mut self, bitmap: &Bitmap, color: &Color, x: usize, y: usize) {
        for (col_idx, col) in bitmap.bits.iter().enumerate() {
            for (row_idx, bit) in col.iter().enumerate() {
                if *bit != 0 && col_idx + x < COLS && row_idx + y < ROWS {
                    self.pixels[col_idx + x][row_idx + y].set_color(color)
                }
            }
        }
    }

    pub fn draw_pixel(&mut self, color: &Color, x: usize, y: usize) {
        if x < COLS && y < ROWS {
            self.pixels[x][y].set_color(color)
        }
    }

    pub fn draw_text(&mut self, text: &str, color: &Color, x: usize, y: usize) {
        let mut x_offset = x;
        for char in text.chars() {
            let bitmap = Bitmap::from_char(char);
            self.draw_bitmap(&bitmap, color, x_offset, y);
            x_offset += bitmap.bits.len() + 1;
        }
    }

    fn to_bytes(&self) -> [u8; 3 * ROWS * COLS] {
        let mut result = [0u8; 3 * ROWS * COLS];
        let mut idx = 0;
        for (col_idx, col) in self.pixels.iter().enumerate() {
            let mut row_idx: usize = if col_idx % 2 == 1 { 7 } else { 0 };
            let apply = |pixel: &Pixel| {
                for channel in pixel
                    .to_bytes(self.brightness, self.step, col_idx, row_idx)
                    .iter()
                {
                    result[idx] = *channel;
                    idx += 1;
                }
                if col_idx % 2 == 1 {
                    row_idx -= 1;
                } else {
                    row_idx += 1;
                }
            };

            if col_idx % 2 == 1 {
                col.iter().rev().for_each(apply);
            } else {
                col.iter().for_each(apply);
            }
        }
        result
    }

    pub fn get_spi_data(&mut self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        let bytes = self.to_bytes();
        let bits = bytes.view_bits::<Msb0>();

        for bit in bits.iter() {
            if *bit {
                result.push(0b1111_1111);
                result.push(0b1111_0000);
            } else {
                result.push(0b1111_0000);
                result.push(0b0000_0000);
            }
        }

        self.step += 1;
        result
    }
}
