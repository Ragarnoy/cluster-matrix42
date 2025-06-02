//! PIO state machine programs and configuration for Hub75 scanning

use crate::config::*;
use defmt::error;
use embassy_rp::Peri;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{
    Config, Direction, FifoJoin::TxOnly, Pio, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};

/// PIO state machines for Hub75 control
///
/// Three coordinated state machines handle the display:
/// 1. Data SM: Shifts out pixel data with clock
/// 2. Row SM: Sets row address and latch signals  
/// 3. OE SM: Controls output enable timing for BCM
pub struct Hub75StateMachines<'d> {
    pub data_sm: StateMachine<'d, embassy_rp::peripherals::PIO0, 0>,
    pub row_sm: StateMachine<'d, embassy_rp::peripherals::PIO0, 1>,
    pub oe_sm: StateMachine<'d, embassy_rp::peripherals::PIO0, 2>,
}

impl<'d> Hub75StateMachines<'d> {
    /// Initialize all three state machines with their programs
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pio: Peri<'d, embassy_rp::peripherals::PIO0>,
        // Pin assignments
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
            mut sm2,
            ..
        } = Pio::new(pio, crate::Irqs);

        // Convert all pins to PIO pins (matching original code structure)
        let data_pins = [
            common.make_pio_pin(r1_pin),
            common.make_pio_pin(g1_pin),
            common.make_pio_pin(b1_pin),
            common.make_pio_pin(r2_pin),
            common.make_pio_pin(g2_pin),
            common.make_pio_pin(b2_pin),
        ];

        let clk_pio_pin = common.make_pio_pin(clk_pin);

        let addr_pins = [
            common.make_pio_pin(addr_a_pin),
            common.make_pio_pin(addr_b_pin),
            common.make_pio_pin(addr_c_pin),
            common.make_pio_pin(addr_d_pin),
            common.make_pio_pin(addr_e_pin),
        ];

        let lat_pio_pin = common.make_pio_pin(lat_pin);
        let oe_pio_pin = common.make_pio_pin(oe_pin);

        // IRQ usage for Hub75 PIO state machines:
        // - IRQ 4: Data SM signals row SM when line is complete
        // - IRQ 5: Row SM signals data SM to start next line
        // - IRQ 6: Row SM signals OE SM to start timing
        // - IRQ 7: OE SM signals row SM that timing is complete

        // Setup Data State Machine (SM0)
        Self::setup_data_sm(&mut common, &mut sm0, &data_pins, &clk_pio_pin);

        // Setup Row State Machine (SM1)
        Self::setup_row_sm(&mut common, &mut sm1, &addr_pins, &lat_pio_pin);

        // Setup Output Enable State Machine (SM2)
        Self::setup_oe_sm(&mut common, &mut sm2, &oe_pio_pin);

        Self {
            data_sm: sm0,
            row_sm: sm1,
            oe_sm: sm2,
        }
    }

    /// Setup the data state machine
    ///
    /// Responsible for:
    /// - Receiving pixel data from DMA
    /// - Shifting out RGB data to 6 pins
    /// - Generating pixel clock
    /// - Coordinating with row SM via IRQs
    fn setup_data_sm(
        common: &mut embassy_rp::pio::Common<'d, embassy_rp::peripherals::PIO0>,
        sm: &mut StateMachine<'d, embassy_rp::peripherals::PIO0, 0>,
        data_pins: &[embassy_rp::pio::Pin<'d, embassy_rp::peripherals::PIO0>; 6],
        clk_pin: &embassy_rp::pio::Pin<'d, embassy_rp::peripherals::PIO0>,
    ) {
        let data_program = pio_asm!(
            ".side_set 1",
            "out isr, 32    side 0b0", // Get width-1 and store in ISR
            ".wrap_target",
            "mov x isr      side 0b0", // Load width-1 into X counter
            "pixel:",
            "out pins, 8    side 0b0", // Output 8 bits of RGB data (6 used)
            "jmp x-- pixel  side 0b1", // Clock out the pixel, decrement counter
            "irq 4          side 0b0", // Tell row SM we finished this line
            "wait 1 irq 5   side 0b0", // Wait for row SM to setup next row
            ".wrap",
        );

        let data_installed = common.load_program(&data_program.program);

        let mut data_cfg = Config::default();
        data_cfg.fifo_join = TxOnly; // Use full FIFO for TX
        data_cfg.use_program(&data_installed, &[clk_pin]);

        // Convert array to slice of references
        let data_pin_refs: [&embassy_rp::pio::Pin<'d, embassy_rp::peripherals::PIO0>; 6] = [
            &data_pins[0],
            &data_pins[1],
            &data_pins[2],
            &data_pins[3],
            &data_pins[4],
            &data_pins[5],
        ];
        data_cfg.set_out_pins(&data_pin_refs);

        data_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Right,
        };
        data_cfg.clock_divider = pio_clocks::DATA_SM_CLOCK_DIV;

        sm.set_config(&data_cfg);

        // Configure pin directions
        sm.set_pin_dirs(Direction::Out, &data_pin_refs);
        sm.set_pin_dirs(Direction::Out, &[clk_pin]);

        // Send display width-1 to data SM
        if !sm.tx().try_push((DISPLAY_WIDTH - 1) as u32) {
            error!("Failed to push display width to data SM");
        }
    }

    /// Setup the row address state machine
    ///
    /// Responsible for:
    /// - Setting 5-bit row address (A-E pins)
    /// - Generating latch pulse
    /// - Coordinating with data and OE SMs via IRQs
    fn setup_row_sm(
        common: &mut embassy_rp::pio::Common<'d, embassy_rp::peripherals::PIO0>,
        sm: &mut StateMachine<'d, embassy_rp::peripherals::PIO0, 1>,
        addr_pins: &[embassy_rp::pio::Pin<'d, embassy_rp::peripherals::PIO0>; 5],
        lat_pin: &embassy_rp::pio::Pin<'d, embassy_rp::peripherals::PIO0>,
    ) {
        let row_program = pio_asm!(
            ".side_set 1",
            "pull           side 0b0", // Pull active_rows-1
            "out isr, 32    side 0b0", // Store in ISR
            "pull           side 0b0", // Pull color_bits-1
            ".wrap_target",
            "mov x, isr     side 0b0", // Load row counter
            "addr:",
            "mov pins, ~x   side 0b0", // Set inverted row address
            "mov y, osr     side 0b0", // Load bit plane counter
            "row:",
            "wait 1 irq 4   side 0b0", // Wait for data SM to finish line
            "nop            side 0b1", // Latch pulse
            "irq 6          side 0b1", // Tell OE SM to start timing
            "irq 5          side 0b0", // Tell data SM to start next line
            "wait 1 irq 7   side 0b0", // Wait for OE cycle to complete
            "jmp y-- row    side 0b0", // Next bit plane
            "jmp x-- addr   side 0b0", // Next row
            ".wrap",
        );

        let row_installed = common.load_program(&row_program.program);

        let mut row_cfg = Config::default();
        row_cfg.use_program(&row_installed, &[lat_pin]);

        // Convert array to slice of references
        let addr_pin_refs: [&embassy_rp::pio::Pin<'d, embassy_rp::peripherals::PIO0>; 5] = [
            &addr_pins[0],
            &addr_pins[1],
            &addr_pins[2],
            &addr_pins[3],
            &addr_pins[4],
        ];
        row_cfg.set_out_pins(&addr_pin_refs);

        row_cfg.clock_divider = pio_clocks::ROW_SM_CLOCK_DIV;

        sm.set_config(&row_cfg);

        // Configure pin directions
        sm.set_pin_dirs(Direction::Out, &addr_pin_refs);
        sm.set_pin_dirs(Direction::Out, &[lat_pin]);

        // Send parameters to row SM
        if !sm.tx().try_push((ACTIVE_ROWS - 1) as u32) {
            error!("Failed to push active rows to row SM");
        }

        if !sm.tx().try_push((COLOR_BITS - 1) as u32) {
            error!("Failed to push color bits to row SM");
        }
    }

    /// Setup the output enable state machine
    ///
    /// Responsible for:
    /// - Controlling OE pin timing for Binary Color Modulation
    /// - Receiving delay values from DMA
    /// - Coordinating with row SM via IRQs
    fn setup_oe_sm(
        common: &mut embassy_rp::pio::Common<'d, embassy_rp::peripherals::PIO0>,
        sm: &mut StateMachine<'d, embassy_rp::peripherals::PIO0, 2>,
        oe_pin: &embassy_rp::pio::Pin<'d, embassy_rp::peripherals::PIO0>,
    ) {
        let oe_program = pio_asm!(
            ".side_set 1",
            ".wrap_target",
            "out x, 32      side 0b1", // Get delay value, keep OE high (disabled)
            "wait 1 irq 6   side 0b1", // Wait for row SM to latch data
            "delay:",
            "jmp x-- delay  side 0b0", // Enable output for delay cycles
            "irq 7          side 0b1", // Tell row SM we're done, disable output
            ".wrap",
        );

        let oe_installed = common.load_program(&oe_program.program);

        let mut oe_cfg = Config::default();
        oe_cfg.fifo_join = TxOnly;
        oe_cfg.use_program(&oe_installed, &[oe_pin]);
        oe_cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 32,
            direction: ShiftDirection::Right,
        };
        oe_cfg.clock_divider = pio_clocks::OE_SM_CLOCK_DIV;

        sm.set_config(&oe_cfg);

        // Configure pin direction
        sm.set_pin_dirs(Direction::Out, &[oe_pin]);
    }

    /// Start all state machines
    pub fn start(&mut self) {
        self.data_sm.set_enable(true);
        self.row_sm.set_enable(true);
        self.oe_sm.set_enable(true);
    }

    /// Stop all state machines
    pub fn stop(&mut self) {
        self.data_sm.set_enable(false);
        self.row_sm.set_enable(false);
        self.oe_sm.set_enable(false);
    }
}
