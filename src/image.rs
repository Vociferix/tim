use std::io::{BufRead, Read, Seek, Write};

use crossterm::{
    terminal,
    cursor,
    style::{self, Stylize, Color},
    queue
};

use anyhow::Result;

const PIXEL_CHAR: char = '▀';

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Image {
    pixels: Vec<Pixel>,
    width: usize,
    height: usize,
}

fn apply_alpha16(value: u16, alpha: u16) -> u16 {
    ((value as u32) * (alpha as u32) / 65535) as u16
}

fn apply_alpha(value: u8, alpha: u8) -> u8 {
    ((value as u32) * (alpha as u32) / 255) as u8
}

fn u16_to_u8(value: u16) -> u8 {
    (value >> 8) as u8
}

fn f32_to_u8(value: f32) -> u8 {
    let value = value * 255.0;
    if value < 0.0 {
        0
    } else if value > 255.0 {
        255
    } else {
        value as u8
    }
}

impl Image {
    fn new_gray8(im: image::GrayImage) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            pixels.push(Pixel {
                r: pix.0[0],
                g: pix.0[0],
                b: pix.0[0],
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new_grayalpha8(im: image::GrayAlphaImage) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            let val = apply_alpha(pix.0[0], pix.0[1]);
            pixels.push(Pixel {
                r: val,
                g: val,
                b: val,
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new_rgb8(im: image::RgbImage) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            pixels.push(Pixel {
                r: pix.0[0],
                g: pix.0[1],
                b: pix.0[2],
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new_rgba8(im: image::RgbaImage) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            pixels.push(Pixel {
                r: apply_alpha(pix.0[0], pix.0[3]),
                g: apply_alpha(pix.0[1], pix.0[3]),
                b: apply_alpha(pix.0[2], pix.0[3]),
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new_gray16(im: image::ImageBuffer<image::Luma<u16>, Vec<u16>>) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            let val = u16_to_u8(pix.0[0]);
            pixels.push(Pixel {
                r: val,
                g: val,
                b: val,
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new_grayalpha16(im: image::ImageBuffer<image::LumaA<u16>, Vec<u16>>) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            let val = u16_to_u8(apply_alpha16(pix.0[0], pix.0[1]));
            pixels.push(Pixel {
                r: val,
                g: val,
                b: val,
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new_rgb16(im: image::ImageBuffer<image::Rgb<u16>, Vec<u16>>) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            pixels.push(Pixel {
                r: u16_to_u8(pix.0[0]),
                g: u16_to_u8(pix.0[1]),
                b: u16_to_u8(pix.0[2]),
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new_rgba16(im: image::ImageBuffer<image::Rgba<u16>, Vec<u16>>) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            pixels.push(Pixel {
                r: u16_to_u8(apply_alpha16(pix.0[0], pix.0[3])),
                g: u16_to_u8(apply_alpha16(pix.0[1], pix.0[3])),
                b: u16_to_u8(apply_alpha16(pix.0[2], pix.0[3])),
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new_rgb32f(im: image::Rgb32FImage) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            pixels.push(Pixel {
                r: f32_to_u8(pix.0[0]),
                g: f32_to_u8(pix.0[1]),
                b: f32_to_u8(pix.0[2]),
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new_rgba32f(im: image::Rgba32FImage) -> Result<Self> {
        let (width, height) = im.dimensions();
        let width = width as usize;
        let height = height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for pix in im.pixels() {
            pixels.push(Pixel {
                r: f32_to_u8(pix.0[0] * pix.0[3]),
                g: f32_to_u8(pix.0[1] * pix.0[3]),
                b: f32_to_u8(pix.0[2] * pix.0[3]),
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    fn new(im: image::DynamicImage) -> Result<Self> {
        match im {
            image::DynamicImage::ImageLuma8(im) => {
                Self::new_gray8(im)
            },
            image::DynamicImage::ImageLumaA8(im) => {
                Self::new_grayalpha8(im)
            },
            image::DynamicImage::ImageRgb8(im) => {
                Self::new_rgb8(im)
            },
            image::DynamicImage::ImageRgba8(im) => {
                Self::new_rgba8(im)
            },
            image::DynamicImage::ImageLuma16(im) => {
                Self::new_gray16(im)
            },
            image::DynamicImage::ImageLumaA16(im) => {
                Self::new_grayalpha16(im)
            },
            image::DynamicImage::ImageRgb16(im) => {
                Self::new_rgb16(im)
            },
            image::DynamicImage::ImageRgba16(im) => {
                Self::new_rgba16(im)
            },
            image::DynamicImage::ImageRgb32F(im) => {
                Self::new_rgb32f(im)
            },
            image::DynamicImage::ImageRgba32F(im) => {
                Self::new_rgba32f(im)
            },
            _ => {
                todo!()
            },
        }
    }

    pub fn load<R: BufRead + Seek>(im: R) -> Result<Self> {
        Self::new(image::io::Reader::new(im).with_guessed_format()?.decode()?)
    }

    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Self::new(image::io::Reader::open(path)?.decode()?)
    }

    pub fn draw<W: Write>(&self, term: &mut W, pos: (usize, usize), offset: (usize, usize), zoom: f32) -> Result<()> {
        let ws = crossterm::terminal::window_size()?;
        let twidth = ws.columns as usize;
        let theight = ws.rows as usize;

        for x in 0..twidth {
            for y in 0..theight {
                if x < offset.0 || y < offset.1 {
                    queue!(term, cursor::MoveTo(x as u16, y as u16), style::PrintStyledContent(' '.on_black()))?;
                } else {
                    let pix1 = self.pixel(((x - offset.0) + pos.0, ((y - offset.1) * 2) + pos.1), zoom);
                    let pix2 = self.pixel(((x - offset.0) + pos.0, ((y - offset.1) * 2) + pos.1 + 1), zoom);
                    queue!(term, cursor::MoveTo(x as u16, y as u16), style::PrintStyledContent(PIXEL_CHAR.with(Color::Rgb { r: pix1.r, g: pix1.g, b: pix1.b }).on(Color::Rgb { r: pix2.r, g: pix2.g, b: pix2.b })))?;
                }
            }
        }

        Ok(())
    }

    pub fn size(&self, zoom: f32) -> (usize, usize) {
        ((self.width as f32 * zoom) as usize, (self.height as f32 * zoom) as usize)
    }

    pub fn pixel(&self, pos: (usize, usize), zoom: f32) -> Pixel {
        let x = (pos.0 as f32 / zoom) as usize;
        let y = (pos.1 as f32 / zoom) as usize;

        if x >= self.width || y >= self.height {
            Pixel::default()
        } else {
            let pos = (y * self.width) + x;
            self.pixels[pos].clone()
        }
    }
}
