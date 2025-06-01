//! High-Performance Hub75 LED Matrix Driver for RP2350 with Embassy
//!
//! Achieves ~2100Hz refresh rate with zero CPU overhead using:
//! - 3 coordinated PIO state machines
//! - Chained DMA for continuous operation
//! - Binary color modulation
//! - Double buffering

#![no_std]

use core::convert::Infallible;
use defmt::error;
use embassy_rp::pac::dma::regs::{ChTransCount, CtrlTrig};
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

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 64;
const ACTIVE_ROWS: usize = DISPLAY_HEIGHT / 2; // 32 rows (requires 5 address bits)
const COLOR_BITS: usize = 12;

// Memory layout: [row][bit_plane][column] -> packed RGB data
const FRAME_SIZE: usize = ACTIVE_ROWS * COLOR_BITS * DISPLAY_WIDTH;

/// Compute delay values for binary color modulation
const fn compute_delays() -> [u32; COLOR_BITS] {
    let mut delays = [0u32; COLOR_BITS];
    let mut i = 0;
    while i < COLOR_BITS {
        delays[i] = (1 << i) - 1; // 0, 1, 3, 7, 15, 31, 63, 127
        i += 1;
    }
    delays
}

/// Double-buffered framebuffer with hardware-optimized layout
pub struct DisplayMemory {
    // Double buffers - each byte contains RGB data for 2 pixels
    fb0: [u8; FRAME_SIZE],
    fb1: [u8; FRAME_SIZE],
    // DMA will read this pointer to know which buffer to use
    fb_ptr: *mut u8,
    // Delay values for binary color modulation
    delays: [u32; COLOR_BITS],
    delay_ptr: *mut u32,
    current_buffer: bool, // false = fb0, true = fb1
}

impl DisplayMemory {
    pub const fn new() -> Self {
        let mut fb0 = [0u8; FRAME_SIZE];
        let mut fb1 = [0u8; FRAME_SIZE];
        let mut delays = compute_delays();
        Self {
            fb_ptr: fb0.as_mut_ptr(),
            fb0,
            fb1,
            delays,
            delay_ptr: delays.as_mut_ptr(),
            current_buffer: false,
        }
    }

    /// Get the currently inactive buffer for drawing
    fn get_draw_buffer(&mut self) -> &mut [u8; FRAME_SIZE] {
        if self.current_buffer {
            &mut self.fb0
        } else {
            &mut self.fb1
        }
    }

    /// Commit the drawn buffer and make it active
    pub fn commit(&mut self) {
        // Switch buffers
        self.current_buffer = !self.current_buffer;

        // Update pointer for DMA
        self.fb_ptr = if self.current_buffer {
            self.fb1.as_mut_ptr()
        } else {
            self.fb0.as_mut_ptr()
        };

        // Clear the new draw buffer
        self.get_draw_buffer().fill(0);
    }

    /// Set a pixel in the draw buffer
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb565, brightness: u8) {
        // invert the screen
        // let x = DISPLAY_WIDTH - 1 - x;
        // let y = DISPLAY_HEIGHT - 1 - y;
        // Half of the screen
        let h = y > (DISPLAY_HEIGHT / 2) - 1;
        let shift = if h { 3 } else { 0 };

        let c_b: u16 = ((color.r() as f32) * (brightness as f32 / 255f32)) as u16;
        let c_g: u16 = ((color.g() as f32) * (brightness as f32 / 255f32)) as u16;
        let c_r: u16 = ((color.b() as f32) * (brightness as f32 / 255f32)) as u16;
        let base_idx = x + ((y % (DISPLAY_HEIGHT / 2)) * DISPLAY_WIDTH * COLOR_BITS);
        for b in 0..COLOR_BITS {
            // Extract the n-th bit of each component of the color and pack them
            let cr = c_r >> b & 0b1;
            let cg = c_g >> b & 0b1;
            let cb = c_b >> b & 0b1;
            let packed_rgb = (cb << 2 | cg << 1 | cr) as u8;
            let idx = base_idx + b * DISPLAY_WIDTH;
            if self.fb_ptr == self.fb0.as_mut_ptr() {
                self.fb1[idx] &= !(0b111 << shift);
                self.fb1[idx] |= packed_rgb << shift;
            } else {
                self.fb0[idx] &= !(0b111 << shift);
                self.fb0[idx] |= packed_rgb << shift;
            }
        }
    }

    pub fn clear(&mut self) {
        self.get_draw_buffer().fill(0);
    }
}

/// High-performance Hub75 driver with three PIO state machines
pub struct Hub75<'d> {
    // PIO state machines
    data_sm: StateMachine<'d, PIO0, 0>,
    row_sm: StateMachine<'d, PIO0, 1>,
    oe_sm: StateMachine<'d, PIO0, 2>,

    // DMA channels (will be consumed during setup)
    dma_fb: Peri<'d, DMA_CH0>,
    dma_fb_loop: Peri<'d, DMA_CH1>,
    dma_oe: Peri<'d, DMA_CH2>,
    dma_oe_loop: Peri<'d, DMA_CH3>,

    // Display memory
    memory: &'static mut DisplayMemory,

    // Configuration
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
        // PIO-controlled pins
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
        addr_e_pin: Peri<'d, impl PioPin>, // Added E address pin
        lat_pin: Peri<'d, impl PioPin>,
        oe_pin: Peri<'d, impl PioPin>,
    ) -> Self {
        let Pio {
            mut common,
            mut sm0,
            mut sm1,
            mut sm2,
            ..
        } = Pio::new(pio, Irqs);

        // Initialize memory pointers
        memory.fb_ptr = memory.fb0.as_mut_ptr();
        memory.delay_ptr = memory.delays.as_mut_ptr();

        // ===== DATA STATE MACHINE =====
        let data_program = pio_asm!(
            ".side_set 1",
            "out isr, 32    side 0b0",
            ".wrap_target",
            "mov x isr      side 0b0",
            // Wait for the row program to set the ADDR pins
            "pixel:",
            "out pins, 8    side 0b0",
            "jmp x-- pixel  side 0b1", // clock out the pixel
            "irq 4          side 0b0", // tell the row program to set the next row
            "wait 1 irq 5   side 0b0",
            ".wrap",
        );

        let data_installed = common.load_program(&data_program.program);

        let data_pins = [
            common.make_pio_pin(r1_pin),
            common.make_pio_pin(g1_pin),
            common.make_pio_pin(b1_pin),
            common.make_pio_pin(r2_pin),
            common.make_pio_pin(g2_pin),
            common.make_pio_pin(b2_pin),
        ];
        let clk_pio_pin = common.make_pio_pin(clk_pin);

        let mut data_cfg = Config::default();
        data_cfg.fifo_join = TxOnly;
        data_cfg.use_program(&data_installed, &[&clk_pio_pin]);
        data_cfg.set_out_pins(&[
            &data_pins[0],
            &data_pins[1],
            &data_pins[2],
            &data_pins[3],
            &data_pins[4],
            &data_pins[5],
        ]);
        data_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Right,
        };
        data_cfg.clock_divider = U24F8::from_num(2.0);

        sm0.set_config(&data_cfg);

        sm0.set_pin_dirs(
            Direction::Out,
            &[
                &data_pins[0],
                &data_pins[1],
                &data_pins[2],
                &data_pins[3],
                &data_pins[4],
                &data_pins[5],
            ],
        );
        sm0.set_pin_dirs(Direction::Out, &[&clk_pio_pin]);
        // Send width-1 to data SM
        if !sm0.tx().try_push((DISPLAY_WIDTH - 1) as u32) {
            error!("Failed to push width to SM0")
        }

        // ===== ROW STATE MACHINE =====
        let row_program = pio_asm!(
            ".side_set 1",
            "pull           side 0b0", // Pull the height / 2 into OSR
            "out isr, 32    side 0b0", // and move it to OSR
            "pull           side 0b0", // Pull the color depth - 1 into OSR
            ".wrap_target",
            "mov x, isr     side 0b0",
            "addr:",
            "mov pins, ~x   side 0b0", // Set the row address
            "mov y, osr     side 0b0",
            "row:",
            "wait 1 irq 4   side 0b0", // Wait until the data is clocked in
            "nop            side 0b1",
            "irq 6          side 0b1", // Display the latched data
            "irq 5          side 0b0", // Clock in next row
            "wait 1 irq 7   side 0b0", // Wait for the OE cycle to complete
            "jmp y-- row    side 0b0",
            "jmp x-- addr   side 0b0",
            ".wrap",
        );

        let row_installed = common.load_program(&row_program.program);

        // All 5 address pins: A, B, C, D, E
        let addr_pins = [
            common.make_pio_pin(addr_a_pin),
            common.make_pio_pin(addr_b_pin),
            common.make_pio_pin(addr_c_pin),
            common.make_pio_pin(addr_d_pin),
            common.make_pio_pin(addr_e_pin), // Added E pin
        ];
        let lat_pio_pin = common.make_pio_pin(lat_pin);

        let mut row_cfg = Config::default();
        row_cfg.use_program(&row_installed, &[&lat_pio_pin]);
        row_cfg.set_out_pins(&[
            &addr_pins[0],
            &addr_pins[1],
            &addr_pins[2],
            &addr_pins[3],
            &addr_pins[4],
        ]); // Now uses all 5 address pins
        row_cfg.clock_divider = U24F8::from_num(1.5);

        sm1.set_config(&row_cfg);
        sm1.set_pin_dirs(
            Direction::Out,
            &[
                &addr_pins[0],
                &addr_pins[1],
                &addr_pins[2],
                &addr_pins[3],
                &addr_pins[4],
            ],
        );
        sm1.set_pin_dirs(Direction::Out, &[&lat_pio_pin]);

        // Send parameters to row SM
        if !sm1.tx().try_push((ACTIVE_ROWS - 1) as u32) {
            error!("Failed to push active rows")
        } // 31 (for 32 rows)

        if !sm1.tx().try_push((COLOR_BITS - 1) as u32) {
            error!("Failed to push active colors")
        }

        // ===== OUTPUT ENABLE STATE MACHINE =====
        let oe_program = pio_asm!(
              ".side_set 1"
                ".wrap_target",
                "out x, 32      side 0b1",
                "wait 1 irq 6   side 0b1",
                "delay:",
                "jmp x-- delay  side 0b0",
                "irq 7          side 0b1",
                ".wrap",
        );

        let oe_installed = common.load_program(&oe_program.program);
        let oe_pio_pin = common.make_pio_pin(oe_pin);

        let mut oe_cfg = Config::default();
        oe_cfg.fifo_join = TxOnly;
        oe_cfg.use_program(&oe_installed, &[&oe_pio_pin]);
        oe_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Right,
        };
        oe_cfg.clock_divider = U24F8::from_num(1.5);

        sm2.set_config(&oe_cfg);
        sm2.set_pin_dirs(Direction::Out, &[&oe_pio_pin]);

        let (dma_fb, dma_fb_loop, dma_oe, dma_oe_loop) = dma_channels;
        // Store DMA channel IDs for chaining

        let mut driver = Self {
            data_sm: sm0,
            row_sm: sm1,
            oe_sm: sm2,
            memory,
            brightness: 255,
            dma_fb,
            dma_fb_loop,
            dma_oe,
            dma_oe_loop,
        };

        driver.setup_dma();
        driver.start();
        driver
    }

    fn setup_dma(&mut self) {
        use embassy_rp::pac::dma::vals::{DataSize, TreqSel};

        let dma = embassy_rp::pac::DMA;

        // Correct DREQ values for PIO0
        let data_dreq = 0; // PIO0_TX0
        let oe_dreq = 2; // PIO0_TX2

        // Get proper FIFO addresses using the PAC
        let pio0 = embassy_rp::pac::PIO0;
        let data_fifo_addr = pio0.txf(0).as_ptr() as u32; // TX FIFO for SM0
        let oe_fifo_addr = pio0.txf(2).as_ptr() as u32; // TX FIFO for SM2

        // Initialize memory pointers to point to actual data
        self.memory.fb_ptr = self.memory.fb0.as_mut_ptr();
        self.memory.delay_ptr = self.memory.delays.as_mut_ptr();

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

        // This should read the pointer to fb_ptr, not fb_ptr itself
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

        let delay_ptr_addr = &self.memory.delay_ptr as *const _ as u32;
        dma.ch(3).read_addr().write_value(delay_ptr_addr);
        dma.ch(3)
            .write_addr()
            .write_value(dma.ch(2).read_addr().as_ptr() as u32);
        dma.ch(3).trans_count().write_value(ChTransCount(1));
        // let mut ch3_ctrl = CtrlTrig(0);
        // ch3_ctrl.bits(dma.ch(2).regs().ch_read_addr.as_ptr() as u32);

        // dma.ch(3).al2_write_addr_trig().write_value(|w| {});

        // Enable all channels
        dma.ch(1).ctrl_trig().modify(|w| w.set_en(true));
        dma.ch(3).ctrl_trig().modify(|w| w.set_en(true));

        dma.ch(0).ctrl_trig().modify(|w| w.set_en(true));
        dma.ch(2).ctrl_trig().modify(|w| w.set_en(true));
    }

    fn start(&mut self) {
        // Start all state machines
        self.data_sm.set_enable(true);
        self.row_sm.set_enable(true);
        self.oe_sm.set_enable(true);
        let pio0 = embassy_rp::pac::PIO0;
        defmt::info!("FSTAT: 0x{:08x}", pio0.fstat().read().0);
    }

    /// Set a pixel color (non-blocking)
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb565) {
        // let r = ((color.r() as u16 * self.brightness as u16) >> 8) as u8;
        // let g = ((color.g() as u16 * self.brightness as u16) >> 8) as u8;
        // let b = ((color.b() as u16 * self.brightness as u16) >> 8) as u8;

        self.memory.set_pixel(x, y, color, self.brightness);
    }

    /// Commit the current drawing buffer (non-blocking)
    pub fn commit(&mut self) {
        self.memory.commit();
    }

    /// Clear the drawing buffer
    pub fn clear(&mut self) {
        self.memory.clear();
    }

    /// Set overall brightness (0-255)
    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness;
    }

    /// Draw a test pattern
    pub fn draw_test_pattern(&mut self) {
        self.clear();

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let color = match (x / 8, y / 8) {
                    (0, 0) => Rgb565::RED,
                    (1, 0) => Rgb565::GREEN,
                    (2, 0) => Rgb565::BLUE,
                    (3, 0) => Rgb565::WHITE,
                    (0, 1) => Rgb565::CYAN,
                    (1, 1) => Rgb565::MAGENTA,
                    (2, 1) => Rgb565::YELLOW,
                    _ => Rgb565::new(x as u8 % 32, y as u8 % 64, (x + y) as u8 % 32),
                };
                self.set_pixel(x, y, color);
            }
        }
    }
}

// Implement embedded-graphics traits
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
