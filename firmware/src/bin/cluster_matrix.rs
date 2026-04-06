//! Production firmware for the cluster-matrix display

#![no_std]
#![no_main]

use cluster_core::models::Layout;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::peripherals::*;
use embassy_rp::{Peri, gpio};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::rwlock::RwLock;
use embassy_time::{Duration, Timer};
use firmware::{DISPLAY_MEMORY, DmaChannels, Hub75Pins};
use graphics_common::animations;
use hub75_driver::{DisplayMemory, Hub75};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let pins = Hub75Pins {
        r1_pin: p.PIN_0,
        g1_pin: p.PIN_1,
        b1_pin: p.PIN_2,
        r2_pin: p.PIN_3,
        g2_pin: p.PIN_4,
        b2_pin: p.PIN_5,
        a_pin: p.PIN_6,
        b_pin: p.PIN_7,
        c_pin: p.PIN_8,
        d_pin: p.PIN_9,
        e_pin: p.PIN_10,
        clk_pin: p.PIN_12,
        lat_pin: p.PIN_11,
        oe_pin: p.PIN_13,
    };

    let dma_channels = DmaChannels {
        dma_ch0: p.DMA_CH0,
        dma_ch1: p.DMA_CH1,
        dma_ch2: p.DMA_CH2,
        dma_ch3: p.DMA_CH3,
    };

    spawner.spawn(matrix_task(p.PIO0, dma_channels, pins).unwrap());
}

enum ErrorState {
    Network,
}
enum State {
    Init,
    Running(Layout),
    Error(ErrorState),
}

static CLUSTERS: StaticCell<RwLock<CriticalSectionRawMutex, State>> = StaticCell::new();

#[embassy_executor::task]
async fn matrix_task(pio: Peri<'static, PIO0>, dma_channels: DmaChannels, pins: Hub75Pins) {
    info!("Starting Hub75 LED matrix control with 3 PIO SMs + chained DMA");

    let mut display = Hub75::new(
        pio,
        (
            dma_channels.dma_ch0,
            dma_channels.dma_ch1,
            dma_channels.dma_ch2,
            dma_channels.dma_ch3,
        ),
        DISPLAY_MEMORY.init(DisplayMemory::new()),
        pins.r1_pin,
        pins.g1_pin,
        pins.b1_pin,
        pins.r2_pin,
        pins.g2_pin,
        pins.b2_pin,
        pins.clk_pin,
        pins.a_pin,
        pins.b_pin,
        pins.c_pin,
        pins.d_pin,
        pins.e_pin,
        pins.lat_pin,
        pins.oe_pin,
    );
    info!("Hub75 driver initialized");

    let mut frame_counter: u32 = 0;
    let mut last_time = embassy_time::Instant::now();

    let state = CLUSTERS.init(RwLock::new(State::Init));

    loop {
        let current_time = embassy_time::Instant::now();
        let elapsed = current_time.duration_since(last_time);
        let micros = elapsed.as_micros();
        let fps = if micros > 0 { 1_000_000 / micros } else { 0 };
        last_time = current_time;

        if frame_counter % 60 == 0 {
            info!("Animation FPS: {}", fps);
        }

        let anim_start = embassy_time::Instant::now();

        match &*state.read().await {
            State::Init => animations::fortytwo::draw_animation_frame(&mut display, frame_counter),
            State::Running(layout) => {
                cluster_core::visualization::draw_cluster_frame(&mut display, layout, frame_counter)
            }
            State::Error(_) => {
                animations::fortytwo::draw_animation_frame(&mut display, frame_counter)
            }
        }
        .unwrap();

        let anim_time = anim_start.elapsed();

        let commit_start = embassy_time::Instant::now();
        display.commit();
        let commit_time = commit_start.elapsed();

        if frame_counter % 60 == 0 {
            info!(
                "Animation draw time: {}us, Buffer commit time: {}us",
                anim_time.as_micros(),
                commit_time.as_micros()
            );
        }

        frame_counter = frame_counter.wrapping_add(1);
    }
}

#[embassy_executor::task]
async fn core1_task(mut led: gpio::Output<'static>) {
    info!("Hello from core 1 - Starting LED blink");

    loop {
        led.set_high();
        Timer::after(Duration::from_secs(1)).await;
        led.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}
