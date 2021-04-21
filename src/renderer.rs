use crate::bitmap::Bitmap;
use bitvec::prelude::*;

const ROWS: usize = 8;
const COLS: usize = 32;

#[derive(Clone)]
pub enum Color {
    White,
    Black,
    Raw(u8, u8, u8),
}

impl Color {
    fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::White => (12, 12, 12),
            Color::Black => (0, 0, 0),
            Color::Raw(r, g, b) => (*r, *g, *b),
        }
    }
}

struct Pixel {
    color: Color,
}

impl Pixel {
    fn to_bytes(&self) -> [u8; 3] {
        let (r, g, b) = self.color.to_rgb();
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
}

impl Frame {
    pub fn new() -> Frame {
        let pixels: [[Pixel; 8]; 32] = Default::default();
        Frame { pixels }
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
            let apply = |pixel: &Pixel| {
                for channel in pixel.to_bytes().iter() {
                    result[idx] = *channel;
                    idx += 1;
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

    pub fn get_spi_data(&self) -> Vec<u8> {
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

        for _ in 1..1000 {
            result.push(0b0000_0000);
        }
        result
    }
}
