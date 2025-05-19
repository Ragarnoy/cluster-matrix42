//! Multicore Hub75 LED Matrix Demo for 64x64 display with Embassy
//! Core 0: Controls Hub75 LED matrix and WS2812 LED
//! Core 1: Handles USB logging
//!
//! Features:
//! - 64x64 LED matrix showing alternating background colors with a grey square
//! - Brightness cycling between very dim, dim, normal, and bright
//! - WS2812 LED blinking with rainbow colors
//! - USB logging from Core 1

#![no_std]
#![no_main]

use embassy_executor::Executor;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::{PIO0, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use log::{debug, info};
use smart_leds::RGB8;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Import our Hub75 driver
use hub_75_driver::{Hub75, Hub75Config, Hub75Pins};

// Number of WS2812 LEDs in the strip
const NUM_LEDS: usize = 1;

// Interrupt bindings
bind_interrupts!(struct UsbIrqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

// Multicore setup
static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());

    // Setup USB driver for Core 1
    let usb_driver = Driver::new(p.USB, UsbIrqs);

    // Setup WS2812 for Core 0 - using PIN_25 for WS2812
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, PioIrqs);
    let program = PioWs2812Program::new(&mut common);
    let ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_25, &program);

    // Configure GPIO pins for Hub75 LED matrix
    let r1 = gpio::Output::new(p.PIN_28, gpio::Level::Low);
    let g1 = gpio::Output::new(p.PIN_5, gpio::Level::Low);
    let b1 = gpio::Output::new(p.PIN_27, gpio::Level::Low);
    let r2 = gpio::Output::new(p.PIN_26, gpio::Level::Low);
    let g2 = gpio::Output::new(p.PIN_4, gpio::Level::Low);
    let b2 = gpio::Output::new(p.PIN_22, gpio::Level::Low);

    let a = gpio::Output::new(p.PIN_9, gpio::Level::Low);
    let b = gpio::Output::new(p.PIN_2, gpio::Level::Low);
    let c = gpio::Output::new(p.PIN_8, gpio::Level::Low);
    let d = gpio::Output::new(p.PIN_1, gpio::Level::Low);
    let e = gpio::Output::new(p.PIN_3, gpio::Level::Low);

    let lat = gpio::Output::new(p.PIN_0, gpio::Level::Low);
    let clk = gpio::Output::new(p.PIN_7, gpio::Level::Low);
    let oe = gpio::Output::new(p.PIN_6, gpio::Level::High); // OE is active low, start disabled

    // Create pin tuple for our Hub75 driver
    let pins = (r1, g1, b1, r2, g2, b2, a, b, c, d, e, clk, lat, oe);

    // Spawn Core 1 to handle USB logging
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.spawn(core1_task(usb_driver)).unwrap();
            });
        },
    );

    // Core 0 handles Hub75 matrix and WS2812 LED
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        spawner.spawn(matrix_task(pins)).unwrap();
        spawner.spawn(ws2812_task(ws2812)).unwrap();
    });
}

// Define a specific type for our WS2812 LED controller to avoid generic task functions
type WS2812Type = PioWs2812<'static, PIO0, 0, NUM_LEDS>;

#[embassy_executor::task]
async fn ws2812_task(mut ws2812: WS2812Type) {
    info!("Starting WS2812 LED control");
    let mut data = [RGB8::default(); NUM_LEDS];

    // Create a ticker for WS2812 updates
    let mut color_wheel_pos: u16 = 0;

    // Main LED control loop
    loop {
        // Update all LEDs with current color
        for i in 0..NUM_LEDS {
            let wheel_pos = (((i * 256) as u16 / NUM_LEDS as u16 + color_wheel_pos) & 255) as u8;
            data[i] = wheel(wheel_pos);
            debug!("LED {}: R: {} G: {} B: {}", i, data[i].r, data[i].g, data[i].b);
        }

        // Write the colors to the WS2812 LEDs
        ws2812.write(&data).await;

        // Log current state
        info!("WS2812 update - Color wheel position: {}", color_wheel_pos);

        // Increment the color wheel position
        color_wheel_pos = (color_wheel_pos + 10) % 256;

        // Wait before the next update
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::task]
async fn matrix_task(pins: (
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
    gpio::Output<'static>,
)) {
    // Use Embassy's built-in Delay implementation
    let mut delay = Delay;

    info!("Starting Hub75 LED matrix control");
    
    let (r1, g1, b1, r2, g2, b2, a, b, c, d, e, clk, lat, oe) = pins;

    // Create pins struct with static dispatch
    let pins = Hub75Pins::new(r1, g1, b1, r2, g2, b2, a, b, c, d, e, clk, lat, oe);

    // Create the LED matrix driver
    let config = Hub75Config {
        pwm_bits: 4,
        brightness: 100,
        use_gamma_correction: false,
        chain_length: 1,
        row_step_time_us: 1,
    };

    let mut display = Hub75::new_with_config(pins, config);

    // Define colors to alternate for the background
    let colors = [
        Rgb565::RED,
        Rgb565::GREEN,
        Rgb565::BLUE,
        Rgb565::YELLOW,
        Rgb565::CYAN,
        Rgb565::MAGENTA,
    ];
    let mut color_index = 0;
    let mut frame_counter = 0;

    // The central grey square
    let grey = Rgb565::new(150, 150, 150); // Medium grey
    
    // Main animation loop
    loop {
        let start_time = embassy_time::Instant::now();

        while (embassy_time::Instant::now() - start_time) < Duration::from_secs(2) {
            // Clear the display
            display.clear();

            // Get current background color
            let bg_color = colors[color_index];

            // Fill screen with background color
            // (except where the grey square will be)
            for y in 0..64 {
                for x in 0..64 {
                    // Skip pixels that will be part of the grey square
                    if (20..44).contains(&x) && (20..44).contains(&y) {
                        continue;
                    }
                    display.draw_iter([Pixel(Point::new(x, y), bg_color)]).unwrap();
                }
            }

            // Draw the grey square in the middle (24x24 pixels)
            let square_style = PrimitiveStyleBuilder::new()
                .fill_color(grey)
                .build();

            Rectangle::new(Point::new(20, 20), Size::new(24, 24))
                .into_styled(square_style)
                .draw(&mut display)
                .unwrap();

            // Log status before update
            info!("Matrix rendering - Frame: {}, Color: {}",
                  frame_counter, color_index);

            // Try-catch the update to prevent crashing if there's an issue
            display.update(&mut delay).unwrap();

            // Increment frame counter
            frame_counter += 1;
        }

        // Cycle to next color
        color_index = (color_index + 1) % colors.len();

        // Wait 500ms before starting next cycle
        Timer::after(Duration::from_millis(500)).await;
    }
}


#[embassy_executor::task]
async fn core1_task(driver: Driver<'static, USB>) {
    info!("Hello from core 1 - Starting USB logger");

    // Start the USB logger - this function doesn't return
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
