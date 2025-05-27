//! Updated example showing how to use the new PIO-based Hub75 driver

#![no_std]
#![no_main]

use common::animations;
use defmt::info;
use embassy_executor::{Executor, Spawner};
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::peripherals::*;
use embassy_rp::{Peri, gpio};
use embassy_time::{Delay, Duration, Timer};
use hub75_rp2350_driver::{Hub75, Hub75Config};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Multicore setup
static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Configure GPIO pins for Hub75 LED matrix
    // Control pins are configured as regular GPIO outputs
    let lat_pin = gpio::Output::new(p.PIN_7, gpio::Level::Low);
    let oe_pin = gpio::Output::new(p.PIN_8, gpio::Level::High); // OE is active low, start disabled

    // Address pins
    let a_pin = gpio::Output::new(p.PIN_9, gpio::Level::Low);
    let b_pin = gpio::Output::new(p.PIN_10, gpio::Level::Low);
    let c_pin = gpio::Output::new(p.PIN_11, gpio::Level::Low);
    let d_pin = gpio::Output::new(p.PIN_12, gpio::Level::Low);
    let e_pin = gpio::Output::new(p.PIN_13, gpio::Level::Low);

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

    // Core 0 handles Hub75 matrix with PIO + DMA
    spawner
        .spawn(matrix_task(
            p.PIO0, p.DMA_CH0, p.PIN_0, // r1_pin
            p.PIN_1, // g1_pin
            p.PIN_2, // b1_pin
            p.PIN_3, // r2_pin
            p.PIN_4, // g2_pin
            p.PIN_5, // b2_pin
            p.PIN_6, // clk_pin
            lat_pin, oe_pin, a_pin, b_pin, c_pin, d_pin, e_pin,
        ))
        .unwrap();
}

#[embassy_executor::task]
async fn matrix_task(
    pio: Peri<'static, PIO0>,
    dma_chan: Peri<'static, DMA_CH0>,
    r1_pin: Peri<'static, PIN_0>,
    g1_pin: Peri<'static, PIN_1>,
    b1_pin: Peri<'static, PIN_2>,
    r2_pin: Peri<'static, PIN_3>,
    g2_pin: Peri<'static, PIN_4>,
    b2_pin: Peri<'static, PIN_5>,
    clk_pin: Peri<'static, PIN_6>,
    lat_pin: gpio::Output<'static>,
    oe_pin: gpio::Output<'static>,
    a_pin: gpio::Output<'static>,
    b_pin: gpio::Output<'static>,
    c_pin: gpio::Output<'static>,
    d_pin: gpio::Output<'static>,
    e_pin: gpio::Output<'static>,
) {
    let mut delay = Delay;

    info!("Starting Hub75 LED matrix control with PIO + DMA");

    // Create the LED matrix driver with PIO + DMA
    let config = Hub75Config::default();
    let mut display = Hub75::new(
        pio, dma_chan, config, r1_pin, g1_pin, b1_pin, r2_pin, g2_pin, b2_pin, clk_pin, lat_pin,
        oe_pin, a_pin, b_pin, c_pin, d_pin, e_pin,
    );

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

        if frame_counter % 60 == 0 {
            info!("Current FPS: {}", fps);
        }

        // Measure animation frame drawing time
        let anim_start = embassy_time::Instant::now();
        // Draw the current animation frame
        animations::stars::draw_animation_frame(&mut display, frame_counter).unwrap();
        // animations::fortytwo::draw_animation_frame(&mut display, frame_counter).unwrap();
        // display.draw_test_gradient();
        // display.draw_channel_test();
        // display.draw_test_pattern();
        let anim_time = anim_start.elapsed();

        // Measure display update time
        let update_start = embassy_time::Instant::now();
        // Update the display
        unsafe {
            display.update(&mut delay).await.unwrap_unchecked();
        }
        let update_time = update_start.elapsed();

        if frame_counter % 60 == 0 {
            info!(
                "Animation time: {}us, Update time: {}us",
                anim_time.as_micros(),
                update_time.as_micros()
            );
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
