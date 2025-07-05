//! Updated example showing how to use the new PIO-based Hub75 driver

#![no_std]
#![no_main]

use cluster_matrix::animations;
use core::ptr::addr_of_mut;
use defmt::info;
use embassy_executor::{Executor, Spawner};
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::peripherals::*;
use embassy_rp::{Peri, gpio};
use embassy_time::{Duration, Timer};
use hub75_rp2350_driver::{DisplayMemory, Hub75};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Multicore setup
static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

// Static memory for the display - required for the driver
static DISPLAY_MEMORY: StaticCell<DisplayMemory> = StaticCell::new();

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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Spawn Core 1 to handle led blinking
    let led = gpio::Output::new(p.PIN_25, gpio::Level::Low);
    spawn_core1(
        p.CORE1,
        unsafe { &mut *addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.spawn(core1_task(led)).unwrap();
            });
        },
    );

    // Group pins and DMA channels
    let pins = Hub75Pins {
        r1_pin: p.PIN_0,
        g1_pin: p.PIN_1,
        b1_pin: p.PIN_2,
        r2_pin: p.PIN_3,
        g2_pin: p.PIN_4,
        b2_pin: p.PIN_5,

        a_pin: p.PIN_6,  // Changed from PIN_9
        b_pin: p.PIN_7,  // Changed from PIN_10
        c_pin: p.PIN_8,  // Changed from PIN_11
        d_pin: p.PIN_9,  // Changed from PIN_12
        e_pin: p.PIN_10, // Changed from PIN_13

        clk_pin: p.PIN_11, // Changed from PIN_6
        lat_pin: p.PIN_12, // Changed from PIN_7
        oe_pin: p.PIN_13,  // Changed from PIN_8
    };

    let dma_channels = DmaChannels {
        dma_ch0: p.DMA_CH0,
        dma_ch1: p.DMA_CH1,
        dma_ch2: p.DMA_CH2,
        dma_ch3: p.DMA_CH3,
    };

    // Core 0 handles Hub75 matrix with PIO + DMA
    spawner
        .spawn(matrix_task(p.PIO0, dma_channels, pins))
        .unwrap();
}

#[embassy_executor::task]
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

        if frame_counter % 60 == 0 {
            info!("Animation FPS: {}", fps);
        }

        // Measure animation frame drawing time
        let anim_start = embassy_time::Instant::now();

        //animations::quadrant::draw_animation_frame(&mut display, frame_counter).unwrap();
        // animations::stars::draw_animation_frame(&mut display, frame_counter).unwrap();

        // animations::arrow::draw_animation_frame(&mut display, frame_counter).unwrap();
        animations::fortytwo::draw_animation_frame(&mut display, frame_counter).unwrap();
        // display.draw_test_pattern();

        let anim_time = anim_start.elapsed();

        // Commit the buffer - this makes it visible on the display
        // This is very fast (just a pointer swap) and non-blocking
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
