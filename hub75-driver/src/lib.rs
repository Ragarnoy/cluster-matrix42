#![no_std]

use core::convert::Infallible;
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor},
    Pixel,
};
use embedded_hal::{delay::DelayNs, digital::OutputPin};

/// Constants for the display dimensions
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 64;
const ACTIVE_ROWS: usize = DISPLAY_HEIGHT / 2; // Number of rows to address

/// Buffer format for dual scanning 64x64 matrix
/// Each entry represents the color values for both top and bottom pixels
#[derive(Clone, Copy, Default)]
pub struct DualPixel {
    pub r1: u8, // Red for top half
    pub g1: u8, // Green for top half
    pub b1: u8, // Blue for top half
    pub r2: u8, // Red for bottom half
    pub g2: u8, // Green for bottom half
    pub b2: u8, // Blue for bottom half
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
    pub row_step_time_us: u32,      // Delay between row updates
}

impl Default for Hub75Config {
    fn default() -> Self {
        Self {
            pwm_bits: 6,                // 6-bit PWM
            brightness: 220,            // High brightness
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

/// Generic Hub75 pins structure using static dispatch with shared error type
pub struct Hub75Pins<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>
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
    // RGB pins for the top half
    r1: R1,
    g1: G1,
    b1: B1,

    // RGB pins for the bottom half
    r2: R2,
    g2: G2,
    b2: B2,

    // Row address pins
    a: A,
    b: B,
    c: C,
    d: D,
    e: E0,

    // Control pins
    clk: CLK,
    lat: LAT,
    oe: OE,
}

impl<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>
    Hub75Pins<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>
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
    /// Create new pins structure
    pub fn new(
        r1: R1,
        g1: G1,
        b1: B1,
        r2: R2,
        g2: G2,
        b2: B2,
        a: A,
        b: B,
        c: C,
        d: D,
        e: E0,
        clk: CLK,
        lat: LAT,
        oe: OE,
    ) -> Self {
        Self {
            r1,
            g1,
            b1,
            r2,
            g2,
            b2,
            a,
            b,
            c,
            d,
            e,
            clk,
            lat,
            oe,
        }
    }

    /// Set the row address pins based on the row number
    pub fn set_row(&mut self, row: usize) -> Result<(), E> {
        // For 64x64 dual-scan panels:

        if row & 0x01 != 0 {
            self.a.set_high()?
        } else {
            self.a.set_low()?
        }
        if row & 0x02 != 0 {
            self.b.set_high()?
        } else {
            self.b.set_low()?
        }
        if row & 0x04 != 0 {
            self.c.set_high()?
        } else {
            self.c.set_low()?
        }
        if row & 0x08 != 0 {
            self.d.set_high()?
        } else {
            self.d.set_low()?
        }
        if row & 0x10 != 0 {
            self.e.set_high()?
        } else {
            self.e.set_low()?
        }

        Ok(())
    }

    /// Set the color pins for both the top and bottom halves
    pub fn set_color_pins(&mut self, pixel: &DualPixel, threshold: u8) -> Result<(), E> {
        // Set the RGB pins for both halves based on the comparison with the threshold
        if pixel.r1 > threshold {
            self.r1.set_high()?
        } else {
            self.r1.set_low()?
        }
        if pixel.g1 > threshold {
            self.g1.set_high()?
        } else {
            self.g1.set_low()?
        }
        if pixel.b1 > threshold {
            self.b1.set_high()?
        } else {
            self.b1.set_low()?
        }

        if pixel.r2 > threshold {
            self.r2.set_high()?
        } else {
            self.r2.set_low()?
        }
        if pixel.g2 > threshold {
            self.g2.set_high()?
        } else {
            self.g2.set_low()?
        }
        if pixel.b2 > threshold {
            self.b2.set_high()?
        } else {
            self.b2.set_low()?
        }

        Ok(())
    }

    /// Generate a clock pulse
    pub fn clock_pulse(&mut self) -> Result<(), E> {
        self.clk.set_high()?;
        self.clk.set_low()?;
        Ok(())
    }

    /// Latch the data into the display registers
    pub fn latch(&mut self) -> Result<(), E> {
        self.lat.set_high()?;
        self.lat.set_low()?;
        Ok(())
    }

    /// Enable or disable display output
    pub fn set_output_enabled(&mut self, enabled: bool) -> Result<(), E> {
        if enabled {
            self.oe.set_low()? // Active low
        } else {
            self.oe.set_high()?
        }
        Ok(())
    }
}

/// Main Hub75 driver structure with static dispatch
pub struct Hub75<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>
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
    pins: Hub75Pins<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>,
    pub config: Hub75Config,
    framebuffer: FrameBuffer,
}

impl<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>
    Hub75<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>
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
    /// Create a new Hub75 driver with default configuration
    pub fn new(pins: Hub75Pins<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>) -> Self {
        Self::new_with_config(pins, Hub75Config::default())
    }

    /// Create a new Hub75 driver with custom configuration
    pub fn new_with_config(
        pins: Hub75Pins<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>,
        config: Hub75Config,
    ) -> Self {
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
    pub fn update(&mut self, delay: &mut impl DelayNs) -> Result<(), E> {
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
                    let (mut r1, mut g1, mut b1, mut r2, mut g2, mut b2) =
                        (pixel.r1, pixel.g1, pixel.b1, pixel.r2, pixel.g2, pixel.b2);
                    // Apply brightness
                    let brightness = self.config.brightness as u16;
                    r1 = (r1 as u16 * brightness >> 8) as u8;
                    g1 = (g1 as u16 * brightness >> 8) as u8;
                    b1 = (b1 as u16 * brightness >> 8) as u8;
                    r2 = (r2 as u16 * brightness >> 8) as u8;
                    g2 = (g2 as u16 * brightness >> 8) as u8;
                    b2 = (b2 as u16 * brightness >> 8) as u8;

                    if self.config.use_gamma_correction {
                        r1 = GAMMA8[r1 as usize];
                        g1 = GAMMA8[g1 as usize];
                        b1 = GAMMA8[b1 as usize];
                        r2 = GAMMA8[r2 as usize];
                        g2 = GAMMA8[g2 as usize];
                        b2 = GAMMA8[b2 as usize];
                    }

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
                        r1: r1_active as u8,
                        g1: g1_active as u8,
                        b1: b1_active as u8,
                        r2: r2_active as u8,
                        g2: g2_active as u8,
                        b2: b2_active as u8,
                    };
                    self.pins.set_color_pins(&dual_pixel, 0)?;
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

    /// Set a pixel in the framebuffer
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Rgb565) {
        // Convert Rgb565 to 8-bit linear scale
        let r_original = color.r() << 3; // 5-bit -> 8-bit
        let g_original = color.g() << 2; // 6-bit -> 8-bit
        let b_original = color.b() << 3;

        // Swap the colors to match the hardware configuration
        // Based on your description: blue→green, green→red, red→blue
        let r = b_original; // Red pin receives what should be blue
        let g = r_original; // Green pin receives what should be red
        let b = g_original; // Blue pin receives what should be green

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
                        (x as usize * 32 / DISPLAY_WIDTH) as u8,
                        32,
                        (y as usize * 32 / DISPLAY_HEIGHT) as u8,
                    ),
                );
            }
        }
    }
}

// Implement embedded-graphics interfaces
impl<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE> OriginDimensions
    for Hub75<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>
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
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}

impl<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE> DrawTarget
    for Hub75<E, R1, G1, B1, R2, G2, B2, A, B, C, D, E0, CLK, LAT, OE>
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
