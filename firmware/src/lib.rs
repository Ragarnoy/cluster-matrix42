#![no_std]

use cluster_core::models::Layout;
use cluster_core::types::ClusterId;
use embassy_executor::Executor;
use embassy_rp::Peri;
use embassy_rp::multicore::Stack;
use embassy_rp::peripherals::{
    DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, PIN_0, PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7,
    PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::rwlock::RwLock;
use hub75_driver::DisplayMemory;
use static_cell::StaticCell;

pub type LayoutLock = RwLock<CriticalSectionRawMutex, Layout>;

// Multicore setup
pub static mut CORE1_STACK: Stack<32768> = Stack::new();
pub static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
pub static DISPLAY_MEMORY: StaticCell<DisplayMemory> = StaticCell::new();
pub static LAYOUT: StaticCell<LayoutLock> = StaticCell::new();
pub static SELECTED_CLUSTER: StaticCell<Channel<CriticalSectionRawMutex, ClusterId, 8>> =
    StaticCell::new();

pub mod helpers;

pub struct Hub75Pins {
    // RGB data pins
    pub r1_pin: Peri<'static, PIN_0>,
    pub g1_pin: Peri<'static, PIN_1>,
    pub b1_pin: Peri<'static, PIN_2>,
    pub r2_pin: Peri<'static, PIN_3>,
    pub g2_pin: Peri<'static, PIN_4>,
    pub b2_pin: Peri<'static, PIN_5>,
    // Address pins
    pub a_pin: Peri<'static, PIN_6>,
    pub b_pin: Peri<'static, PIN_7>,
    pub c_pin: Peri<'static, PIN_8>,
    pub d_pin: Peri<'static, PIN_9>,
    pub e_pin: Peri<'static, PIN_10>,
    // Control pins. Devkit PCB swaps CLK/LAT through the level shifter,
    // so the underlying pin types differ between configurations. Stock
    // Pico 2 wiring uses PIN_11=CLK / PIN_12=LAT; devkit uses the opposite.
    #[cfg(not(feature = "devkit_remap"))]
    pub clk_pin: Peri<'static, PIN_11>,
    #[cfg(not(feature = "devkit_remap"))]
    pub lat_pin: Peri<'static, PIN_12>,
    #[cfg(feature = "devkit_remap")]
    pub clk_pin: Peri<'static, PIN_12>,
    #[cfg(feature = "devkit_remap")]
    pub lat_pin: Peri<'static, PIN_11>,
    pub oe_pin: Peri<'static, PIN_13>,
}

impl Hub75Pins {
    /// Build a `Hub75Pins` from raw peripherals, applying the board-specific
    /// CLK/LAT routing automatically based on the active features.
    ///
    /// Stock Pico 2 (default): PIN_11 → CLK, PIN_12 → LAT.
    /// Devkit (`devkit_remap`): PIN_11 → LAT, PIN_12 → CLK.
    #[allow(clippy::too_many_arguments)]
    pub fn for_board(
        pin_0: Peri<'static, PIN_0>,
        pin_1: Peri<'static, PIN_1>,
        pin_2: Peri<'static, PIN_2>,
        pin_3: Peri<'static, PIN_3>,
        pin_4: Peri<'static, PIN_4>,
        pin_5: Peri<'static, PIN_5>,
        pin_6: Peri<'static, PIN_6>,
        pin_7: Peri<'static, PIN_7>,
        pin_8: Peri<'static, PIN_8>,
        pin_9: Peri<'static, PIN_9>,
        pin_10: Peri<'static, PIN_10>,
        pin_11: Peri<'static, PIN_11>,
        pin_12: Peri<'static, PIN_12>,
        pin_13: Peri<'static, PIN_13>,
    ) -> Self {
        Self {
            r1_pin: pin_0,
            g1_pin: pin_1,
            b1_pin: pin_2,
            r2_pin: pin_3,
            g2_pin: pin_4,
            b2_pin: pin_5,
            a_pin: pin_6,
            b_pin: pin_7,
            c_pin: pin_8,
            d_pin: pin_9,
            e_pin: pin_10,
            #[cfg(not(feature = "devkit_remap"))]
            clk_pin: pin_11,
            #[cfg(not(feature = "devkit_remap"))]
            lat_pin: pin_12,
            #[cfg(feature = "devkit_remap")]
            clk_pin: pin_12,
            #[cfg(feature = "devkit_remap")]
            lat_pin: pin_11,
            oe_pin: pin_13,
        }
    }
}

pub struct DmaChannels {
    pub dma_ch0: Peri<'static, DMA_CH0>,
    pub dma_ch1: Peri<'static, DMA_CH1>,
    pub dma_ch2: Peri<'static, DMA_CH2>,
    pub dma_ch3: Peri<'static, DMA_CH3>,
}
