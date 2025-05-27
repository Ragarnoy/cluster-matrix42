use embassy_rp::gpio::Output;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

pub type R1 = Output<'static>;
pub type G1 = Output<'static>;
pub type B1 = Output<'static>;

pub type R2 = Output<'static>;
pub type G2 = Output<'static>;
pub type B2 = Output<'static>;

pub type A = Output<'static>;
pub type B = Output<'static>;
pub type C = Output<'static>;
pub type D = Output<'static>;
pub type E = Output<'static>;

pub type CLK = Output<'static>;
pub type LAT = Output<'static>;
pub type OE = Output<'static>;

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

// Modified Hub75Pins with your specific pin assignments
pub struct Hub75PioPins {
    // Data pins (6 consecutive: 0-5)
    data_pins: [Output<'static>; 6], // Order: R1, G1, B1, R2, G2, B2

    // Control pins (3 consecutive: 11-13)
    control_pins: [Output<'static>; 3], // Order: CLK, LAT, OE

    // Address pins (6-10)
    addr_pins: [Output<'static>; 5], // Order: A, B, C, D, E
}

impl Hub75PioPins {
    pub fn new(
        r1: Output<'static>,
        g1: Output<'static>,
        b1: Output<'static>,
        r2: Output<'static>,
        g2: Output<'static>,
        b2: Output<'static>,
        clk: Output<'static>,
        lat: Output<'static>,
        oe: Output<'static>,
        a: Output<'static>,
        b: Output<'static>,
        c: Output<'static>,
        d: Output<'static>,
        e: Output<'static>,
    ) -> Self {
        // Convert data pins to Flex (PIO-controlled)
        let data_pins = [
            r1.into(),
            g1.into(),
            b1.into(),
            r2.into(),
            g2.into(),
            b2.into(),
        ];

        // Convert control pins to Flex
        let control_pins = [clk.into(), lat.into(), oe.into()];

        // Address pins remain regular outputs
        let addr_pins = [a, b, c, d, e];

        Self {
            data_pins,
            control_pins,
            addr_pins,
        }
    }

    pub fn data_pins(&self) -> &[Output<'static>; 6] {
        &self.data_pins
    }

    pub fn control_pins(&self) -> &[Output<'static>; 3] {
        &self.control_pins
    }

    pub fn addr_pins(&self) -> &[Output<'static>; 5] {
        &self.addr_pins
    }

    // Keep existing set_row implementation
    pub fn set_row(&mut self, row: usize) {
        self.addr_pins[0]
            .set_state((row & 0x01 != 0).into())
            .unwrap();
        self.addr_pins[1]
            .set_state((row & 0x02 != 0).into())
            .unwrap();
        self.addr_pins[2]
            .set_state((row & 0x04 != 0).into())
            .unwrap();
        self.addr_pins[3]
            .set_state((row & 0x08 != 0).into())
            .unwrap();
        self.addr_pins[4]
            .set_state((row & 0x10 != 0).into())
            .unwrap();
    }
}

pub struct Hub75Pins {
    pub r1: R1,
    pub g1: G1,
    pub b1: B1,

    pub r2: R2,
    pub g2: G2,
    pub b2: B2,

    pub a: A,
    pub b: B,
    pub c: C,
    pub d: D,
    pub e: E,

    pub clk: CLK,
    pub lat: LAT,
    pub oe: OE,
}

impl Hub75Pins {
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
        e: E,
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
    #[inline]
    pub fn set_row(&mut self, row: usize) {
        // For 64x64 dual-scan panels, we have 32 addressable rows (0-31)
        // requiring 5 address lines (A-E)

        if row & 0x01 != 0 {
            self.a.set_high();
        } else {
            self.a.set_low();
        }

        if row & 0x02 != 0 {
            self.b.set_high();
        } else {
            self.b.set_low();
        }

        if row & 0x04 != 0 {
            self.c.set_high();
        } else {
            self.c.set_low();
        }

        if row & 0x08 != 0 {
            self.d.set_high();
        } else {
            self.d.set_low();
        }

        if row & 0x10 != 0 {
            self.e.set_high();
        } else {
            self.e.set_low();
        }
    }

    #[inline]
    fn set_pin_from_bit(pin: &mut Output<'static>, bit: u8) {
        if bit != 0 {
            pin.set_high();
        } else {
            pin.set_low();
        }
    }

    /// Set the color pins based on individual bit values
    pub fn set_color_bits(&mut self, r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) {
        // Set pins for top half
        Self::set_pin_from_bit(&mut self.r1, r1);
        Self::set_pin_from_bit(&mut self.g1, g1);
        Self::set_pin_from_bit(&mut self.b1, b1);

        // Set pins for bottom half
        Self::set_pin_from_bit(&mut self.r2, r2);
        Self::set_pin_from_bit(&mut self.g2, g2);
        Self::set_pin_from_bit(&mut self.b2, b2);
    }

    /// Generate a clock pulse with a configurable delay
    pub fn clock_pulse_with_delay(&mut self, delay: &mut impl DelayNs, delay_ns: u32) {
        self.clk.set_high();
        if delay_ns > 0 {
            delay.delay_ns(delay_ns);
        }
        self.clk.set_low();
        if delay_ns > 0 {
            delay.delay_ns(delay_ns);
        }
    }

    /// Latch the data with a delay
    #[inline]
    pub fn latch_with_delay(&mut self, delay: &mut impl DelayNs) {
        self.lat.set_high();
        delay.delay_ns(25); // 25ns latch pulse
        self.lat.set_low();
        delay.delay_ns(25); // Hold time after latch
    }

    /// Enable or disable display output
    #[inline]
    pub fn set_output_enabled(&mut self, enabled: bool) {
        // OE is active low - low enables output, high disables
        if enabled {
            self.oe.set_low();
        } else {
            self.oe.set_high();
        }
    }
}
