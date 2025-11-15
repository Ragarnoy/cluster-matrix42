//! Test the C plugin loading system on real hardware
//! This binary loads the embedded plasma plugin and runs it on the LED matrix

#![no_std]
#![no_main]

use basic_panel::{CORE1_STACK, DISPLAY_MEMORY, DmaChannels, EXECUTOR1, Hub75Pins};
use core::ptr::addr_of_mut;
use defmt::{info, unwrap, warn};
use embassy_executor::{Executor, Spawner};
use embassy_rp::multicore::spawn_core1;
use embassy_rp::peripherals::*;
use embassy_rp::{Peri, gpio};
use embassy_time::{Duration, Timer};
use hub75_rp2350_driver::{
    COLOR_BITS, DISPLAY_HEIGHT, DISPLAY_WIDTH, DisplayMemory, Hub75, lut::GAMMA8,
};
use plugin_host::PluginRuntime;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Plugin Test Starting!");

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

    // Group pins and DMA channels
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

        clk_pin: p.PIN_11,
        lat_pin: p.PIN_12,
        oe_pin: p.PIN_13,
    };

    let dma_channels = DmaChannels {
        dma_ch0: p.DMA_CH0,
        dma_ch1: p.DMA_CH1,
        dma_ch2: p.DMA_CH2,
        dma_ch3: p.DMA_CH3,
    };

    // Core 0 handles Hub75 matrix with plugins
    spawner.spawn(unwrap!(matrix_task(p.PIO0, dma_channels, pins)));
}

#[embassy_executor::task]
async fn matrix_task(pio: Peri<'static, PIO0>, dma_channels: DmaChannels, pins: Hub75Pins) {
    info!("Starting Hub75 LED matrix with plugin system");

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
        // Address pins
        pins.a_pin,
        pins.b_pin,
        pins.c_pin,
        pins.d_pin,
        pins.e_pin,
        // Control pins
        pins.lat_pin,
        pins.oe_pin,
    );
    info!("Hub75 driver initialized");

    // Initialize the plugin runtime
    let runtime = PluginRuntime::init();
    info!("Plugin runtime initialized");

    // List available plugins
    let plugin_list = plugin_host::get_plugin_list();
    info!("Available plugins: {}", plugin_list.len());

    for (name, bytes) in plugin_list {
        info!("  - {} ({} bytes)", name, bytes.len());
    }

    // Find and load the quadrant plugin
    if plugin_list.is_empty() {
        warn!("No plugins available!");
        loop {
            Timer::after(Duration::from_secs(1)).await;
        }
    }

    // Look for the quadrant plugin
    let plugin_to_load = plugin_list
        .iter()
        .find(|(name, _)| *name == "plasma")
        .or_else(|| plugin_list.first()) // Fallback to first plugin if quadrant not found
        .unwrap();

    let (plugin_name, plugin_bytes) = plugin_to_load;
    info!("Loading plugin: {}", plugin_name);

    match runtime.load_plugin(plugin_bytes) {
        Ok(()) => {
            info!("Plugin loaded successfully!");
        }
        Err(e) => {
            warn!("Failed to load plugin: {:?}", e);
            loop {
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    }

    // Animation frame counter and time tracking
    let mut frame_counter: u32 = 0;
    let mut last_time = embassy_time::Instant::now();

    info!("Starting plugin animation loop");
    info!(
        "Display dimensions: {}x{}",
        hub75_rp2350_driver::DISPLAY_WIDTH,
        hub75_rp2350_driver::DISPLAY_HEIGHT
    );
    info!("Plugin framebuffer: 128x128");

    // Main animation loop - run the plugin!
    loop {
        let current_time = embassy_time::Instant::now();
        let elapsed = current_time.duration_since(last_time);
        let micros = elapsed.as_micros();
        let fps = if micros > 0 { 1_000_000 / micros } else { 0 };
        last_time = current_time;

        if frame_counter % 60 == 0 {
            info!("Plugin FPS: {}", fps);
        }

        // Run the plugin's update function
        let update_start = embassy_time::Instant::now();
        runtime.update(0); // No input for now
        let update_time = update_start.elapsed();

        // Copy the plugin's framebuffer to the display
        // The plugin renders to a 128x128 buffer, we need to copy it to the display
        let copy_start = embassy_time::Instant::now();
        copy_framebuffer_to_display(runtime.framebuffer(), &mut display);
        let copy_time = copy_start.elapsed();

        // Commit the buffer to make it visible
        let commit_start = embassy_time::Instant::now();
        display.commit();
        let commit_time = commit_start.elapsed();

        if frame_counter % 60 == 0 {
            info!(
                "Timing - Update: {}us, Copy: {}us, Commit: {}us",
                update_time.as_micros(),
                copy_time.as_micros(),
                commit_time.as_micros()
            );
        }

        // Increment frame counter
        frame_counter = frame_counter.wrapping_add(1);

        // Small delay to avoid busy-waiting
        Timer::after(Duration::from_millis(16)).await; // ~60 FPS
    }
}

/// Copy the plugin's framebuffer to the display using optimized direct buffer writes
/// Plugin renders to 128x128, driver transforms coords to 256x64 physical layout
fn copy_framebuffer_to_display(plugin_fb: &plugin_api::FrameBuffer, display: &mut Hub75) {
    // Get direct access to the display buffer for optimal performance
    let buffer = display.get_buffer_mut();

    // Clear the buffer first
    buffer.fill(0);

    // Plugin uses logical 128x128 coordinates
    // Driver's coord_transfer maps them to physical 256x64:
    // - Top half (y=0-63) -> right side (x=128-255, y=0-63)
    // - Bottom half (y=64-127) -> left side (x=0-127, y=0-63)

    for plugin_y in 0..128 {
        for plugin_x in 0..128 {
            let plugin_idx = plugin_y * 128 + plugin_x;
            let color_u16 = plugin_fb.pixels[plugin_idx];

            // RGB565 format: RRRR RGGG GGGB BBBB
            let r = ((color_u16 >> 11) & 0x1F) as u8;
            let g = ((color_u16 >> 5) & 0x3F) as u8;
            let b = (color_u16 & 0x1F) as u8;

            // Apply coord_transfer to get physical display coordinates
            let (disp_x, disp_y) = if plugin_y < 64 {
                (plugin_x + 128, plugin_y) // Top half -> right side
            } else {
                (plugin_x, plugin_y - 64) // Bottom half -> left side
            };

            // Skip if out of bounds
            if disp_x >= DISPLAY_WIDTH || disp_y >= DISPLAY_HEIGHT {
                continue;
            }

            // Expand RGB565 to 8-bit per channel and apply GBR swap + gamma in one step
            // GBR swap: R->G, G->B, B->R
            let c_g = GAMMA8[(r << 3) as usize] as u16; // Red channel → Green (physical)
            let c_b = GAMMA8[(g << 2) as usize] as u16; // Green channel → Blue (physical)
            let c_r = GAMMA8[(b << 3) as usize] as u16; // Blue channel → Red (physical)

            // Determine if this is top or bottom half of display
            let shift = if disp_y >= (DISPLAY_HEIGHT / 2) { 3 } else { 0 };

            // Calculate base index in buffer
            // Buffer layout: [row][bit_plane][column]
            let base_idx = disp_x + ((disp_y % (DISPLAY_HEIGHT / 2)) * DISPLAY_WIDTH * COLOR_BITS);

            // Encode in BCM format - write each bit plane (unrolled for performance)
            // Bit plane 0 (LSB)
            buffer[base_idx] |=
                (((c_b & 0b1) << 2 | (c_g & 0b1) << 1 | (c_r & 0b1)) as u8) << shift;
            // Bit plane 1
            buffer[base_idx + DISPLAY_WIDTH] |=
                ((((c_b >> 1) & 0b1) << 2 | ((c_g >> 1) & 0b1) << 1 | ((c_r >> 1) & 0b1)) as u8)
                    << shift;
            // Bit plane 2
            buffer[base_idx + DISPLAY_WIDTH * 2] |=
                ((((c_b >> 2) & 0b1) << 2 | ((c_g >> 2) & 0b1) << 1 | ((c_r >> 2) & 0b1)) as u8)
                    << shift;
            // Bit plane 3
            buffer[base_idx + DISPLAY_WIDTH * 3] |=
                ((((c_b >> 3) & 0b1) << 2 | ((c_g >> 3) & 0b1) << 1 | ((c_r >> 3) & 0b1)) as u8)
                    << shift;
            // Bit plane 4
            buffer[base_idx + DISPLAY_WIDTH * 4] |=
                ((((c_b >> 4) & 0b1) << 2 | ((c_g >> 4) & 0b1) << 1 | ((c_r >> 4) & 0b1)) as u8)
                    << shift;
            // Bit plane 5
            buffer[base_idx + DISPLAY_WIDTH * 5] |=
                ((((c_b >> 5) & 0b1) << 2 | ((c_g >> 5) & 0b1) << 1 | ((c_r >> 5) & 0b1)) as u8)
                    << shift;
            // Bit plane 6
            buffer[base_idx + DISPLAY_WIDTH * 6] |=
                ((((c_b >> 6) & 0b1) << 2 | ((c_g >> 6) & 0b1) << 1 | ((c_r >> 6) & 0b1)) as u8)
                    << shift;
            // Bit plane 7 (MSB)
            buffer[base_idx + DISPLAY_WIDTH * 7] |=
                ((((c_b >> 7) & 0b1) << 2 | ((c_g >> 7) & 0b1) << 1 | ((c_r >> 7) & 0b1)) as u8)
                    << shift;
        }
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
