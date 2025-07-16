#![no_std]

use cluster_core::models::Layout;
use embassy_executor::Executor;
use embassy_rp::Peri;
use embassy_rp::multicore::Stack;
use embassy_rp::peripherals::{
    DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, PIN_0, PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7,
    PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::rwlock::RwLock;
use hub75_rp2350_driver::DisplayMemory;
use static_cell::StaticCell;

// Multicore setup
pub static mut CORE1_STACK: Stack<4096> = Stack::new();
pub static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
// Static memory for the display - required for the driver
pub static DISPLAY_MEMORY: StaticCell<DisplayMemory> = StaticCell::new();
pub static LAYOUT: StaticCell<RwLock<CriticalSectionRawMutex, Layout>> = StaticCell::new();

// Pin grouping structures to reduce parameter count
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
    // Control pins
    pub clk_pin: Peri<'static, PIN_11>,
    pub lat_pin: Peri<'static, PIN_12>,
    pub oe_pin: Peri<'static, PIN_13>,
}

pub struct DmaChannels {
    pub dma_ch0: Peri<'static, DMA_CH0>,
    pub dma_ch1: Peri<'static, DMA_CH1>,
    pub dma_ch2: Peri<'static, DMA_CH2>,
    pub dma_ch3: Peri<'static, DMA_CH3>,
}
