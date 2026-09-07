//! Configuration constants and types for the Hub75 driver

/// Display dimensions - must match your physical panel
pub const DISPLAY_WIDTH: usize = if cfg!(feature = "size_128x128") {
    256
} else {
    64
};

pub const DISPLAY_HEIGHT: usize = if cfg!(feature = "size_64x32") { 32 } else { 64 };

/// Number of rows that need to be addressed (dual-scan panels use half)
pub const ACTIVE_ROWS: usize = DISPLAY_HEIGHT / 2; // 32 rows (requires 5 address bits)

/// Color depth in bits (affects refresh rate vs color quality trade-off)
pub const COLOR_BITS: usize = 8;

/// Total memory required for one complete frame
/// Layout: \[row]\[bit_plane]\[column] -> packed RGB data
pub const FRAME_SIZE: usize = ACTIVE_ROWS * COLOR_BITS * DISPLAY_WIDTH;

/// Brightness multiplier for BCM delays. Higher = brighter.
/// 1 = default, 2 = 2x brighter, 4 = 4x brighter, etc.
///
/// Refresh rate is unaffected as long as the top bit plane's on-time
/// (128 * BCM_BRIGHTNESS OE cycles) stays below the time the data SM
/// needs to shift one line (DISPLAY_WIDTH * 2 * DATA_SM_CLOCK_DIV sys
/// cycles, i.e. 1536 at 256px/div 3) — values above ~11 start trading
/// refresh for brightness. If overall brightness or gradient linearity
/// looks wrong, this constant and `compute_bcm_delays` are the knobs.
const BCM_BRIGHTNESS: u32 = 5;

/// Compute delay values for binary color modulation (BCM)
/// Each bit plane is displayed for 2^n * BCM_BRIGHTNESS time units.
/// The OE state machine's `jmp x-- delay` loop runs for delay+1 cycles,
/// so the -1 keeps the actual on-times exactly binary: B, 2B, 4B, ...
pub const fn compute_bcm_delays() -> [u32; COLOR_BITS] {
    let mut delays = [0u32; COLOR_BITS];
    let mut i = 0;
    while i < COLOR_BITS {
        delays[i] = (1 << i) * BCM_BRIGHTNESS - 1;
        i += 1;
    }
    delays
}

/// PIO clock dividers for different state machines
pub mod pio_clocks {
    use fixed_macro::__fixed::types::U24F8;

    pub const DATA_SM_CLOCK_DIV: U24F8 = U24F8::lit("3.0");
    pub const ROW_SM_CLOCK_DIV: U24F8 = U24F8::lit("1.0");
    pub const OE_SM_CLOCK_DIV: U24F8 = U24F8::lit("1.0");
}

/// DMA DREQ (Data Request) values for PIO0
pub mod dma_dreq {
    /// PIO0 SM0 TX FIFO data request
    pub const DATA_SM: u8 = 0; // PIO0_TX0

    /// PIO0 SM2 TX FIFO data request  
    pub const OE_SM: u8 = 2; // PIO0_TX2
}
