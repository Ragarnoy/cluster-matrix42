//! High-Performance Hub75 LED Matrix Driver for RP2350 with Embassy
//!
//! This driver achieves ~2100Hz refresh rate with zero CPU overhead using:
//! - 3 coordinated PIO state machines for pixel data, row addressing, and output enable
//! - Chained DMA for continuous operation without CPU intervention
//! - Binary Color Modulation (BCM) for smooth color gradients
//! - Double buffering for tear-free animation
//!
//! # Example
//!
//! ```rust,no_run
//! use hub75_rp2350_driver::{Hub75, DisplayMemory};
//! use embassy_rp::peripherals::*;
//!
//! // Create static display memory
//! static mut DISPLAY_MEMORY: DisplayMemory = DisplayMemory::new();
//!
//! // Initialize the driver (assuming you have the required pins)
//! let mut display = Hub75::new(
//!     pio0,                           // PIO peripheral
//!     (dma_ch0, dma_ch1, dma_ch2, dma_ch3), // DMA channels
//!     unsafe { &mut DISPLAY_MEMORY }, // Display memory
//!     r1_pin, g1_pin, b1_pin,         // Top half RGB
//!     r2_pin, g2_pin, b2_pin,         // Bottom half RGB  
//!     clk_pin,                        // Pixel clock
//!     addr_a_pin, addr_b_pin,         // Row address pins
//!     addr_c_pin, addr_d_pin, addr_e_pin,
//!     lat_pin,                        // Latch
//!     oe_pin,                         // Output enable
//! );
//!
//! // Draw pixels
//! display.set_pixel(10, 20, Rgb565::RED);
//! display.commit(); // Make changes visible
//! ```

#![no_std]

pub mod config;
pub mod dma;
pub mod lut;
pub mod memory;
pub mod pio;

pub use config::*;
use core::convert::Infallible;
use defmt::info;
pub use dma::{DmaStatus, Hub75DmaChannels};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, PIO0};
use embassy_rp::pio::{InterruptHandler, PioPin};
use embassy_rp::{Peri, bind_interrupts};
use embedded_graphics_core::prelude::RgbColor;
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::Rgb565,
};
pub use memory::DisplayMemory;
pub use pio::Hub75StateMachines;

// Bind PIO interrupts
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

/// High-performance Hub75 LED matrix driver
///
/// This driver uses a sophisticated hardware-accelerated approach:
/// - PIO state machines handle the low-level Hub75 protocol
/// - DMA provides continuous data flow without CPU intervention
/// - Double buffering enables smooth animations
/// - Binary Color Modulation provides smooth color gradients
pub struct Hub75<'d> {
    /// PIO state machines for Hub75 control
    _state_machines: Hub75StateMachines<'d>,

    /// DMA channels (stored but consumed during setup)
    #[allow(dead_code)]
    dma_fb: Peri<'d, DMA_CH0>,
    #[allow(dead_code)]
    dma_fb_loop: Peri<'d, DMA_CH1>,
    #[allow(dead_code)]
    dma_oe: Peri<'d, DMA_CH2>,
    #[allow(dead_code)]
    dma_oe_loop: Peri<'d, DMA_CH3>,

    /// Display memory with double buffering
    memory: &'static mut DisplayMemory,

    /// Global brightness control (0-255)
    brightness: u8,
}

impl<'d> Hub75<'d> {
    /// Create a new Hub75 driver instance
    ///
    /// # Arguments
    ///
    /// * `pio` - PIO0 peripheral
    /// * `dma_channels` - Tuple of 4 DMA channels (CH0-CH3)
    /// * `memory` - Static reference to display memory
    /// * Pin assignments following Hub75 standard:
    ///   - `r1_pin`, `g1_pin`, `b1_pin` - RGB for top half
    ///   - `r2_pin`, `g2_pin`, `b2_pin` - RGB for bottom half
    ///   - `clk_pin` - Pixel clock
    ///   - `addr_a_pin` through `addr_e_pin` - 5-bit row address
    ///   - `lat_pin` - Latch signal
    ///   - `oe_pin` - Output enable (active low)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pio: Peri<'d, PIO0>,
        dma_channels: (
            Peri<'d, DMA_CH0>,
            Peri<'d, DMA_CH1>,
            Peri<'d, DMA_CH2>,
            Peri<'d, DMA_CH3>,
        ),
        memory: &'static mut DisplayMemory,
        // RGB data pins
        r1_pin: Peri<'d, impl PioPin>,
        g1_pin: Peri<'d, impl PioPin>,
        b1_pin: Peri<'d, impl PioPin>,
        r2_pin: Peri<'d, impl PioPin>,
        g2_pin: Peri<'d, impl PioPin>,
        b2_pin: Peri<'d, impl PioPin>,
        // Control pins
        clk_pin: Peri<'d, impl PioPin>,
        addr_a_pin: Peri<'d, impl PioPin>,
        addr_b_pin: Peri<'d, impl PioPin>,
        addr_c_pin: Peri<'d, impl PioPin>,
        addr_d_pin: Peri<'d, impl PioPin>,
        addr_e_pin: Peri<'d, impl PioPin>,
        lat_pin: Peri<'d, impl PioPin>,
        oe_pin: Peri<'d, impl PioPin>,
    ) -> Self {
        // Initialize memory pointers to point to actual data
        memory.fb_ptr = memory.fb0.as_mut_ptr();
        memory.delay_ptr = memory.delays.as_mut_ptr();

        info!("Initializing Hub75 PIO state machines...");

        // Initialize PIO state machines
        let mut state_machines = Hub75StateMachines::new(
            pio, r1_pin, g1_pin, b1_pin, r2_pin, g2_pin, b2_pin, clk_pin, addr_a_pin, addr_b_pin,
            addr_c_pin, addr_d_pin, addr_e_pin, lat_pin, oe_pin,
        );

        info!("Starting Hub75 state machines...");

        // Start the state machines
        state_machines.start();

        // Create driver instance
        let mut driver = Self {
            _state_machines: state_machines,
            dma_fb: dma_channels.0,
            dma_fb_loop: dma_channels.1,
            dma_oe: dma_channels.2,
            dma_oe_loop: dma_channels.3,
            memory,
            brightness: 255, // Full brightness by default
        };

        info!("Initializing Hub75 DMA channels...");

        // Setup DMA after driver creation
        driver.setup_dma();
        driver
    }

    /// Set a pixel color (non-blocking)
    ///
    /// # Arguments
    /// * `x` - X coordinate (0 to 63)
    /// * `y` - Y coordinate (0 to 63)
    /// * `color` - RGB565 color value
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb565) {
        self.memory.set_pixel(x, y, color, self.brightness);
    }

    /// Commit the current drawing buffer (non-blocking)
    ///
    /// This swaps the double buffers, making the drawn frame visible
    /// and providing a fresh buffer for the next frame.
    pub fn commit(&mut self) {
        self.memory.commit();
    }

    /// Clear the drawing buffer
    ///
    /// Sets all pixels in the draw buffer to black.
    /// Call `commit()` to make the cleared display visible.
    pub fn clear(&mut self) {
        self.memory.clear();
    }

    /// Set overall brightness (0-255)
    ///
    /// This affects all subsequently drawn pixels.
    /// Existing pixels in the buffer are not affected.
    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness;
    }

    /// Get current brightness setting
    pub fn get_brightness(&self) -> u8 {
        self.brightness
    }

    /// Draw a test pattern for verification
    ///
    /// Creates a colorful test pattern to verify correct operation:
    /// - Color blocks in different regions
    /// - Gradient in one corner
    pub fn draw_test_pattern(&mut self) {
        self.clear();

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let color = match (x / (DISPLAY_WIDTH / 4), y / (DISPLAY_HEIGHT / 4)) {
                    (0, 0) => Rgb565::RED,
                    (1, 0) => Rgb565::GREEN,
                    (2, 0) => Rgb565::BLUE,
                    (3, 0) => Rgb565::WHITE,
                    (0, 1) => Rgb565::CYAN,
                    (1, 1) => Rgb565::MAGENTA,
                    (2, 1) => Rgb565::YELLOW,
                    (3, 1) => Rgb565::new(31, 31, 0), // Orange
                    _ => {
                        // Gradient in bottom half
                        let r = (x * 31 / (DISPLAY_WIDTH - 1)) as u8;
                        let g = 31;
                        let b = ((y - 32) * 31 / 31) as u8;
                        Rgb565::new(r, g, b)
                    }
                };
                self.set_pixel(x, y, color);
            }
        }
    }

    /// Get DMA status for debugging
    pub fn get_dma_status(&self) -> DmaStatus {
        let dma = embassy_rp::pac::DMA;

        DmaStatus {
            ch0_busy: dma.ch(0).ctrl_trig().read().busy(),
            ch1_busy: dma.ch(1).ctrl_trig().read().busy(),
            ch2_busy: dma.ch(2).ctrl_trig().read().busy(),
            ch3_busy: dma.ch(3).ctrl_trig().read().busy(),
            ch0_trans_count: dma.ch(0).trans_count().read().0,
            ch2_trans_count: dma.ch(2).trans_count().read().0,
        }
    }

    /// Setup DMA channels (CRITICAL: matches original exactly)
    fn setup_dma(&mut self) {
        use embassy_rp::pac::dma::regs::{ChTransCount, CtrlTrig};
        use embassy_rp::pac::dma::vals::{DataSize, TreqSel};

        let dma = embassy_rp::pac::DMA;

        // Correct DREQ values for PIO0
        let data_dreq = 0; // PIO0_TX0
        let oe_dreq = 2; // PIO0_TX2

        // Get proper FIFO addresses using the PAC
        let pio0 = embassy_rp::pac::PIO0;
        let data_fifo_addr = pio0.txf(0).as_ptr() as u32; // TX FIFO for SM0
        let oe_fifo_addr = pio0.txf(2).as_ptr() as u32; // TX FIFO for SM2

        let mut ch0_ctrl = CtrlTrig(0);
        ch0_ctrl.set_incr_read(true);
        ch0_ctrl.set_incr_write(false);
        ch0_ctrl.set_data_size(DataSize::SIZE_WORD);
        ch0_ctrl.set_treq_sel(TreqSel::from_bits(data_dreq));
        ch0_ctrl.set_chain_to(1);
        ch0_ctrl.set_irq_quiet(true);
        ch0_ctrl.set_en(true); // Enable yet !
        // Channel 0: Transfer framebuffer data to data_sm
        dma.ch(0).al1_ctrl().write_value(ch0_ctrl.0);

        dma.ch(0).read_addr().write_value(self.memory.fb_ptr as u32);
        dma.ch(0)
            .trans_count()
            .write_value(ChTransCount((FRAME_SIZE / 4) as u32));
        dma.ch(0).write_addr().write_value(data_fifo_addr);

        let mut ch1_ctrl = CtrlTrig(0);
        ch1_ctrl.set_incr_read(false);
        ch1_ctrl.set_incr_write(false);
        ch1_ctrl.set_data_size(DataSize::SIZE_WORD);
        ch1_ctrl.set_treq_sel(TreqSel::PERMANENT);
        ch1_ctrl.set_chain_to(0);
        ch1_ctrl.set_irq_quiet(true);
        ch1_ctrl.set_en(false); // Don't enable yet
        // Channel 1: Reset channel 0's read address
        dma.ch(1).al1_ctrl().write_value(ch1_ctrl.0);

        // DMA channel 1 needs to read the current value of fb_ptr to reset channel 0's read address
        // Safety: fb_ptr is part of 'static memory and won't move. The DMA will only read this address.
        let fb_ptr_addr = &self.memory.fb_ptr as *const _ as u32;
        dma.ch(1).read_addr().write_value(fb_ptr_addr);
        dma.ch(1)
            .write_addr()
            .write_value(dma.ch(0).read_addr().as_ptr() as u32);
        dma.ch(1).trans_count().write_value(ChTransCount(1));

        let mut ch2_ctrl = CtrlTrig(0);
        ch2_ctrl.set_incr_read(true);
        ch2_ctrl.set_incr_write(false);
        ch2_ctrl.set_data_size(DataSize::SIZE_WORD);
        ch2_ctrl.set_treq_sel(TreqSel::from_bits(oe_dreq));
        ch2_ctrl.set_chain_to(3);
        ch2_ctrl.set_irq_quiet(true);
        ch2_ctrl.set_en(false); // Don't enable yet

        // Channel 2: Transfer delay values to oe_sm
        dma.ch(2).al1_ctrl().write_value(ch2_ctrl.0);

        dma.ch(2)
            .read_addr()
            .write_value(self.memory.delays.as_ptr() as u32);
        dma.ch(2).write_addr().write_value(oe_fifo_addr);
        dma.ch(2)
            .trans_count()
            .write_value(ChTransCount(COLOR_BITS as u32));

        // Channel 3: Reset channel 2's read address
        let mut ch3_ctrl = CtrlTrig(0);
        ch3_ctrl.set_incr_read(false);
        ch3_ctrl.set_incr_write(false);
        ch3_ctrl.set_data_size(DataSize::SIZE_WORD);
        ch3_ctrl.set_treq_sel(TreqSel::PERMANENT);
        ch3_ctrl.set_chain_to(2);
        ch3_ctrl.set_irq_quiet(true);
        ch3_ctrl.set_en(false); // Don't enable yet
        // Channel 3: Reset channel 2's read address
        dma.ch(3).al1_ctrl().write_value(ch3_ctrl.0);

        // DMA channel 3 needs to read the current value of delay_ptr to reset channel 2's read address
        // Safety: delay_ptr is part of 'static memory and won't move. The DMA will only read this address.
        let delay_ptr_addr = &self.memory.delay_ptr as *const _ as u32;
        dma.ch(3).read_addr().write_value(delay_ptr_addr);
        dma.ch(3)
            .write_addr()
            .write_value(dma.ch(2).read_addr().as_ptr() as u32);
        dma.ch(3).trans_count().write_value(ChTransCount(1));

        // Enable all channels
        dma.ch(1).ctrl_trig().modify(|w| w.set_en(true));
        dma.ch(3).ctrl_trig().modify(|w| w.set_en(true));

        dma.ch(0).ctrl_trig().modify(|w| w.set_en(true));
        dma.ch(2).ctrl_trig().modify(|w| w.set_en(true));
    }
}

// Implement embedded-graphics traits for easy integration
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
            if point.x >= 0 && point.y >= 0 {
                self.set_pixel(point.x as usize, point.y as usize, color);
            }
        }
        Ok(())
    }
}
