//! Fixed Hub75 LED Matrix Driver for RP2350 with PIO + DMA
//!
//! This driver supports 64x64 LED matrices using the HUB75 protocol.
#![no_std]

pub mod pins;
use core::convert::Infallible;
use embassy_rp::gpio::Output;
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{
    Config, InterruptHandler, Pio, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use embassy_rp::{Peri, bind_interrupts};
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor},
};
use embedded_hal::delay::DelayNs;
use fixed_macro::__fixed::prelude::ToFixed;
use fixed_macro::types::U56F8;
use pins::DualPixel;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

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
    pub clock_frequency: u32,       // PIO clock frequency in Hz
}

impl Default for Hub75Config {
    fn default() -> Self {
        Self {
            pwm_bits: 4,                 // Start with 4-bit PWM for testing
            brightness: 128,             // Half brightness for testing
            use_gamma_correction: false, // Disable gamma for debugging
            clock_frequency: 2_000_000,  // 2 MHz PIO clock - slower for debugging
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

/// Main Hub75 driver structure with PIO + DMA
pub struct Hub75<'d> {
    pio_sm: StateMachine<'d, PIO0, 0>,
    dma_chan: Peri<'d, DMA_CH0>,
    config: Hub75Config,
    framebuffer: FrameBuffer,
    r_lut: [u8; 32],
    g_lut: [u8; 64],
    b_lut: [u8; 32],

    // Manual control pins (not controlled by PIO)
    addr_pins: [Output<'d>; 5], // A, B, C, D, E
    lat_pin: Output<'d>,        // Latch
    oe_pin: Output<'d>,         // Output Enable
}

impl<'d> Hub75<'d> {
    /// Create a new Hub75 driver with PIO + DMA
    pub fn new(
        pio: Peri<'d, PIO0>,
        dma_chan: Peri<'d, DMA_CH0>,
        config: Hub75Config,
        // Raw pin peripherals for PIO use
        r1_pin: Peri<'d, impl PioPin>,
        g1_pin: Peri<'d, impl PioPin>,
        b1_pin: Peri<'d, impl PioPin>,
        r2_pin: Peri<'d, impl PioPin>,
        g2_pin: Peri<'d, impl PioPin>,
        b2_pin: Peri<'d, impl PioPin>,
        clk_pin: Peri<'d, impl PioPin>,
        // Regular GPIO pins for manual control
        lat_pin: Output<'d>,
        oe_pin: Output<'d>,
        a_pin: Output<'d>,
        b_pin: Output<'d>,
        c_pin: Output<'d>,
        d_pin: Output<'d>,
        e_pin: Output<'d>,
    ) -> Self {
        let Pio {
            mut common,
            mut sm0,
            ..
        } = Pio::new(pio, Irqs);

        let program = pio_asm!(
            ".origin 0",
            ".wrap_target",
            "    out pins, 6      ", // Output 6 data bits (R1,G1,B1,R2,G2,B2)
            "    set pins, 1 [1]  ", // Clock high with delay
            "    set pins, 0 [1]  ", // Clock low with delay
            ".wrap"
        );

        let installed = common.load_program(&program.program);

        // Convert pins to PIO pins
        let data_pins = [
            common.make_pio_pin(r1_pin),
            common.make_pio_pin(g1_pin),
            common.make_pio_pin(b1_pin),
            common.make_pio_pin(r2_pin),
            common.make_pio_pin(g2_pin),
            common.make_pio_pin(b2_pin),
        ];
        let clk_pio_pin = common.make_pio_pin(clk_pin);

        // CORRECTED PIO configuration
        let mut cfg = Config::default();
        cfg.use_program(&installed, &[]);
        cfg.set_out_pins(&[
            &data_pins[0],
            &data_pins[1],
            &data_pins[2],
            &data_pins[3],
            &data_pins[4],
            &data_pins[5],
        ]);
        cfg.set_set_pins(&[&clk_pio_pin]);

        // CRITICAL: Proper shift configuration
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Left, // LEFT shift means LSB goes out first
        };
        cfg.clock_divider = (U56F8!(125_000_000) / U56F8!(5_000)).to_fixed();

        sm0.set_config(&cfg);
        sm0.set_enable(true);

        let mut driver = Self {
            pio_sm: sm0,
            dma_chan,
            config,
            framebuffer: FrameBuffer::new(),
            r_lut: [0; 32],
            g_lut: [0; 64],
            b_lut: [0; 32],
            addr_pins: [a_pin, b_pin, c_pin, d_pin, e_pin],
            lat_pin,
            oe_pin,
        };

        driver.update_luts();
        driver
    }

    fn update_luts(&mut self) {
        let brightness = self.config.brightness as u16;
        let shift = 8 - self.config.pwm_bits;
        let use_gamma = self.config.use_gamma_correction;

        // Precompute red LUT (5-bit input)
        for i in 0..32 {
            let mut val = (i as u16 * 255) / 31;
            if brightness < 255 {
                val = (val * brightness) / 255;
            }
            let val_8bit = if use_gamma {
                GAMMA8[val as usize]
            } else {
                val as u8
            };
            self.r_lut[i] = val_8bit >> shift;
        }

        // Precompute green LUT (6-bit input)
        for i in 0..64 {
            let mut val = (i as u16 * 255) / 63;
            if brightness < 255 {
                val = (val * brightness) / 255;
            }
            let val_8bit = if use_gamma {
                GAMMA8[val as usize]
            } else {
                val as u8
            };
            self.g_lut[i] = val_8bit >> shift;
        }

        // Precompute blue LUT (5-bit input)
        for i in 0..32 {
            let mut val = (i as u16 * 255) / 31;
            if brightness < 255 {
                val = (val * brightness) / 255;
            }
            let val_8bit = if use_gamma {
                GAMMA8[val as usize]
            } else {
                val as u8
            };
            self.b_lut[i] = val_8bit >> shift;
        }
    }

    /// Set row address pins
    fn set_row(&mut self, row: usize) {
        for (i, pin) in self.addr_pins.iter_mut().enumerate() {
            if row & (1 << i) != 0 {
                pin.set_high();
            } else {
                pin.set_low();
            }
        }
    }

    /// Control latch pin
    fn latch(&mut self, delay: &mut impl DelayNs) {
        self.lat_pin.set_high();
        delay.delay_ns(100); // Longer latch pulse
        self.lat_pin.set_low();
        delay.delay_ns(100);
    }

    /// Control output enable pin
    fn set_output_enabled(&mut self, enabled: bool) {
        if enabled {
            self.oe_pin.set_low(); // OE is active low
        } else {
            self.oe_pin.set_high();
        }
    }

    pub async fn update(&mut self, delay: &mut impl DelayNs) -> Result<(), Infallible> {
        if !self.framebuffer.is_modified() {
            return Ok(());
        }

        // Process each bit plane (for PWM)
        for bit_plane in 0..self.config.pwm_bits {
            // Process each row pair (Hub75 addresses 32 rows for 64-high display)
            for row in 0..ACTIVE_ROWS {
                // STEP 1: Disable output while updating
                self.set_output_enabled(false);

                // STEP 2: Set row address FIRST
                self.set_row(row);

                // STEP 3: Prepare and send pixel data for this row
                let mut row_data = [0u32; DISPLAY_WIDTH];

                for (col, pixel) in self.framebuffer.buffer[row].iter().enumerate() {
                    // Extract bit for this bit plane from each color channel
                    let r1_bit = (pixel.r1 >> bit_plane) & 1;
                    let g1_bit = (pixel.g1 >> bit_plane) & 1;
                    let b1_bit = (pixel.b1 >> bit_plane) & 1;
                    let r2_bit = (pixel.r2 >> bit_plane) & 1;
                    let g2_bit = (pixel.g2 >> bit_plane) & 1;
                    let b2_bit = (pixel.b2 >> bit_plane) & 1;

                    // Pack bits into correct order (LSB first due to LEFT shift)
                    // Bit 0 = R1, Bit 1 = G1, Bit 2 = B1, Bit 3 = R2, Bit 4 = G2, Bit 5 = B2
                    let bits = r1_bit
                        | (g1_bit << 1)
                        | (b1_bit << 2)
                        | (r2_bit << 3)
                        | (g2_bit << 4)
                        | (b2_bit << 5);

                    // Put bits in lower 6 bits of 32-bit word
                    row_data[col] = bits as u32;
                }

                // STEP 4: Use DMA to send all row data at once
                let tx = self.pio_sm.tx();
                tx.dma_push(self.dma_chan.reborrow(), &row_data, false)
                    .await;

                // STEP 5: Latch the data into the display registers
                self.latch(delay);

                // STEP 6: Enable output for PWM timing
                self.set_output_enabled(true);

                // STEP 7: Hold for time proportional to bit weight (Binary Code Modulation)
                let hold_time = 1u32 << bit_plane; // 1, 2, 4, 8, 16... microseconds
                delay.delay_us(hold_time);
            }
        }

        self.framebuffer.reset_modified();
        Ok(())
    }

    /// Set a pixel in the framebuffer
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Rgb565) {
        if x < 0 || y < 0 || x >= DISPLAY_WIDTH as i32 || y >= DISPLAY_HEIGHT as i32 {
            return;
        }

        let r = self.r_lut[color.r() as usize];
        let g = self.g_lut[color.g() as usize];
        let b = self.b_lut[color.b() as usize];

        self.framebuffer.set_pixel(x as usize, y as usize, r, g, b);
    }

    /// Clear the framebuffer
    pub fn clear(&mut self) {
        self.framebuffer.clear();
    }

    /// Simple test: draw a few colored pixels
    pub fn draw_debug_pattern(&mut self) {
        self.clear();

        // Draw some simple test pixels
        for x in 0..8 {
            // Top row - red pixels
            self.set_pixel(x, 0, Rgb565::RED);
            // Second row - green pixels
            self.set_pixel(x, 1, Rgb565::GREEN);
            // Third row - blue pixels
            self.set_pixel(x, 2, Rgb565::BLUE);
            // Fourth row - white pixels
            self.set_pixel(x, 3, Rgb565::WHITE);
        }

        // Draw some pixels in bottom half
        for x in 0..8 {
            self.set_pixel(x, 32, Rgb565::CYAN); // Should appear on row 0 of bottom half
            self.set_pixel(x, 33, Rgb565::MAGENTA); // Should appear on row 1 of bottom half
            self.set_pixel(x, 34, Rgb565::YELLOW); // Should appear on row 2 of bottom half
        }
    }

    // Keep existing test methods
    pub fn draw_test_pattern(&mut self) {
        self.clear();

        for y in 0..DISPLAY_HEIGHT {
            let color = match (y / 8) % 8 {
                0 => Rgb565::RED,
                1 => Rgb565::GREEN,
                2 => Rgb565::BLUE,
                3 => Rgb565::CYAN,
                4 => Rgb565::MAGENTA,
                5 => Rgb565::YELLOW,
                6 => Rgb565::WHITE,
                _ => Rgb565::new(15, 31, 15),
            };

            for x in 0..DISPLAY_WIDTH {
                self.set_pixel(x as i32, y as i32, color);
            }
        }
    }

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
}

// Implement embedded-graphics interfaces
impl<'d> OriginDimensions for Hub75<'d> {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}

impl<'d> DrawTarget for Hub75<'d> {
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
