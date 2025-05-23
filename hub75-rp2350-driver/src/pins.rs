use embassy_rp::gpio::Output;

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

pub struct Hub75Pins {
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
    pub fn set_row(&mut self, row: usize) {
        // For 64x64 dual-scan panels:

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

    /// Set the color pins for both the top and bottom halves
    pub fn set_color_pins(&mut self, pixel: &DualPixel, threshold: u8) {
        // Set the RGB pins for both halves based on the comparison with the threshold
        if pixel.r1 > threshold {
            self.r1.set_high();
        } else {
            self.r1.set_low();
        }
        if pixel.g1 > threshold {
            self.g1.set_high();
        } else {
            self.g1.set_low();
        }
        if pixel.b1 > threshold {
            self.b1.set_high();
        } else {
            self.b1.set_low();
        }

        if pixel.r2 > threshold {
            self.r2.set_high();
        } else {
            self.r2.set_low();
        }
        if pixel.g2 > threshold {
            self.g2.set_high();
        } else {
            self.g2.set_low();
        }
        if pixel.b2 > threshold {
            self.b2.set_high();
        } else {
            self.b2.set_low();
        }
    }

    /// Generate a clock pulse
    pub fn clock_pulse(&mut self) {
        self.clk.set_high();
        self.clk.set_low();
    }

    /// Latch the data into the display registers
    pub fn latch(&mut self) {
        self.lat.set_high();
        self.lat.set_low();
    }

    /// Enable or disable display output
    pub fn set_output_enabled(&mut self, enabled: bool) {
        if enabled {
            self.oe.set_low();
        } else {
            self.oe.set_high();
        }
    }
}
