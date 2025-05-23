#![no_std]

pub mod pins;
use core::convert::Infallible;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_hal::{delay::DelayNs};
use pins::{DualPixel, Hub75Pins};

/// Constants for the display dimensions
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 64;
const ACTIVE_ROWS: usize = DISPLAY_HEIGHT / 2; // Number of rows to address

/// Complete framebuffer for a 64x64 display
pub struct FrameBuffer {
    buffer: [[DualPixel; DISPLAY_WIDTH]; ACTIVE_ROWS],
    modified: bool,
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameBuffer {
    /// Create a new, empty framebuffer
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: [[DualPixel::default(); DISPLAY_WIDTH]; ACTIVE_ROWS],
            modified: true,
        }
    }

    /// Set a single pixel's color
    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return;
        }

        // Determine if this is in the top or bottom half
        let row_address = y % ACTIVE_ROWS;

        // Update the appropriate pixel
        if y < ACTIVE_ROWS {
            // Top half
            self.buffer[row_address][x].r1 = r;
            self.buffer[row_address][x].g1 = g;
            self.buffer[row_address][x].b1 = b;
        } else {
            // Bottom half
            self.buffer[row_address][x].r2 = r;
            self.buffer[row_address][x].g2 = g;
            self.buffer[row_address][x].b2 = b;
        }

        self.modified = true;
    }

    /// Clear the framebuffer
    pub fn clear(&mut self) {
        for row in &mut self.buffer {
            for pixel in row.iter_mut() {
                *pixel = DualPixel::default();
            }
        }
        self.modified = true;
    }

    /// Check if the framebuffer has been modified
    #[must_use]
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Reset the modified flag
    pub fn reset_modified(&mut self) {
        self.modified = false;
    }
}

/// Configuration options for the Hub75 driver
#[derive(Clone, Copy)]
pub struct Hub75Config {
    pub pwm_bits: u8,               // Number of bits for PWM (1-8)
    pub brightness: u8,             // Overall brightness (0-255)
    pub use_gamma_correction: bool, // Apply gamma correction to colors
    pub row_step_time_us: u32,      // Delay between row updates
}

impl Default for Hub75Config {
    fn default() -> Self {
        Self {
            pwm_bits: 6,                // 6-bit PWM
            brightness: 255,            // High brightness
            use_gamma_correction: true, // Enable gamma correction for better visuals
            row_step_time_us: 1,        // 1µs delay between row transitions
        }
    }
}

/// Gamma correction lookup table for better color representation
static GAMMA8: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5,
    5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14,
    14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25, 25, 26, 27,
    27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72,
    73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104,
    105, 107, 109, 110, 112, 114, 115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137,
    138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175,
    177, 180, 182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
    223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
];

/// Main Hub75 driver structure with static dispatch
pub struct Hub75 {
    pins: Hub75Pins,
    pub config: Hub75Config,
    framebuffer: FrameBuffer,
}

impl Hub75 {
    /// Create a new Hub75 driver with default configuration
    pub fn new(pins: Hub75Pins) -> Self {
        Self::new_with_config(pins, Hub75Config::default())
    }

    /// Create a new Hub75 driver with custom configuration
    pub fn new_with_config(pins: Hub75Pins, config: Hub75Config) -> Self {
        let framebuffer = FrameBuffer::new();

        Self {
            pins,
            config,
            framebuffer,
        }
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: Hub75Config) {
        self.config = config;
    }

    /// Update the display with the current framebuffer contents
    pub fn update(&mut self, delay: &mut impl DelayNs) {
        // Only update if the framebuffer has changed
        if !self.framebuffer.is_modified() {
            return;
        }

        // Start with output disabled
        self.pins.set_output_enabled(false);

        // Correct PWM bit plane implementation - directly use the bit count
        let num_bit_planes = self.config.pwm_bits as usize;

        // Process each row
        for row in 0..ACTIVE_ROWS {
            // For each bit position in PWM sequence (binary-coded modulation)
            for bit_plane in 0..num_bit_planes {
                // Calculate the bit mask for this bit position
                // MSB (highest bit_plane) has the largest weight and should be displayed longest
                let bit_position = num_bit_planes - 1 - bit_plane;

                // Shift in the data for this row
                for col in 0..DISPLAY_WIDTH {
                    let pixel = self.framebuffer.buffer[row][col];

                    // Apply gamma and brightness in-place
                    let (r1, g1, b1, r2, g2, b2) =
                        (pixel.r1, pixel.g1, pixel.b1, pixel.r2, pixel.g2, pixel.b2);
                    // Apply brightness


                    // Bit plane comparison
                    let mask = 1 << (7 - bit_plane); // MSB first
                    let r1_active = (r1 & mask) != 0;
                    let g1_active = (g1 & mask) != 0;
                    let b1_active = (b1 & mask) != 0;

                    let r2_active = (r2 & mask) != 0;
                    let g2_active = (g2 & mask) != 0;
                    let b2_active = (b2 & mask) != 0;

                    // Set the color pins
                    let dual_pixel = DualPixel {
                        r1: u8::from(r1_active),
                        g1: u8::from(g1_active),
                        b1: u8::from(b1_active),
                        r2: u8::from(r2_active),
                        g2: u8::from(g2_active),
                        b2: u8::from(b2_active),
                    };
                    self.pins.set_color_pins(&dual_pixel, 0);
                    self.pins.clock_pulse();
                }

                // Latch the data
                self.pins.latch();

                // Set row address
                self.pins.set_row(row);

                // Enable output
                self.pins.set_output_enabled(true);

                // Hold proportionally to the bit weight (binary coded modulation)
                // MSB (bit_position = pwm_bits-1) should be displayed longest
                let hold_time = (1 << bit_position) * self.config.row_step_time_us;
                delay.delay_us(hold_time);

                // Disable output before next bit plane
                self.pins.set_output_enabled(false);

                // Small delay to prevent ghosting
                delay.delay_ns(1);
            }
        }

        // Mark framebuffer as updated
        self.framebuffer.reset_modified();
    }

    /// Set a pixel in the framebuffer
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Rgb565) {
        // Convert Rgb565 to 8-bit linear scale
        let mut r = color.r() << 3; // 5-bit -> 8-bit
        let mut g = color.g() << 2; // 6-bit -> 8-bit
        let mut b = color.b() << 3;

        let brightness = u16::from(self.config.brightness);
        r = ((u16::from(r) * brightness) >> 8) as u8;
        g = ((u16::from(g) * brightness) >> 8) as u8;
        b = ((u16::from(b) * brightness) >> 8) as u8;

        if self.config.use_gamma_correction {
            r = GAMMA8[r as usize];
            g = GAMMA8[g as usize];
            b = GAMMA8[b as usize];
        }

        // Swap the colors to match the hardware configuration
        // Based on your description: blue→green, green→red, red→blue
        let r_final = b; // Red pin receives what should be blue
        let g_final = r; // Green pin receives what should be red
        let b_final = g; // Blue pin receives what should be green

        self.framebuffer.set_pixel(x as usize, y as usize, r_final, g_final, b_final);
    }

    /// Clear the framebuffer
    pub fn clear(&mut self) {
        self.framebuffer.clear();
    }

    /// Draw a test pattern to verify correct row mapping and scanning
    pub fn draw_test_pattern(&mut self) {
        // Clear the framebuffer first
        self.clear();

        // Draw horizontal color bands
        for y in 0..DISPLAY_HEIGHT {
            let color = match (y / 8) % 8 {
                0 => Rgb565::RED,
                1 => Rgb565::GREEN,
                2 => Rgb565::BLUE,
                3 => Rgb565::CYAN,
                4 => Rgb565::MAGENTA,
                5 => Rgb565::YELLOW,
                6 => Rgb565::WHITE,
                _ => Rgb565::new(255 >> 3, 128 >> 2, 0), // Orange
            };

            for x in 0..DISPLAY_WIDTH {
                self.set_pixel(x as i32, y as i32, color);
            }
        }

        // Add a diagonal line for visual confirmation
        for i in 0..DISPLAY_HEIGHT {
            self.set_pixel(i as i32, i as i32, Rgb565::WHITE);
            // Draw a thicker line for better visibility
            if i > 0 {
                self.set_pixel(i as i32 - 1, i as i32, Rgb565::WHITE);
            }
            if i < DISPLAY_WIDTH - 1 {
                self.set_pixel(i as i32 + 1, i as i32, Rgb565::WHITE);
            }
        }

        // Draw a grid pattern
        for i in 0..DISPLAY_HEIGHT {
            if i % 8 == 0 {
                for x in 0..DISPLAY_WIDTH {
                    self.set_pixel(x as i32, i as i32, Rgb565::BLACK);
                }
            }
        }

        for i in 0..DISPLAY_WIDTH {
            if i % 8 == 0 {
                for y in 0..DISPLAY_HEIGHT {
                    self.set_pixel(i as i32, y as i32, Rgb565::BLACK);
                }
            }
        }
    }

    // Draw a test gradient
    pub fn draw_test_gradient(&mut self) {
        self.clear();

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                self.set_pixel(
                    x as i32,
                    y as i32,
                    Rgb565::new(
                        (x * 32 / DISPLAY_WIDTH) as u8,
                        32,
                        (y * 32 / DISPLAY_HEIGHT) as u8,
                    ),
                );
            }
        }
    }
}

// Implement embedded-graphics interfaces
impl OriginDimensions for Hub75  {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}

impl DrawTarget for Hub75 {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            self.set_pixel(point.x, point.y, color);
        }

        Ok(())
    }
}
