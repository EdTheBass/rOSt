use alloc::{boxed::Box, slice};
use kernel::structures::kernel_information::{KernelInformation, PixelFormat};
use noto_sans_mono_bitmap::{get_bitmap, BitmapChar};
use tinytga::RawTga;
use utils::static_stack::StaticStack;

use crate::{
    pixel_buffer::{BasePixelBuffer, PixelBuffer},
    vga_core::{ImageDrawable, CHAR_HEIGHT, INVALID_CHAR},
};

use super::{
    point_2d::Point2D,
    vga_color::VGAColor,
    vga_core::{Clearable, PlainDrawable, ShapeDrawable, TextDrawable, CHAR_WEIGHT, CHAR_WIDTH},
};

pub struct VGADevice {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pixel_buffer: Box<dyn PixelBuffer>,
}

impl Clearable for VGADevice {
    fn clear(&mut self, color: VGAColor<u8>) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.draw_point(x as u16, y as u16, color);
            }
        }
    }
}

impl PlainDrawable for VGADevice {
    #[inline(always)]
    fn draw_point(&mut self, x: u16, y: u16, color: VGAColor<u8>) {
        let x = x as usize;
        let y = y as usize;
        let index = (y * self.stride) + x;
        self.pixel_buffer.put_pixel(index, color);
    }
    fn draw_point_p(&mut self, p: Point2D<u16>, color: VGAColor<u8>) {
        self.draw_point(p.x, p.y, color);
    }

    fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: VGAColor<u8>) {
        let x2 = x2 as i16;
        let y2 = y2 as i16;
        // Bresenham's algorithm

        let mut x1 = x1 as i16;
        let mut y1 = y1 as i16;

        let xi: i16;
        let dx: i16;
        if x1 < x2 {
            xi = 1;
            dx = x2 - x1;
        } else {
            xi = -1;
            dx = x1 - x2;
        }

        let yi: i16;
        let dy: i16;
        if y1 < y2 {
            yi = 1;
            dy = y2 - y1;
        } else {
            yi = -1;
            dy = y1 - y2;
        }
        self.draw_point(x1 as u16, y1 as u16, color);

        let ai;
        let bi;
        let mut d: i16;
        // OX axis
        if dx > dy {
            ai = (dy - dx) * 2;
            bi = dy * 2;
            d = bi - dx;
            // Loop over next Xs
            while x1 != x2 {
                if d >= 0 {
                    x1 += xi;
                    y1 += yi;
                    d += ai;
                } else {
                    d += bi;
                    x1 += xi;
                }
                self.draw_point(x1 as u16, y1 as u16, color);
            }
        }
        // OY axis
        else {
            ai = (dx - dy) * 2;
            bi = dx * 2;
            d = bi - dy;
            // Loop over next Ys
            while y1 != y2 {
                if d >= 0 {
                    x1 += xi;
                    y1 += yi;
                    d += ai;
                } else {
                    d += bi;
                    y1 += yi;
                }
                self.draw_point(x1 as u16, y1 as u16, color);
            }
        }
    }
    fn draw_line_p(&mut self, a: Point2D<u16>, b: Point2D<u16>, color: VGAColor<u8>) {
        self.draw_line(a.x, a.y, b.x, b.y, color);
    }

    fn draw_bezier(
        &mut self,
        p1: Point2D<u16>,
        p2: Point2D<u16>,
        p3: Point2D<u16>,
        p4: Point2D<u16>,
        color: VGAColor<u8>,
    ) {
        let mut t_stack: StaticStack<(f32, f32), 32> = StaticStack::new();
        t_stack.push(&(0f32, 1f32)).unwrap();
        while t_stack.length() > 0 {
            let frame = t_stack.pop().unwrap();
            let a = bezier_point(p1, p2, p3, p4, frame.0);
            let b = bezier_point(p1, p2, p3, p4, frame.1);
            if a.sqr_distance::<i32>(b) > 16 {
                let mid = (frame.1 + frame.0) * 0.5;
                t_stack.push(&(frame.0, mid)).unwrap();
                t_stack.push(&(mid, frame.1)).unwrap();
            } else {
                self.draw_line_p(a, b, color);
            }
        }
    }
}

fn bezier_point(
    p1: Point2D<u16>,
    p2: Point2D<u16>,
    p3: Point2D<u16>,
    p4: Point2D<u16>,
    t: f32,
) -> Point2D<u16> {
    let t_1 = 1f32 - t;
    let t2 = t * t;
    let t3 = t2 * t;
    let _p1: Point2D<f32> = p1.into();
    let _p2: Point2D<f32> = (p2 * 3).into();
    let _p3: Point2D<f32> = (p3 * 3).into();
    let _p4: Point2D<f32> = p4.into();
    (((_p1 * t_1 + _p2 * t) * t_1 + _p3 * t2) * t_1 + _p4 * t3).into()
}

impl ShapeDrawable for VGADevice {
    fn draw_rectangle(&mut self, x: u16, y: u16, width: u16, height: u16, color: VGAColor<u8>) {
        self.draw_line(x, y, x + width, y, color);
        self.draw_line(x, y + height, x + width, y + height, color);
        self.draw_line(x, y, x, y + height, color);
        self.draw_line(x + width, y, x + width, y + height, color);
    }
    fn draw_rectangle_p(&mut self, min: Point2D<u16>, max: Point2D<u16>, color: VGAColor<u8>) {
        self.draw_rectangle(min.x, min.y, max.x - min.x, max.y - min.y, color);
    }

    fn fill_rectangle(&mut self, x: u16, y: u16, width: u16, height: u16, color: VGAColor<u8>) {
        for i in x..x + width {
            for j in y..y + height {
                self.draw_point(i, j, color);
            }
        }
    }
    fn fill_rectangle_p(&mut self, min: Point2D<u16>, max: Point2D<u16>, color: VGAColor<u8>) {
        self.fill_rectangle(min.x, min.y, max.x - min.x, max.y - min.y, color);
    }
}

impl TextDrawable for VGADevice {
    fn draw_string(
        &mut self,
        x: u16,
        y: u16,
        color: VGAColor<u8>,
        text: &str,
        reset_x: u16,
    ) -> (u16, u16) {
        let mut pos_x = x;
        let mut pos_y = y;
        for (_i, c) in text.chars().enumerate() {
            match c {
                '\n' => {
                    pos_x = reset_x;
                    pos_y += CHAR_HEIGHT as u16;
                }
                _ => {
                    if pos_x + CHAR_WIDTH > self.width as u16 {
                        pos_x = reset_x;
                        pos_y += CHAR_HEIGHT as u16;
                    }
                    let invalid_char = &*INVALID_CHAR;
                    let bitmap_char = get_bitmap(c, CHAR_WEIGHT, CHAR_HEIGHT);
                    self.draw_char(
                        pos_x,
                        pos_y,
                        &bitmap_char.as_ref().unwrap_or(invalid_char),
                        color,
                    );
                    pos_x += CHAR_WIDTH;
                }
            }
        }
        (pos_x, pos_y)
    }

    fn measure_string(&self, x: u16, y: u16, text: &str, reset_x: u16) -> (u16, u16) {
        let mut pos_x = x;
        let mut pos_y = y;
        for (_i, c) in text.chars().enumerate() {
            match c {
                '\n' => {
                    pos_x = reset_x;
                    pos_y += CHAR_HEIGHT as u16;
                }
                _ => {
                    pos_x += CHAR_WIDTH;
                    if pos_x > self.width as u16 {
                        pos_x = reset_x;
                        pos_y += CHAR_HEIGHT as u16;
                    }
                }
            }
        }
        (pos_x, pos_y)
    }
}

impl ImageDrawable for VGADevice {
    fn draw_image(&mut self, x0: u16, y0: u16, image: &RawTga) {
        for pixel in image.pixels() {
            let pos = pixel.position;
            let color = VGAColor::<u8> {
                red: (pixel.color >> 16) as u8,
                green: (pixel.color >> 8) as u8,
                blue: pixel.color as u8,
                alpha: 255,
            };
            self.draw_point(pos.x as u16 + x0, pos.y as u16 + y0, color);
        }
    }
    fn draw_image_p(&mut self, pos: Point2D<u16>, image: &RawTga) {
        self.draw_image(pos.x, pos.y, image);
    }
}

impl VGADevice {
    fn draw_char(&mut self, x: u16, y: u16, char: &BitmapChar, color: VGAColor<u8>) {
        for (iy, row) in char.bitmap().iter().enumerate() {
            for (ix, byte) in row.iter().enumerate() {
                self.draw_point(ix as u16 + x, iy as u16 + y, color.mul_alpha(*byte));
            }
        }
    }
}

pub struct VGADeviceFactory;

impl VGADeviceFactory {
    pub fn from_kernel_info(kernel_info: KernelInformation) -> VGADevice {
        let buffer = kernel_info.framebuffer.as_ref().unwrap();
        VGADevice {
            width: buffer.width,
            height: buffer.height,
            stride: buffer.stride,
            pixel_buffer: match buffer.format {
                PixelFormat::RGB => Box::new(BasePixelBuffer::<{ PixelFormat::RGB }> {
                    bytes_per_pixel_shift: log2(buffer.bytes_per_pixel),
                    frame_pointer: unsafe {
                        slice::from_raw_parts_mut::<u8>(
                            buffer.buffer,
                            buffer.bytes_per_pixel * buffer.stride * buffer.height,
                        )
                    },
                }),
                PixelFormat::BGR => Box::new(BasePixelBuffer::<{ PixelFormat::BGR }> {
                    bytes_per_pixel_shift: log2(buffer.bytes_per_pixel),
                    frame_pointer: unsafe {
                        slice::from_raw_parts_mut::<u8>(
                            buffer.buffer,
                            buffer.bytes_per_pixel * buffer.stride * buffer.height,
                        )
                    },
                }),
                PixelFormat::U8 => Box::new(BasePixelBuffer::<{ PixelFormat::U8 }> {
                    bytes_per_pixel_shift: log2(buffer.bytes_per_pixel),
                    frame_pointer: unsafe {
                        slice::from_raw_parts_mut::<u8>(
                            buffer.buffer,
                            buffer.bytes_per_pixel * buffer.stride * buffer.height,
                        )
                    },
                }),
            },
        }
    }
}

fn log2(value: usize) -> u8 {
    match value {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 2,
        _ => todo!("log2 needs more values"),
    }
}
