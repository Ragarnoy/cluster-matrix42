//! Updated example showing how to use the new PIO-based Hub75 driver

#![no_std]
#![no_main]

use core::ptr::addr_of_mut;
use defmt::{info, unwrap};
use embassy_executor::{Executor, Spawner};
use embassy_rp::multicore::spawn_core1;
use embassy_rp::peripherals::*;
use embassy_rp::{Peri, gpio};
use embassy_time::{Duration, Timer};
use firmware::{CORE1_STACK, DISPLAY_MEMORY, DmaChannels, EXECUTOR1, Hub75Pins};
use graphics_common::animations;
use hub75_driver::{DisplayMemory, Hub75};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut clk_config = embassy_rp::clocks::ClockConfig::rosc();
    clk_config.core_voltage = embassy_rp::clocks::CoreVoltage::V1_20;
    let p = embassy_rp::init(embassy_rp::config::Config::new(clk_config));

    // Spawn Core 1 to handle led blinking
    let led = gpio::Output::new(p.PIN_25, gpio::Level::Low);
    spawn_core1(
        p.CORE1,
        unsafe { &mut *addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.spawn(unwrap!(core1_task(led)));
            });
        },
    );

    let pins = Hub75Pins::for_board(
        p.PIN_0, p.PIN_1, p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, p.PIN_6, p.PIN_7, p.PIN_8, p.PIN_9,
        p.PIN_10, p.PIN_11, p.PIN_12, p.PIN_13,
    );

    let dma_channels = DmaChannels {
        dma_ch0: p.DMA_CH0,
        dma_ch1: p.DMA_CH1,
        dma_ch2: p.DMA_CH2,
        dma_ch3: p.DMA_CH3,
    };

    // Core 0 handles Hub75 matrix with PIO + DMA
    spawner.spawn(unwrap!(matrix_task(p.PIO0, dma_channels, pins)));
}

#[embassy_executor::task]
#[allow(clippy::unused_async)] // embassy tasks must be async
async fn matrix_task(pio: Peri<'static, PIO0>, dma_channels: DmaChannels, pins: Hub75Pins) {
    info!("Starting Hub75 LED matrix control with 3 PIO SMs + chained DMA");

    // Create the LED matrix driver with PIO + DMA
    let mut display = Hub75::new(
        pio,
        (
            dma_channels.dma_ch0,
            dma_channels.dma_ch1,
            dma_channels.dma_ch2,
            dma_channels.dma_ch3,
        ),
        DISPLAY_MEMORY.init(DisplayMemory::new()),
        // RGB data pins
        pins.r1_pin,
        pins.g1_pin,
        pins.b1_pin,
        pins.r2_pin,
        pins.g2_pin,
        pins.b2_pin,
        pins.clk_pin,
        // Address pins (all 5 for 64x64 display)
        pins.a_pin,
        pins.b_pin,
        pins.c_pin,
        pins.d_pin,
        pins.e_pin,
        // Control pins
        pins.lat_pin,
        pins.oe_pin,
    );
    info!("Hub75 driver initialized - display running continuously with zero CPU overhead");

    // Animation frame counter and time tracking
    let mut frame_counter: u32 = 0;
    let mut last_time = embassy_time::Instant::now();

    // Main animation loop - no need to call update(), display runs automatically!
    loop {
        let current_time = embassy_time::Instant::now();
        let elapsed = current_time.duration_since(last_time);
        let micros = elapsed.as_micros();
        let fps = if micros > 0 { 1_000_000 / micros } else { 0 };
        last_time = current_time;

        if frame_counter.is_multiple_of(60) {
            info!("Animation FPS: {}", fps);
        }

        // Measure animation frame drawing time
        let anim_start = embassy_time::Instant::now();

        // animations::quadrant::draw_animation_frame(&mut display, frame_counter).unwrap();
        // animations::stars::draw_animation_frame(&mut display, frame_counter).unwrap();

        // animations::arrow::draw_animation_frame(&mut display, frame_counter).unwrap();
        animations::fortytwo::draw_animation_frame(&mut display, frame_counter).unwrap();
        // display.draw_test_pattern();

        let anim_time = anim_start.elapsed();

        // Commit the buffer - this makes it visible on the display
        // Blocks until the DMA read pointer enters the new buffer's range (up to one full scan)
        let commit_start = embassy_time::Instant::now();
        display.commit();
        let commit_time = commit_start.elapsed();

        if frame_counter.is_multiple_of(60) {
            info!(
                "Animation draw time: {}us, Buffer commit time: {}us",
                anim_time.as_micros(),
                commit_time.as_micros()
            );
        }

        // Control animation frame rate (optional - you can go as fast as you want)
        // Timer::after(Duration::from_millis(16)).await; // ~60 FPS animation

        // Increment frame counter
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
