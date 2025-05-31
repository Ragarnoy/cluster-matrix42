//! High-Performance Hub75 LED Matrix Driver for RP2350 with Embassy
//!
//! Simplified and fixed version with better synchronization

#![no_std]

use core::convert::Infallible;
use embassy_rp::pac::dma::regs::ChTransCount;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, PIO0};
use embassy_rp::pio::FifoJoin::TxOnly;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{
    Config, Direction, InterruptHandler, Pio, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use embassy_rp::{Peri, bind_interrupts};
use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor},
};
use fixed_macro::__fixed::types::U24F8;
use fixed_macro::types::U24F8;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 64;
const ACTIVE_ROWS: usize = DISPLAY_HEIGHT / 2;
const COLOR_BITS: usize = 6; // Start with 6-bit color for easier debugging

// Simple memory layout: one byte per pixel pair (top and bottom)
// Bit layout: [B2 G2 R2 B1 G1 R1] for each column
const BYTES_PER_ROW: usize = DISPLAY_WIDTH;
const BYTES_PER_BITPLANE: usize = ACTIVE_ROWS * BYTES_PER_ROW;
const FRAME_SIZE: usize = COLOR_BITS * BYTES_PER_BITPLANE;

/// BCM delay values
const fn compute_delays() -> [u32; COLOR_BITS] {
    let mut delays = [0u32; COLOR_BITS];
    let mut i = 0;
    while i < COLOR_BITS {
        delays[i] = (1 << i) * 2; // Scale for visibility
        i += 1;
    }
    delays
}

/// Display memory with double buffering
pub struct DisplayMemory {
    // Frame buffers
    fb0: [u8; FRAME_SIZE],
    fb1: [u8; FRAME_SIZE],
    fb_ptr: *const u8,

    // BCM delays
    delays: [u32; COLOR_BITS],
    delay_ptr: *const u32,

    // Current buffer for drawing
    current_buffer: bool,
}

impl DisplayMemory {
    pub const fn new() -> Self {
        let delays = compute_delays();
        Self {
            fb0: [0u8; FRAME_SIZE],
            fb1: [0u8; FRAME_SIZE],
            fb_ptr: core::ptr::null(),
            delays,
            delay_ptr: core::ptr::null(),
            current_buffer: false,
        }
    }

    pub fn init(&mut self) {
        self.fb_ptr = self.fb0.as_ptr();
        self.delay_ptr = self.delays.as_ptr();
    }

    fn get_draw_buffer(&mut self) -> &mut [u8; FRAME_SIZE] {
        if self.current_buffer {
            &mut self.fb0
        } else {
            &mut self.fb1
        }
    }

    pub fn commit(&mut self) {
        self.current_buffer = !self.current_buffer;
        self.fb_ptr = if self.current_buffer {
            self.fb1.as_ptr()
        } else {
            self.fb0.as_ptr()
        };
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return;
        }

        let row = y % ACTIVE_ROWS;
        let is_bottom = y >= ACTIVE_ROWS;
        let buffer = self.get_draw_buffer();

        // Process each bit plane
        for bit in 0..COLOR_BITS {
            let idx = bit * BYTES_PER_BITPLANE + row * BYTES_PER_ROW + x;

            let r_bit = (r >> (7 - bit)) & 1; // Start from MSB
            let g_bit = (g >> (7 - bit)) & 1;
            let b_bit = (b >> (7 - bit)) & 1;

            if is_bottom {
                // Bottom half: bits 5:3 = B2 G2 R2
                buffer[idx] &= 0b11000111; // Clear bits 5:3
                buffer[idx] |= (b_bit << 5) | (g_bit << 4) | (r_bit << 3);
            } else {
                // Top half: bits 2:0 = B1 G1 R1
                buffer[idx] &= 0b11111000; // Clear bits 2:0
                buffer[idx] |= (b_bit << 2) | (g_bit << 1) | r_bit;
            }
        }
    }

    pub fn clear(&mut self) {
        self.get_draw_buffer().fill(0);
    }
}

/// Hub75 driver with PIO state machines
pub struct Hub75<'d> {
    data_sm: StateMachine<'d, PIO0, 0>,
    ctrl_sm: StateMachine<'d, PIO0, 1>,

    dma_data: Peri<'d, DMA_CH0>,
    dma_data_loop: Peri<'d, DMA_CH1>,
    dma_delay: Peri<'d, DMA_CH2>,
    dma_delay_loop: Peri<'d, DMA_CH3>,

    memory: &'static mut DisplayMemory,
    brightness: u8,
}

impl<'d> Hub75<'d> {
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
        // Pins
        r1_pin: Peri<'d, impl PioPin>,
        g1_pin: Peri<'d, impl PioPin>,
        b1_pin: Peri<'d, impl PioPin>,
        r2_pin: Peri<'d, impl PioPin>,
        g2_pin: Peri<'d, impl PioPin>,
        b2_pin: Peri<'d, impl PioPin>,
        clk_pin: Peri<'d, impl PioPin>,
        addr_a_pin: Peri<'d, impl PioPin>,
        addr_b_pin: Peri<'d, impl PioPin>,
        addr_c_pin: Peri<'d, impl PioPin>,
        addr_d_pin: Peri<'d, impl PioPin>,
        addr_e_pin: Peri<'d, impl PioPin>,
        lat_pin: Peri<'d, impl PioPin>,
        oe_pin: Peri<'d, impl PioPin>,
    ) -> Self {
        let Pio {
            mut common,
            mut sm0,
            mut sm1,
            ..
        } = Pio::new(pio, Irqs);

        memory.init();

        // ===== DATA STATE MACHINE =====
        // Shifts out pixel data with clock
        let data_program = pio_asm!(
            ".side_set 1 opt",
            ".wrap_target",
            "out pins, 6    side 0", // Output 6 bits (R1G1B1R2G2B2)
            "nop            side 1", // Clock high
            ".wrap",
        );

        let data_installed = common.load_program(&data_program.program);

        // Setup pins
        let r1 = common.make_pio_pin(r1_pin);
        let g1 = common.make_pio_pin(g1_pin);
        let b1 = common.make_pio_pin(b1_pin);
        let r2 = common.make_pio_pin(r2_pin);
        let g2 = common.make_pio_pin(g2_pin);
        let b2 = common.make_pio_pin(b2_pin);
        let clk = common.make_pio_pin(clk_pin);

        let mut data_cfg = Config::default();
        data_cfg.fifo_join = TxOnly;
        data_cfg.use_program(&data_installed, &[&clk]);
        data_cfg.set_out_pins(&[&r1, &g1, &b1, &r2, &g2, &b2]);
        data_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 8,
            direction: ShiftDirection::Right,
        };
        data_cfg.clock_divider = U24F8::from_num(4.0); // Slower for debugging

        sm0.set_config(&data_cfg);
        sm0.set_pin_dirs(Direction::Out, &[&r1, &g1, &b1, &r2, &g2, &b2]);
        sm0.set_pin_dirs(Direction::Out, &[&clk]);

        // ===== CONTROL STATE MACHINE =====
        // Manages row addressing, latching, and OE
        let ctrl_program = pio_asm!(
            ".side_set 2",                       // LAT on bit 0, OE on bit 1
            "set x, 5                side 0b11", // 6 bit planes (x = COLOR_BITS - 1), OE high
            "bitplane:",
            "set y, 31               side 0b11", // 32 rows (y = ACTIVE_ROWS - 1)
            "row:",
            "out pins, 5             side 0b10", // Output row address, OE high, LAT low
            "out null, 27            side 0b10", // Discard rest, keep outputting pixels
            // Pixels are being shifted out by SM0 in parallel
            "set pins, 0             side 0b11  [7]", // Brief LAT pulse, OE high
            "out exec, 16            side 0b01  [7]", // Pull and exec delay instruction, OE low
            "jmp y-- row             side 0b11",      // Next row, OE high
            "jmp x-- bitplane        side 0b11",      // Next bit plane
            ".wrap",
        );

        let ctrl_installed = common.load_program(&ctrl_program.program);

        // Setup control pins
        let addr_a = common.make_pio_pin(addr_a_pin);
        let addr_b = common.make_pio_pin(addr_b_pin);
        let addr_c = common.make_pio_pin(addr_c_pin);
        let addr_d = common.make_pio_pin(addr_d_pin);
        let addr_e = common.make_pio_pin(addr_e_pin);
        let lat = common.make_pio_pin(lat_pin);
        let oe = common.make_pio_pin(oe_pin);

        let mut ctrl_cfg = Config::default();
        ctrl_cfg.fifo_join = TxOnly;
        ctrl_cfg.use_program(&ctrl_installed, &[&lat, &oe]);
        ctrl_cfg.set_out_pins(&[&addr_a, &addr_b, &addr_c, &addr_d, &addr_e]);
        ctrl_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Right,
        };
        ctrl_cfg.clock_divider = U24F8::from_num(4.0);

        sm1.set_config(&ctrl_cfg);
        sm1.set_pin_dirs(
            Direction::Out,
            &[&addr_a, &addr_b, &addr_c, &addr_d, &addr_e],
        );
        sm1.set_pin_dirs(Direction::Out, &[&lat, &oe]);

        let (dma_data, dma_data_loop, dma_delay, dma_delay_loop) = dma_channels;

        let mut driver = Self {
            data_sm: sm0,
            ctrl_sm: sm1,
            memory,
            brightness: 255,
            dma_data,
            dma_data_loop,
            dma_delay,
            dma_delay_loop,
        };

        driver.setup_dma();
        driver.start();
        driver
    }

    fn setup_dma(&mut self) {
        use embassy_rp::pac::dma::vals::{DataSize, TreqSel};

        let dma = embassy_rp::pac::DMA;
        let pio0 = embassy_rp::pac::PIO0;

        // Data goes to SM0, control data (including delays) goes to SM1
        let data_fifo = pio0.txf(0).as_ptr() as u32;
        let ctrl_fifo = pio0.txf(1).as_ptr() as u32;

        // Generate control data including delays
        let mut ctrl_data = [0u32; ACTIVE_ROWS * COLOR_BITS + COLOR_BITS];
        let mut idx = 0;

        for bit_plane in 0..COLOR_BITS {
            for row in 0..ACTIVE_ROWS {
                // Row address in bits 0-4, rest is padding
                ctrl_data[idx] = row as u32;
                idx += 1;
            }
            // Add delay instruction for this bit plane
            let delay = self.memory.delays[bit_plane];
            // PIO "out exec" will execute this as a delay loop
            ctrl_data[idx] = 0x6000 | (delay & 0x1f); // "set x, delay"
            idx += 1;
        }

        // Store control data in unused part of fb0 or allocate separately
        // For now, we'll use immediate values

        // Channel 0: Transfer framebuffer to data SM
        dma.ch(0).ctrl_trig().write(|w| {
            w.set_incr_read(true);
            w.set_incr_write(false);
            w.set_data_size(DataSize::SIZE_BYTE);
            w.set_treq_sel(TreqSel::from_bits(0)); // PIO0_TX0
            w.set_chain_to(1);
            w.set_en(false);
        });

        dma.ch(0).read_addr().write_value(self.memory.fb_ptr as u32);
        dma.ch(0).write_addr().write_value(data_fifo);
        dma.ch(0)
            .trans_count()
            .write_value(ChTransCount(FRAME_SIZE as u32));

        // Channel 1: Loop back
        dma.ch(1).ctrl_trig().write(|w| {
            w.set_incr_read(false);
            w.set_incr_write(false);
            w.set_data_size(DataSize::SIZE_WORD);
            w.set_treq_sel(TreqSel::PERMANENT);
            w.set_chain_to(0);
            w.set_en(false);
        });

        let fb_ptr_addr = &self.memory.fb_ptr as *const _ as u32;
        dma.ch(1).read_addr().write_value(fb_ptr_addr);
        dma.ch(1)
            .write_addr()
            .write_value(dma.ch(0).read_addr().as_ptr() as u32);
        dma.ch(1).trans_count().write_value(ChTransCount(1));

        // For now, let's skip the control DMA and test with just data
        // Enable data DMA
        dma.ch(1).ctrl_trig().modify(|w| w.set_en(true));
        dma.ch(0).ctrl_trig().modify(|w| w.set_en(true));
    }

    fn start(&mut self) {
        // Clear debug flags
        let pio0 = embassy_rp::pac::PIO0;
        pio0.fdebug().write(|w| w.0 = 0xFFFFFFFF);

        // Start state machines
        self.data_sm.set_enable(true);
        self.ctrl_sm.set_enable(true);

        // For testing, manually drive the control SM
        // Send dummy row addresses to keep it running
        for _ in 0..32 {
            if !self.ctrl_sm.tx().try_push(0) {
                break;
            }
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb565) {
        let r = ((color.r() as u16 * 255 / 31 * self.brightness as u16) >> 8) as u8;
        let g = ((color.g() as u16 * 255 / 63 * self.brightness as u16) >> 8) as u8;
        let b = ((color.b() as u16 * 255 / 31 * self.brightness as u16) >> 8) as u8;

        self.memory.set_pixel(x, y, r, g, b);
    }

    pub fn commit(&mut self) {
        self.memory.commit();
    }

    pub fn clear(&mut self) {
        self.memory.clear();
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness;
    }

    pub fn draw_test_pattern(&mut self) {
        self.clear();

        // Simple test: light up specific pixels to verify addressing
        // Single pixels in corners
        self.set_pixel(0, 0, Rgb565::RED); // Top-left
        self.set_pixel(63, 0, Rgb565::GREEN); // Top-right
        self.set_pixel(0, 63, Rgb565::BLUE); // Bottom-left
        self.set_pixel(63, 63, Rgb565::WHITE); // Bottom-right

        // Draw lines to verify row/column mapping
        for i in 0..16 {
            self.set_pixel(i * 4, 0, Rgb565::YELLOW); // Top row
            self.set_pixel(0, i * 4, Rgb565::MAGENTA); // Left column
            self.set_pixel(i * 4, 32, Rgb565::CYAN); // Middle row
        }

        // Fill quadrants with dim colors to see boundaries
        for y in 16..32 {
            for x in 16..32 {
                self.set_pixel(x, y, Rgb565::new(8, 0, 0)); // Dim red
                self.set_pixel(x + 16, y, Rgb565::new(0, 16, 0)); // Dim green
                self.set_pixel(x, y + 16, Rgb565::new(0, 0, 8)); // Dim blue
                self.set_pixel(x + 16, y + 16, Rgb565::new(8, 16, 8)); // Dim white
            }
        }
    }
}

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
