#![no_std]

use core::convert::Infallible;
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{Rgb565, RgbColor},
    Pixel,
};
use embedded_hal::{
    delay::DelayNs,
};
use core::marker::PhantomData;
use embedded_hal::digital::OutputPin;

/// Constants for the display dimensions
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 64;
// const ROWS_PER_PANEL: usize = 32; // Physical rows per panel (64x64 is dual 32-row scanning)
const ACTIVE_ROWS: usize = DISPLAY_HEIGHT / 2; // Number of rows to address

/// Buffer format for dual scanning 64x64 matrix
/// Each entry represents the color values for both top and bottom pixels
#[derive(Clone, Copy, Default)]
pub struct DualPixel {
    pub r1: u8,  // Red for top half
    pub g1: u8,  // Green for top half
    pub b1: u8,  // Blue for top half
    pub r2: u8,  // Red for bottom half
    pub g2: u8,  // Green for bottom half
    pub b2: u8,  // Blue for bottom half
}

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
        for row in self.buffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = DualPixel::default();
            }
        }
        self.modified = true;
    }

    /// Check if the framebuffer has been modified
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
    pub chain_length: usize,        // Number of panels in series (default 1)
    pub row_step_time_us: u32,      // Delay between row updates
}

impl Default for Hub75Config {
    fn default() -> Self {
        Self {
            pwm_bits: 4,               // 4-bit PWM (16 brightness levels)
            brightness: 255,           // Full brightness
            use_gamma_correction: true, // Enable gamma correction for better visuals
            chain_length: 1,           // Single 64x64 panel
            row_step_time_us: 1,       // 1µs delay between row transitions
        }
    }
}

/// Gamma correction lookup table for better color representation
static GAMMA8: [u8; 256] = [
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  1,  1,  1,  1,
    1,  1,  1,  1,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  2,  2,
    2,  3,  3,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,  5,  5,  5,
    5,  6,  6,  6,  6,  7,  7,  7,  7,  8,  8,  8,  9,  9,  9,  10,
    10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16,
    17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25,
    25, 26, 27, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36,
    37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 50,
    51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68,
    69, 70, 72, 73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89,
    90, 92, 93, 95, 96, 98, 99, 101,102,104,105,107,109,110,112,114,
    115,117,119,120,122,124,126,127,129,131,133,135,137,138,140,142,
    144,146,148,150,152,154,156,158,160,162,164,167,169,171,173,175,
    177,180,182,184,186,189,191,193,196,198,200,203,205,208,210,213,
    215,218,220,223,225,228,231,233,236,239,241,244,247,249,252,255,
];

/// Defines the pins required for a Hub75 display
///
/// This trait is implemented for a collection of embedded-hal OutputPin types
pub trait Hub75Pins {
    type Error;

    // R1, G1, B1 are RGB pins for the top half of the display
    fn r1(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn g1(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn b1(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;

    // R2, G2, B2 are RGB pins for the bottom half of the display
    fn r2(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn g2(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn b2(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;

    // A, B, C, D, E are the row address pins (5 pins needed for 64x64 matrix)
    fn a(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn b(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn c(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn d(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn e(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;

    // CLK, LAT, OE are the control pins
    fn clk(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn lat(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;
    fn oe(&mut self) -> &mut dyn OutputPin<Error = Self::Error>;

    /// Set the row address pins based on the row number
    fn set_row(&mut self, logical_row: usize) -> Result<(), Self::Error> where <Self as Hub75Pins>::Error: embedded_hal::digital::Error {
        // For 64x64 dual-scan panels:
        // - Physical rows 0-15: Upper half (bank 0)
        // - Physical rows 16-31: Lower half (bank 1)
        let physical_row = logical_row % 16;
        let bank = (logical_row >= 16) as u8;  // 0 = top half, 1 = bottom half

        self.a().set_state((physical_row & 0x01 != 0).into())?;
        self.b().set_state((physical_row & 0x02 != 0).into())?;
        self.c().set_state((physical_row & 0x04 != 0).into())?;
        self.d().set_state((physical_row & 0x08 != 0).into())?;
        self.e().set_state((bank != 0).into())?;  // Bank select

        Ok(())
    }

    /// Set the color pins for both the top and bottom halves
    fn set_color_pins(&mut self, pixel: &DualPixel, threshold: u8) -> Result<(), Self::Error> where <Self as Hub75Pins>::Error: embedded_hal::digital::Error {
        // Set the RGB pins for both halves based on the comparison with the threshold
        if pixel.r1 > threshold { self.r1().set_high()? } else { self.r1().set_low()? }
        if pixel.g1 > threshold { self.g1().set_high()? } else { self.g1().set_low()? }
        if pixel.b1 > threshold { self.b1().set_high()? } else { self.b1().set_low()? }

        if pixel.r2 > threshold { self.r2().set_high()? } else { self.r2().set_low()? }
        if pixel.g2 > threshold { self.g2().set_high()? } else { self.g2().set_low()? }
        if pixel.b2 > threshold { self.b2().set_high()? } else { self.b2().set_low()? }

        Ok(())
    }

    /// Generate a clock pulse
    fn clock_pulse(&mut self) -> Result<(), Self::Error> where <Self as Hub75Pins>::Error: embedded_hal::digital::Error {
        self.clk().set_high()?;
        self.clk().set_low()?;
        Ok(())
    }

    /// Latch the data into the display registers
    fn latch(&mut self) -> Result<(), Self::Error> where <Self as Hub75Pins>::Error: embedded_hal::digital::Error {
        self.lat().set_high()?;
        self.lat().set_low()?;
        Ok(())
    }

    /// Enable or disable display output
    fn set_output_enabled(&mut self, enabled: bool) -> Result<(), Self::Error> where <Self as Hub75Pins>::Error: embedded_hal::digital::Error {
        if enabled {
            self.oe().set_low()? // Active low
        } else {
            self.oe().set_high()?
        }
        Ok(())
    }
}

/// Implement Hub75Pins for a tuple of pins
impl<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE> Hub75Pins
for (R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE)
where
    E: core::fmt::Debug,
    R1: OutputPin<Error = E>,
    G1: OutputPin<Error = E>,
    B1: OutputPin<Error = E>,
    R2: OutputPin<Error = E>,
    G2: OutputPin<Error = E>,
    B2: OutputPin<Error = E>,
    A: OutputPin<Error = E>,
    B: OutputPin<Error = E>,
    C: OutputPin<Error = E>,
    D: OutputPin<Error = E>,
    E0: OutputPin<Error = E>,
    CLK: OutputPin<Error = E>,
    LAT: OutputPin<Error = E>,
    OE: OutputPin<Error = E>,
{
    type Error = E;

    fn r1(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.0 }
    fn g1(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.1 }
    fn b1(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.2 }
    fn r2(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.3 }
    fn g2(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.4 }
    fn b2(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.5 }
    fn a(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.6 }
    fn b(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.7 }
    fn c(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.8 }
    fn d(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.9 }
    fn e(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.10 }
    fn clk(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.11 }
    fn lat(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.12 }
    fn oe(&mut self) -> &mut dyn OutputPin<Error = Self::Error> { &mut self.13 }
}

/// Main Hub75 driver structure
pub struct Hub75<PINS, DELAY> {
    pins: PINS,
    pub config: Hub75Config,
    framebuffer: FrameBuffer,
    phantom: PhantomData<DELAY>,
}

impl<PINS, DELAY> Hub75<PINS, DELAY>
where
    PINS: Hub75Pins,
    DELAY: DelayNs,
{
    /// Create a new Hub75 driver with default configuration
    pub fn new(pins: PINS) -> Self {
        Self::new_with_config(pins, Hub75Config::default())
    }

    /// Create a new Hub75 driver with custom configuration
    pub fn new_with_config(pins: PINS, config: Hub75Config) -> Self {
        let framebuffer = FrameBuffer::new();

        Self {
            pins,
            config,
            framebuffer,
            phantom: PhantomData,
        }
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: Hub75Config) {
        self.config = config;
    }

    /// Update the display with the current framebuffer contents
    pub fn update<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), PINS::Error> where <PINS as Hub75Pins>::Error: embedded_hal::digital::Error {
        // Only update if the framebuffer has changed
        if !self.framebuffer.is_modified() {
            return Ok(());
        }

        // Start with output disabled
        self.pins.set_output_enabled(false)?;

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
                    let (mut r1, mut g1, mut b1, mut r2, mut g2, mut b2) = (
                        pixel.r1, pixel.g1, pixel.b1,
                        pixel.r2, pixel.g2, pixel.b2
                    );

                    if self.config.use_gamma_correction {
                        r1 = GAMMA8[r1 as usize];
                        g1 = GAMMA8[g1 as usize];
                        b1 = GAMMA8[b1 as usize];
                        r2 = GAMMA8[r2 as usize];
                        g2 = GAMMA8[g2 as usize];
                        b2 = GAMMA8[b2 as usize];
                    }

                    // Apply brightness
                    let brightness = self.config.brightness as u16;
                    r1 = (r1 as u16 * brightness / 255) as u8;
                    g1 = (g1 as u16 * brightness / 255) as u8;
                    b1 = (b1 as u16 * brightness / 255) as u8;
                    r2 = (r2 as u16 * brightness / 255) as u8;
                    g2 = (g2 as u16 * brightness / 255) as u8;
                    b2 = (b2 as u16 * brightness / 255) as u8;

                    // Bit plane comparison
                    let mask = 1 << (7 - bit_plane);  // MSB first
                    let threshold = mask - 1;

                    self.pins.r1().set_state((r1 > threshold).into())?;
                    self.pins.g1().set_state((g1 > threshold).into())?;
                    self.pins.b1().set_state((b1 > threshold).into())?;
                    self.pins.r2().set_state((r2 > threshold).into())?;
                    self.pins.g2().set_state((g2 > threshold).into())?;
                    self.pins.b2().set_state((b2 > threshold).into())?;

                    self.pins.clock_pulse()?;
                }

                // Latch the data
                self.pins.latch()?;

                // Set row address
                self.pins.set_row(row)?;

                // Enable output
                self.pins.set_output_enabled(true)?;

                // Hold proportionally to the bit weight (binary coded modulation)
                // MSB (bit_position = pwm_bits-1) should be displayed longest
                let hold_time = (1 << bit_position) * self.config.row_step_time_us;
                delay.delay_us(hold_time);

                // Disable output before next bit plane
                self.pins.set_output_enabled(false)?;

                // Small delay to prevent ghosting
                delay.delay_us(1);
            }
        }

        // Mark framebuffer as updated
        self.framebuffer.reset_modified();

        Ok(())
    }

    pub fn draw_pixel_p3_mapped(&mut self, x: i32, y: i32, color: Rgb565) {
        // P3 64x64 specific mapping
        let panel_half_height = 32;
        let panel_quarter_height = 16;

        let is_top_stripe = (y % panel_half_height) < panel_quarter_height;
        let mapped_x = (x*2) + (if is_top_stripe { 1 } else { 0 });
        let mapped_y = (y / panel_half_height) * panel_quarter_height
            + y % panel_quarter_height;

        // Call the original set_pixel with mapped coordinates
        self.set_pixel(mapped_x, mapped_y, color);
    }

    /// Set a pixel in the framebuffer
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Rgb565) {
        // Convert Rgb565 to 8-bit linear scale
        let r = ((color.r() as u16 * 255) / 31) as u8;  // 5-bit -> 8-bit
        let g = ((color.g() as u16 * 255) / 63) as u8;  // 6-bit -> 8-bit
        let b = ((color.b() as u16 * 255) / 31) as u8;

        self.framebuffer.set_pixel(x as usize, y as usize, r, g, b);
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
                _ => Rgb565::new(128, 128, 0), // Darker yellow
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
}

// Implement embedded-graphics interfaces
impl<PINS, DELAY> OriginDimensions for Hub75<PINS, DELAY> {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}

impl<PINS, DELAY> DrawTarget for Hub75<PINS, DELAY>
where
    PINS: Hub75Pins,
    DELAY: DelayNs,
{
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

/// Helper functions for testing and diagnosis
pub struct Hub75Test;

impl Hub75Test {
    /// Draw a gradient test pattern
    pub fn draw_gradient<D>(display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        for y in 0..DISPLAY_HEIGHT as i32 {
            // Create gradient colors
            let blue = (y * 4) as u8;
            let color = Rgb565::new(0, 0, blue);

            // Draw horizontal line
            for x in 0..DISPLAY_WIDTH as i32 {
                display.draw_iter([Pixel(Point::new(x, y), color)])?;
            }
        }

        Ok(())
    }

    /// Draw a pattern that highlights dual scan row mapping issues
    pub fn draw_dual_scan_test<D>(display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Clear display
        display.draw_iter([Pixel(Point::new(0, 0), Rgb565::BLACK)])?;

        // Draw top half red (rows 0-31)
        for y in 0..32 {
            for x in 0..DISPLAY_WIDTH as i32 {
                display.draw_iter([Pixel(Point::new(x, y), Rgb565::RED)])?;
            }
        }

        // Draw bottom half blue (rows 32-63)
        for y in 32..64 {
            for x in 0..DISPLAY_WIDTH as i32 {
                display.draw_iter([Pixel(Point::new(x, y), Rgb565::BLUE)])?;
            }
        }

        // Draw horizontal white lines every 8 pixels
        for y in (0..64).step_by(8) {
            for x in 0..DISPLAY_WIDTH as i32 {
                display.draw_iter([Pixel(Point::new(x, y), Rgb565::WHITE)])?;
            }
        }

        // Draw vertical white lines every 8 pixels
        for x in (0..64).step_by(8) {
            for y in 0..DISPLAY_HEIGHT as i32 {
                display.draw_iter([Pixel(Point::new(x, y), Rgb565::WHITE)])?;
            }
        }

        Ok(())
    }
}
