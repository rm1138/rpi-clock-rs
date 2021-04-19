use bitvec::prelude::*;

const ROWS: usize = 8;
const COLS: usize = 32;

enum Color {
    White,
    Black,
    Raw(u8, u8, u8),
}

impl Color {
    fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::White => (255, 255, 255),
            Color::Black => (20, 0, 0),
            Color::Raw(r, g, b) => (*r, *g, *b)
        }
    }
}

struct Bitmap {
    bits: Vec<[bool; 8]>
}

impl Bitmap {
    fn from_char(char: char) -> Bitmap {
        let bits: Vec<[bool; 8]> = match char {
            '1' => {
                vec![]
            }
            _ => {
                vec![]
            }
        };
        Bitmap { bits }
    }
}

struct Pixel {
    color: Color
}

impl Pixel {
    fn to_bytes(&self) -> [u8; 3] {
        let (r, g, b) = self.color.to_rgb();
        [g, r, b]
    }
}

impl Default for Pixel {
    fn default() -> Pixel {
        Pixel {
            color: Color::Black
        }
    }
}

pub struct Frame {
    pixels: [[Pixel; 8]; 32]
}

impl Frame {
    pub fn new() -> Frame {
        let pixels: [[Pixel; 8]; 32] = Default::default();
        Frame {
            pixels
        }
    }

    fn draw_bitmap(&mut self, bitmap: &Box<Bitmap>, color: &Color, x: i32, y: i32) {}

    pub fn draw_text(&mut self, text: &str, color: &Color, x: i32, y: i32) {}

    fn to_bytes(&self) -> [u8; 3 * ROWS * COLS] {
        let mut result = [0u8; 3 * ROWS * COLS];
        let mut idx = 0;
        for (col_idx, col) in self.pixels.iter().enumerate() {
            let iter = if col_idx % 2 == 1 {
                col.iter()
            } else {
                col.iter()
            };
            for pixel in iter {
                for channel in pixel.to_bytes().iter() {
                    result[idx] = *channel;
                    idx += 1;
                }
            }
        }
        result
    }

    pub fn get_spi_data(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        let bytes = self.to_bytes();
        let bits = bytes.view_bits::<Msb0>();

        let mut idx = 0;
        for bit in bits.iter() {
            if *bit {
                result.push(0b1111_1111);
                result.push(0b1000_0000);
            } else {
                result.push(0b1111_1000);
                result.push(0b0000_0000);
            }
            idx += 8;
        };

        for _ in 1..1000 {
            result.push(0)
        }
        result
    }
}