//! Configuration constants and types for the Hub75 driver

/// Display dimensions - must match your physical panel
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 64;

/// Number of rows that need to be addressed (dual-scan panels use half)
pub const ACTIVE_ROWS: usize = DISPLAY_HEIGHT / 2; // 32 rows (requires 5 address bits)

/// Color depth in bits (affects refresh rate vs color quality trade-off)
pub const COLOR_BITS: usize = 8;

/// Total memory required for one complete frame
/// Layout: \[row]\[bit_plane]\[column] -> packed RGB data
pub const FRAME_SIZE: usize = ACTIVE_ROWS * COLOR_BITS * DISPLAY_WIDTH;

/// Compute delay values for binary color modulation (BCM)
/// Each bit plane is displayed for 2^n time units
pub const fn compute_bcm_delays() -> [u32; COLOR_BITS] {
    let mut delays = [0u32; COLOR_BITS];
    let mut i = 0;
    while i < COLOR_BITS {
        delays[i] = (1 << i) - 1; // 0, 1, 3, 7, 15, 31, 63, 127
        i += 1;
    }
    delays
}

/// PIO clock dividers for different state machines
pub mod pio_clocks {
    use fixed_macro::__fixed::types::U24F8;

    /// Data state machine clock divider (2.0)
    pub const DATA_SM_CLOCK_DIV: U24F8 = U24F8::from_bits(512); // 2.0 * 256

    /// Row address state machine clock divider (1.5)
    pub const ROW_SM_CLOCK_DIV: U24F8 = U24F8::from_bits(384); // 1.5 * 256

    /// Output enable state machine clock divider (1.5)
    pub const OE_SM_CLOCK_DIV: U24F8 = U24F8::from_bits(384); // 1.5 * 256
}

/// DMA DREQ (Data Request) values for PIO0
pub mod dma_dreq {
    /// PIO0 SM0 TX FIFO data request
    pub const DATA_SM: u8 = 0; // PIO0_TX0

    /// PIO0 SM2 TX FIFO data request  
    pub const OE_SM: u8 = 2; // PIO0_TX2
}
