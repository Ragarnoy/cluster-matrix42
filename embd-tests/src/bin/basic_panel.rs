//! Multicore Hub75 LED Matrix Demo for 64x64 display with Embassy
//! Core 0: Controls Hub75 LED matrix
//! Core 1: Handles led blinking

#![no_std]
#![no_main]

use common::animations;
use defmt::info;
use embassy_executor::{Executor, Spawner};
use embassy_rp::gpio;
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_time::{Delay, Duration, Timer};
use hub75_rp2350_driver::pins;
use hub75_rp2350_driver::{Hub75, Hub75Config, pins::Hub75Pins};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Multicore setup
static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

/// Input value 0 to 255 to get a color value
/// The colors are a transition r - g - b - back to r.

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Configure GPIO pins for Hub75 LED matrix
    let r1 = gpio::Output::new(p.PIN_0, gpio::Level::Low);
    let g1 = gpio::Output::new(p.PIN_1, gpio::Level::Low);
    let b1 = gpio::Output::new(p.PIN_2, gpio::Level::Low);
    let r2 = gpio::Output::new(p.PIN_3, gpio::Level::Low);
    let g2 = gpio::Output::new(p.PIN_4, gpio::Level::Low);
    let b2 = gpio::Output::new(p.PIN_5, gpio::Level::Low);

    let a = gpio::Output::new(p.PIN_6, gpio::Level::Low);
    let b = gpio::Output::new(p.PIN_7, gpio::Level::Low);
    let c = gpio::Output::new(p.PIN_8, gpio::Level::Low);
    let d = gpio::Output::new(p.PIN_9, gpio::Level::Low);
    let e = gpio::Output::new(p.PIN_10, gpio::Level::Low);

    let clk = gpio::Output::new(p.PIN_11, gpio::Level::Low);
    let lat = gpio::Output::new(p.PIN_12, gpio::Level::Low);
    let oe = gpio::Output::new(p.PIN_13, gpio::Level::High); // OE is active low, start disabled

    // Create pin tuple for our Hub75 driver
    let pins = (r1, g1, b1, r2, g2, b2, a, b, c, d, e, clk, lat, oe);

    // Spawn Core 1 to handle led blinking
    let led = gpio::Output::new(p.PIN_25, gpio::Level::Low);
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.spawn(core1_task(led)).unwrap();
            });
        },
    );

    // Core 0 handles Hub75 matrix
    spawner.spawn(matrix_task(pins)).unwrap();
}

#[embassy_executor::task]
async fn matrix_task(
    pins: (
        pins::R1,
        pins::G1,
        pins::B1,
        pins::R2,
        pins::G2,
        pins::B2,
        pins::A,
        pins::B,
        pins::C,
        pins::D,
        pins::E,
        pins::CLK,
        pins::LAT,
        pins::OE,
    ),
) {
    // Use Embassy's built-in Delay implementation
    let mut delay = Delay;

    info!("Starting Hub75 LED matrix control");

    let (r1, g1, b1, r2, g2, b2, a, b, c, d, e, clk, lat, oe) = pins;

    // Create pins struct with static dispatch
    let pins = Hub75Pins::new(r1, g1, b1, r2, g2, b2, a, b, c, d, e, clk, lat, oe);

    // Create the LED matrix driver
    let config = Hub75Config::default();

    let mut display = Hub75::new_with_config(pins, config);

    // Animation frame counter and time tracking
    let mut frame_counter: u32 = 0;
    let mut last_time = embassy_time::Instant::now();
    let mut fps: u64;

    // Main animation loop
    loop {
        let current_time = embassy_time::Instant::now();
        let elapsed = current_time.duration_since(last_time);
        let micros = elapsed.as_micros();
        fps = if micros > 0 { 1_000_000 / micros } else { 0 };
        last_time = current_time;

        info!("Current FPS: {}", fps);

        // Draw the current animation frame
        // animations::stars::draw_animation_frame(&mut display, frame_counter).unwrap();
        animations::fortytwo::draw_animation_frame(&mut display, frame_counter).unwrap();
        // display.draw_test_gradient();
        // display.draw_channel_test();
        // display.draw_test_pattern();

        // Update the display
        if let Err(e) = display.update(&mut delay) {
            defmt::error!("Display update failed: {:?}", e);
        }

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
