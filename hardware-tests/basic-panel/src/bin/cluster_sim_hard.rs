//! Hardware test for cluster visualization system
//! Combines basic-panel hardware setup with cluster simulation data

#![no_std]
#![no_main]

use basic_panel::{
    CORE1_STACK, DISPLAY_MEMORY, DmaChannels, EXECUTOR1, Hub75Pins, LAYOUT, LayoutLock,
    SELECTED_CLUSTER, helpers,
};
use cluster_core::types::ClusterId;
use cluster_core::visualization::ClusterRenderer;
use core::ptr::addr_of_mut;
use defmt::{Debug2Format, info, warn};
use embassy_executor::{Executor, Spawner};
use embassy_rp::gpio::Output;
use embassy_rp::multicore::spawn_core1;
use embassy_rp::peripherals::*;
use embassy_rp::{Peri, gpio};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::rwlock::RwLock;
use embassy_time::{Duration, Timer};
use hub75_rp2350_driver::{DisplayMemory, Hub75};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let layout = helpers::create_sample_layout().unwrap_or_else(|_| {
        panic!("Failed to create sample cluster layout");
    });
    info!(
        "Sample cluster layout created successfully, size of layout: {}",
        size_of_val(&layout)
    );

    let layout = &*LAYOUT.init(RwLock::new(layout));
    let selected_cluster = &*SELECTED_CLUSTER.init(Channel::new());
    let rx = selected_cluster.receiver();
    let tx = selected_cluster.sender();

    info!("Sample cluster layout created successfully");

    // Spawn Core 1 to handle LED blinking
    let led = Output::new(p.PIN_25, gpio::Level::Low);
    spawn_core1(
        p.CORE1,
        unsafe { &mut *addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.spawn(core1_task(led, layout, tx)).unwrap();
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

    // Core 0 handles Hub75 matrix with cluster visualization
    spawner
        .spawn(cluster_matrix_task(p.PIO0, dma_channels, pins, layout, rx))
        .unwrap();
}

#[embassy_executor::task]
async fn cluster_matrix_task(
    pio: Peri<'static, PIO0>,
    dma_channels: DmaChannels,
    pins: Hub75Pins,
    layout: &'static LayoutLock,
    receiver: Receiver<'static, CriticalSectionRawMutex, ClusterId, 8>,
) {
    info!("Starting Hub75 LED matrix with cluster visualization");

    // Create the LED matrix driver
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

    // Animation loop
    let mut frame_counter: u32 = 0;
    let mut last_time = embassy_time::Instant::now();

    let mut renderer = ClusterRenderer::new();

    loop {
        let current_time = embassy_time::Instant::now();
        let elapsed = current_time.duration_since(last_time);
        let micros = elapsed.as_micros();
        let fps = if micros > 0 { 1_000_000 / micros } else { 0 };
        last_time = current_time;

        if frame_counter % 60 == 0 {
            info!("Cluster visualization FPS: {}", fps);
            if let Ok(cluster_id) = receiver.try_receive() {
                info!("Selected cluster: {:?}", Debug2Format(&cluster_id));
                renderer.set_selected_cluster(cluster_id)
            }
        }

        // Draw cluster frame
        let anim_start = embassy_time::Instant::now();

        if let Ok(layout) = layout.try_read() {
            match renderer.render_frame(&mut display, &layout, frame_counter) {
                Ok(_) => {}
                Err(_) => {
                    info!("Failed to draw cluster frame");
                    display.draw_test_pattern();
                }
            }

            let anim_time = anim_start.elapsed();

            // Commit the buffer
            let commit_start = embassy_time::Instant::now();
            display.commit();
            let commit_time = commit_start.elapsed();

            if frame_counter % 60 == 0 {
                info!(
                    "Cluster draw time: {}us, Buffer commit time: {}us",
                    anim_time.as_micros(),
                    commit_time.as_micros()
                );
            }
        } else {
            warn!("Failed to read layout");
        }

        // Control frame rate - cluster visualization is more static than animations
        Timer::after(Duration::from_millis(20)).await;

        frame_counter = frame_counter.wrapping_add(1);
    }
}

#[embassy_executor::task]
async fn core1_task(
    mut led: Output<'static>,
    layout: &'static LayoutLock,
    sender: Sender<'static, CriticalSectionRawMutex, ClusterId, 8>,
) {
    info!("Core 1 - LED heartbeat for cluster hardware test");

    let mut counter = 0usize;
    loop {
        counter = counter.wrapping_add(1);

        let cluster_id = match counter % 7 {
            0 | 1 => ClusterId::F0,
            2 => ClusterId::F1,
            3 => ClusterId::F1b,
            4 => ClusterId::F2,
            5 => ClusterId::F4,
            _ => ClusterId::F6,
        };

        sender.send(cluster_id).await;

        for _ in 0..5 {
            led.set_high();
            Timer::after(Duration::from_millis(500)).await;
            led.set_low();
            Timer::after(Duration::from_millis(500)).await;
        }

        if counter % 10 == 1 {
            let mut lock = layout.write().await;
            let seat_number = counter % lock.f0.seats.len();
            if let Some(status) = lock.f0.seats.get_mut(seat_number) {
                info!("Core 1 - Changing status of seat {}", seat_number);
                status.status = !status.status;
            } else {
                warn!("Seat {} not found in f0 cluster", seat_number);
            }
        }
    }
}
