//! Hub75 LED Matrix Driver for RP2350
//!
//! This driver supports 64x64 LED matrices using the HUB75 protocol.
#![no_std]

// Compile-time check to ensure only one color mapping is selected
#[cfg(all(
    feature = "mapping-brg",
    any(
        feature = "mapping-gbr",
        feature = "mapping-bgr",
        feature = "mapping-rbg",
        feature = "mapping-grb"
    )
))]
compile_error!("Only one color mapping feature can be enabled at a time");

#[cfg(all(
    feature = "mapping-gbr",
    any(
        feature = "mapping-bgr",
        feature = "mapping-rbg",
        feature = "mapping-grb"
    )
))]
compile_error!("Only one color mapping feature can be enabled at a time");

#[cfg(all(
    feature = "mapping-bgr",
    any(feature = "mapping-rbg", feature = "mapping-grb")
))]
compile_error!("Only one color mapping feature can be enabled at a time");

#[cfg(all(feature = "mapping-rbg", feature = "mapping-grb"))]
compile_error!("Only one color mapping feature can be enabled at a time");

pub mod pins;
use core::convert::Infallible;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_hal::delay::DelayNs;
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
    pub clock_delay_ns: u32,        // Delay for clock pulses in nanoseconds
}

impl Default for Hub75Config {
    fn default() -> Self {
        Self {
            pwm_bits: 6,                // 6-bit PWM
            brightness: 255,            // Full brightness
            use_gamma_correction: true, // Enable gamma correction for better visuals
            clock_delay_ns: 100,        // 100ns clock delay
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
    r_lut: [u8; 32], // 5-bit input (Rgb565 red channel)
    g_lut: [u8; 64], // 6-bit input (Rgb565 green channel)
    b_lut: [u8; 32], // 5-bit input (Rgb565 blue channel)
}

impl Hub75 {
    /// Create a new Hub75 driver with default configuration
    pub fn new(pins: Hub75Pins) -> Self {
        Self::new_with_config(pins, Hub75Config::default())
    }

    /// Create a new Hub75 driver with custom configuration
    pub fn new_with_config(pins: Hub75Pins, config: Hub75Config) -> Self {
        let framebuffer = FrameBuffer::new();
        let mut driver = Self {
            pins,
            config,
            framebuffer,
            r_lut: [0; 32],
            g_lut: [0; 64],
            b_lut: [0; 32],
        };
        driver.update_luts(); // Initialize LUTs
        driver
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: Hub75Config) {
        self.config = config;
        self.update_luts(); // Rebuild LUTs on config change
    }

    fn update_luts(&mut self) {
        let brightness = self.config.brightness as u16;
        let shift = 8 - self.config.pwm_bits;
        let use_gamma = self.config.use_gamma_correction;

        // Precompute red LUT (5-bit input)
        for i in 0..32 {
            // Original conversion from Rgb565 to 8-bit
            let mut val = (i as u16 * 255) / 31;

            // Apply brightness
            if brightness < 255 {
                val = (val * brightness) / 255;
            }

            // Apply gamma correction
            let val_8bit = if use_gamma {
                GAMMA8[val as usize]
            } else {
                val as u8
            };

            // Scale to PWM bit depth
            self.r_lut[i as usize] = val_8bit >> shift;
        }

        // Precompute green LUT (6-bit input)
        for i in 0..64 {
            // Original conversion from Rgb565 to 8-bit
            let mut val = (i as u16 * 255) / 63;

            // Apply brightness
            if brightness < 255 {
                val = (val * brightness) / 255;
            }

            // Apply gamma correction
            let val_8bit = if use_gamma {
                GAMMA8[val as usize]
            } else {
                val as u8
            };

            // Scale to PWM bit depth
            self.g_lut[i as usize] = val_8bit >> shift;
        }

        // Precompute blue LUT (5-bit input)
        for i in 0..32 {
            // Original conversion from Rgb565 to 8-bit
            let mut val = (i as u16 * 255) / 31;

            // Apply brightness
            if brightness < 255 {
                val = (val * brightness) / 255;
            }

            // Apply gamma correction
            let val_8bit = if use_gamma {
                GAMMA8[val as usize]
            } else {
                val as u8
            };

            // Scale to PWM bit depth
            self.b_lut[i as usize] = val_8bit >> shift;
        }
    }

    /// Update the display with the current framebuffer contents
    pub fn update(&mut self, delay: &mut impl DelayNs) -> Result<(), Infallible> {
        // Only update if the framebuffer has changed
        if !self.framebuffer.is_modified() {
            return Ok(());
        }

        // Process each bit plane using Binary Code Modulation (BCM)
        for bit_plane in 0..self.config.pwm_bits {
            // Process each row
            for row in 0..ACTIVE_ROWS {
                // Disable output while shifting in data
                self.pins.set_output_enabled(false);

                // Shift in all the pixels for this row
                for col in 0..DISPLAY_WIDTH {
                    let pixel = &self.framebuffer.buffer[row][col];

                    // Extract the bit for this bit plane
                    // Use bit_plane directly as the shift amount (LSB to MSB)
                    let shift = bit_plane;

                    let r1_bit = (pixel.r1 >> shift) & 1;
                    let g1_bit = (pixel.g1 >> shift) & 1;
                    let b1_bit = (pixel.b1 >> shift) & 1;

                    let r2_bit = (pixel.r2 >> shift) & 1;
                    let g2_bit = (pixel.g2 >> shift) & 1;
                    let b2_bit = (pixel.b2 >> shift) & 1;

                    // Set the color pins based on the extracted bits
                    self.pins
                        .set_color_bits(r1_bit, g1_bit, b1_bit, r2_bit, g2_bit, b2_bit);

                    // Clock the data
                    self.pins
                        .clock_pulse_with_delay(delay, self.config.clock_delay_ns);
                }

                // Latch the row data
                self.pins.latch_with_delay(delay);

                // Set the row address
                self.pins.set_row(row);

                // Enable output
                self.pins.set_output_enabled(true);

                // Hold for a duration proportional to the bit weight
                // For BCM, each bit plane is displayed for 2^bit_plane time units
                let hold_time_us = 1u32 << bit_plane;
                delay.delay_us(hold_time_us);

                // Disable output before moving to next row
                self.pins.set_output_enabled(false);

                // Small delay to prevent ghosting
                delay.delay_ns(100);
            }
        }

        // Mark framebuffer as updated
        self.framebuffer.reset_modified();

        Ok(())
    }

    /// Set a pixel in the framebuffer
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Rgb565) {
        if x < 0 || y < 0 || x >= DISPLAY_WIDTH as i32 || y >= DISPLAY_HEIGHT as i32 {
            return;
        }

        let r = self.r_lut[color.r() as usize]; // 5-bit → 8-bit index
        let g = self.g_lut[color.g() as usize]; // 6-bit → 8-bit index
        let b = self.b_lut[color.b() as usize]; // 5-bit → 8-bit index

        // Apply hardware color mapping based on feature flags
        #[cfg(feature = "mapping-brg")]
        let (r_final, g_final, b_final) = (b, r, g); // Blue→Red, Red→Green, Green→Blue

        #[cfg(feature = "mapping-gbr")]
        let (r_final, g_final, b_final) = (g, b, r); // Green→Red, Blue→Green, Red→Blue

        #[cfg(feature = "mapping-bgr")]
        let (r_final, g_final, b_final) = (b, g, r); // Blue→Red, Green→Green, Red→Blue

        #[cfg(feature = "mapping-rbg")]
        let (r_final, g_final, b_final) = (r, b, g); // Red→Red, Blue→Green, Green→Blue

        #[cfg(feature = "mapping-grb")]
        let (r_final, g_final, b_final) = (g, r, b); // Green→Red, Red→Green, Blue→Blue

        // Default to standard RGB mapping if no feature is selected
        #[cfg(not(any(
            feature = "mapping-brg",
            feature = "mapping-gbr",
            feature = "mapping-bgr",
            feature = "mapping-rbg",
            feature = "mapping-grb"
        )))]
        let (r_final, g_final, b_final) = (r, g, b);

        self.framebuffer
            .set_pixel(x as usize, y as usize, r_final, g_final, b_final);
    }

    /// Clear the framebuffer
    pub fn clear(&mut self) {
        self.framebuffer.clear();
    }

    /// Draw a test pattern to verify correct row mapping and scanning
    pub fn draw_test_pattern(&mut self) {
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
                _ => Rgb565::new(31, 32, 0), // Orange
            };

            for x in 0..DISPLAY_WIDTH {
                self.set_pixel(x as i32, y as i32, color);
            }
        }

        // Add diagonal lines
        for i in 0..DISPLAY_HEIGHT.min(DISPLAY_WIDTH) {
            self.set_pixel(i as i32, i as i32, Rgb565::WHITE);
            self.set_pixel((DISPLAY_WIDTH - 1 - i) as i32, i as i32, Rgb565::WHITE);
        }
    }

    /// Draw a test gradient
    ///
    /// Creates an RGB gradient that will appear differently based on
    /// the color mapping feature selected at compile time:
    ///
    /// - Standard RGB: Red increases left→right, Green top→bottom
    /// - BRG mapping: Actual display shows different colors due to hardware remapping
    pub fn draw_test_gradient(&mut self) {
        self.clear();

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let r = (x * 31 / (DISPLAY_WIDTH - 1)) as u8;
                let g = (y * 63 / (DISPLAY_HEIGHT - 1)) as u8;
                let b = ((x + y) * 31 / (DISPLAY_WIDTH + DISPLAY_HEIGHT - 2)) as u8;

                self.set_pixel(x as i32, y as i32, Rgb565::new(r, g, b));
            }
        }
    }

    /// Draw a color channel test to verify hardware mapping
    ///
    /// This displays three vertical bands showing pure color channels:
    /// - Left third: Should appear RED (if mapping is correct)
    /// - Middle third: Should appear GREEN (if mapping is correct)
    /// - Right third: Should appear BLUE (if mapping is correct)
    ///
    /// If the colors don't match expectations, try a different mapping feature:
    /// - If you see BGR instead of RGB, use `mapping-bgr`
    /// - If you see BRG instead of RGB, use `mapping-brg`
    /// - If you see GBR instead of RGB, use `mapping-gbr`
    /// - If you see GRB instead of RGB, use `mapping-grb`
    /// - If you see RBG instead of RGB, use `mapping-rbg`
    pub fn draw_channel_test(&mut self) {
        self.clear();

        // Divide screen into 3 vertical sections
        let section_width = DISPLAY_WIDTH / 3;

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let intensity = (y * 63 / (DISPLAY_HEIGHT - 1)) as u8;

                let color = if x < section_width {
                    // Left third: Pure red gradient
                    Rgb565::new(intensity >> 1, 0, 0)
                } else if x < section_width * 2 {
                    // Middle third: Pure green gradient
                    Rgb565::new(0, intensity, 0)
                } else {
                    // Right third: Pure blue gradient
                    Rgb565::new(0, 0, intensity >> 1)
                };

                self.set_pixel(x as i32, y as i32, color);
            }
        }
    }
}

// Implement embedded-graphics interfaces
impl OriginDimensions for Hub75 {
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
