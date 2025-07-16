//! Hardware test for cluster visualization system
//! Combines basic-panel hardware setup with cluster simulation data

#![no_std]
#![no_main]

use basic_panel::{CORE1_STACK, DISPLAY_MEMORY, DmaChannels, EXECUTOR1, Hub75Pins};
use cluster_core::models::{Cluster, Layout, Seat, SeatVec, Zone, ZoneVec};
use cluster_core::types::AttributeVec;
use cluster_core::types::{Attribute, ClusterString, Kind, MessageString, SeatId, Status};
use cluster_core::visualization::ClusterRenderer;
use cluster_core::{empty_cluster, seats};
use core::ptr::addr_of_mut;
use defmt::info;
use embassy_executor::{Executor, Spawner};
use embassy_rp::multicore::spawn_core1;
use embassy_rp::peripherals::*;
use embassy_rp::{Peri, gpio};
use embassy_time::{Duration, Timer};
use hub75_rp2350_driver::{DisplayMemory, Hub75};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Spawn Core 1 to handle LED blinking
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
        .spawn(cluster_matrix_task(p.PIO0, dma_channels, pins))
        .unwrap();
}

#[embassy_executor::task]
async fn cluster_matrix_task(pio: Peri<'static, PIO0>, dma_channels: DmaChannels, pins: Hub75Pins) {
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

    // Create sample cluster layout (no_std compatible)
    let layout = match create_sample_layout() {
        Ok(layout) => layout,
        Err(_) => {
            info!("Failed to create sample layout, falling back to test pattern");
            loop {
                display.draw_test_pattern();
                display.commit();
                Timer::after(Duration::from_millis(1000)).await;
            }
        }
    };

    info!("Sample cluster layout created successfully");

    // Animation loop
    let mut frame_counter: u32 = 0;
    let mut last_time = embassy_time::Instant::now();

    let renderer = ClusterRenderer::new();

    loop {
        let current_time = embassy_time::Instant::now();
        let elapsed = current_time.duration_since(last_time);
        let micros = elapsed.as_micros();
        let fps = if micros > 0 { 1_000_000 / micros } else { 0 };
        last_time = current_time;

        if frame_counter % 60 == 0 {
            info!("Cluster visualization FPS: {}", fps);
        }

        // Draw cluster frame
        let anim_start = embassy_time::Instant::now();

        match renderer.render_frame(&mut display, &layout, &layout.f0, frame_counter) {
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

        // Control frame rate - cluster visualization is more static than animations
        // Timer::after(Duration::from_millis(50)).await; // ~20 FPS for cluster updates

        frame_counter = frame_counter.wrapping_add(1);
    }
}

/// Create sample cluster layout using no_std compatible types
fn create_sample_layout() -> Result<Layout, &'static str> {
    // Helper function to create SeatId
    fn make_seat_id(id: &str) -> Result<SeatId, &'static str> {
        SeatId::try_from(id).map_err(|_| "seat id")
    }

    // Helper function to create ClusterString
    fn make_cluster_string(s: &str) -> Result<ClusterString, &'static str> {
        ClusterString::try_from(s).map_err(|_| "str too long")
    }

    // Helper function to create MessageString
    fn make_message_string(s: &str) -> Result<MessageString, &'static str> {
        MessageString::try_from(s).map_err(|_| "msg too long")
    }

    // Create seats using the proper SeatVec type
    let mut all_seats = SeatVec::new();

    // Row 1 (6 seats)
    all_seats.extend(seats![
        pattern: "f0r1s{}", 1..=6;
        kind: Kind::Mac;
        status: [Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken, Status::Taken];
        positions: (0, 0), (3, 1), (6, 0), (9, 1), (12, 0), (15, 1)
    ]);

    // Row 2 (23 seats)
    all_seats.extend(seats![
        pattern: "f0r2s{}", 1..=23;
        kind: Kind::Mac;
        status: [Status::Taken, Status::Taken, Status::Broken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Free, Status::Free, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Free, Status::Taken, Status::Taken, Status::Taken,
                 Status::Free, Status::Taken, Status::Taken];
        positions: (0, 5), (3, 6), (6, 5), (9, 6), (12, 5), (15, 6), (18, 5),
                   (26, 5), (29, 6), (32, 5), (35, 6), (38, 5), (41, 6), (44, 5),
                   (47, 6), (50, 5), (61, 5), (64, 6), (67, 5), (70, 6), (73, 5), (76, 6), (79, 5)
    ]);

    // Row 3 (23 seats)
    all_seats.extend(seats![
        pattern: "f0r3s{}", 1..=23;
        kind: Kind::Mac;
        status: [Status::Free, Status::Taken, Status::Broken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Free, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Free, Status::Taken, Status::Taken];
        positions: (0, 10), (3, 11), (6, 10), (9, 11), (12, 10), (15, 11), (18, 10),
                   (26, 10), (29, 11), (32, 10), (35, 11), (38, 10), (41, 11), (44, 10),(47, 11), (50, 10),
                    (61, 10), (64, 11), (67, 10), (70, 11), (73, 10), (76, 11), (79, 10)
    ]);

    // Row 4 (23 seats)
    all_seats.extend(seats![
        pattern: "f0r4s{}", 1..=23;
        kind: Kind::Mac;
        status: [Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Broken, Status::Taken, Status::Taken, Status::Taken, Status::Free,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Free,
                 Status::Taken, Status::Taken, Status::Taken, Status::Free, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken];
        positions: (0, 15), (3, 16), (6, 15), (9, 16), (12, 15), (15, 16), (18, 15),
                   (26, 15), (29, 16), (32, 15), (35, 16), (38, 15), (41, 16), (44, 15), (47, 16), (50, 15),
                    (61, 15), (64, 16), (67, 15), (70, 16), (73, 15), (76, 16), (79, 15)
    ]);

    // Row 5 (21 seats)
    all_seats.extend(seats![
        pattern: "f0r5s{}", 1..=21;
        kind: Kind::Mac;
        status: [Status::Free, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Broken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Free, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken];
        positions: (0, 20), (3, 21), (6, 20), (9, 21), (12, 20), (15, 21), (18, 20),
                    (29, 21), (32, 20), (35, 21), (38, 20), (41, 21), (44, 20), (47, 21),
                    (61, 20), (64, 21), (67, 20), (70, 21), (73, 20), (76, 21), (79, 20)
    ]);

    // Row 6 (20 seats)
    all_seats.extend(seats![
        pattern: "f0r6s{}", 1..=20;
        kind: Kind::Mac;
        status: [Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Free, Status::Taken,
                 Status::Taken, Status::Taken, Status::Broken, Status::Taken, Status::Taken,
                 Status::Free, Status::Taken, Status::Taken, Status::Taken, Status::Taken];
        positions: (0, 25), (3, 26), (6, 25), (9, 26), (12, 25), (15, 26),
                    (29, 26), (32, 25), (35, 26), (38, 25), (41, 26), (44, 25), (47, 26),
                    (61, 25), (64, 26), (67, 25), (70, 26), (73, 25), (76, 26), (79, 25)
    ]);

    // Row 7 (22 seats)
    all_seats.extend(seats![
        pattern: "f0r7s{}", 1..=22;
        kind: Kind::Mac;
        status: [Status::Free, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Free, Status::Taken, Status::Taken, Status::Broken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken];
        positions: (0, 30), (3, 31), (6, 30), (9, 31), (12, 30), (15, 31),
                    (26, 30), (29, 31), (32, 30), (35, 31), (38, 30), (41, 31), (44, 30), (47, 30), (50, 31),
                    (61, 30), (64, 31), (67, 30), (70, 31), (73, 30), (76, 31), (79, 30)
    ]);

    // Row 8 (20 seats)
    all_seats.extend(seats![
        pattern: "f0r8s{}", 1..=20;
        kind: Kind::Mac;
        status: [Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Free, Status::Taken, Status::Taken, Status::Taken];
        positions: (0, 35), (3, 36), (6, 35), (9, 36), (12, 35), (15, 36), (18, 35),
                    (29, 36), (32, 35), (35, 36), (38, 35), (41, 36), (44, 35), (47, 36),
                    (64, 36), (67, 35), (70, 36), (73, 35), (76, 36), (79, 35)
    ]);

    // Row 9 (22 seats)
    all_seats.extend(seats![
        pattern: "f0r9s{}", 1..=22;
        kind: Kind::Mac;
        status: [Status::Free, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Free,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Free, Status::Taken,
                 Status::Taken, Status::Taken];
        positions: (0, 40), (3, 41), (6, 40), (9, 41), (12, 40), (15, 41), (18, 40),
                    (26, 40), (29, 41), (32, 40), (35, 41), (38, 40), (41, 41), (44, 40), (47, 41), (50, 40),
                    (64, 41), (67, 40), (70, 41), (73, 40), (76, 41), (79, 40)
    ]);

    // Row 10 (23 seats)
    all_seats.extend(seats![
        pattern: "f0r10s{}", 1..=23;
        kind: Kind::Mac;
        status: [Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Free, Status::Taken, Status::Taken, Status::Broken, Status::Taken,
                 Status::Taken, Status::Free, Status::Taken, Status::Taken, Status::Taken,
                 Status::Free, Status::Taken, Status::Taken];
        positions: (0, 45), (3, 46), (6, 45), (9, 46), (12, 45), (15, 46), (18, 45),
                    (26, 45), (29, 46), (32, 45), (35, 46), (38, 45), (41, 46), (44, 45), (47, 46), (50, 45),
                    (61, 45), (64, 46), (67, 45), (70, 46), (73, 45), (76, 46), (79, 45)
    ]);

    // Row 11 (21 seats)
    all_seats.extend(seats![
        pattern: "f0r11s{}", 1..=21;
        kind: Kind::Mac;
        status: [Status::Free, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Broken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken];
        positions: (0, 50), (3, 51), (6, 50), (9, 51), (12, 50), (15, 51), (18, 50),
                    (29, 51), (32, 50), (35, 51), (38, 50), (41, 51), (44, 50), (47, 51),
                    (61, 50), (64, 51), (67, 50), (70, 51), (73, 50), (76, 51), (79, 50)
    ]);

    // Row 12 (23 seats)
    all_seats.extend(seats![
        pattern: "f0r12s{}", 1..=23;
        kind: Kind::Mac;
        status: [Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Free, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Free];
        positions: (0, 55), (3, 56), (6, 55), (9, 56), (12, 55), (15, 56), (18, 55),
                    (26, 55), (29, 56), (32, 55), (35, 56), (38, 55), (41, 56), (44, 55), (47, 56), (50, 55),
                    (61, 55), (64, 56), (67, 55), (70, 56), (73, 55), (76, 56), (79, 55)
    ]);

    // Row 13 (20 seats)
    all_seats.extend(seats![
        pattern: "f0r13s{}", 1..=20;
        kind: Kind::Mac;
        status: [Status::Free, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Taken, Status::Taken, Status::Taken,
                 Status::Taken, Status::Taken, Status::Free, Status::Taken, Status::Taken,
                 Status::Taken, Status::Free, Status::Taken, Status::Taken, Status::Taken];
        positions: (0, 60), (3, 61), (6, 60), (9, 61), (12, 60), (15, 61), (18, 60),
                    (61, 60), (64, 61), (67, 60), (70, 61), (73, 60), (76, 61), (79, 60)
    ]);
    // Create zones using the proper ZoneVec type
    let mut zones = ZoneVec::new();

    // Create zone attributes using AttributeVec
    let mut zone_attrs = AttributeVec::new();
    let _ = zone_attrs.push(Attribute::Silent);

    let _ = zones.push(Zone {
        attributes: zone_attrs,
        name: make_cluster_string("Z0")?,
        x: 4,
        y: 0,
    });

    let zone2_attrs = AttributeVec::new();
    let _ = zones.push(Zone {
        attributes: zone2_attrs,
        name: make_cluster_string("")?,
        x: 0,
        y: 1,
    });

    // Create cluster attributes using AttributeVec
    let mut cluster_attrs = AttributeVec::new();
    let _ = cluster_attrs.push(Attribute::Piscine);

    let f0 = Cluster {
        message: make_message_string("Welcome to 42!")?,
        attributes: cluster_attrs,
        name: make_cluster_string("F0")?,
        seats,
        zones,
    };

    // Create empty clusters for other floors
    let empty_cluster = empty_cluster!("");

    // Create the complete layout
    let layout: Layout = Layout {
        f0,
        f1: empty_cluster.clone(),
        f1b: empty_cluster.clone(),
        f2: empty_cluster.clone(),
        f4: empty_cluster.clone(),
        f6: empty_cluster,
    };

    Ok(layout)
}

#[embassy_executor::task]
async fn core1_task(mut led: gpio::Output<'static>) {
    info!("Core 1 - LED heartbeat for cluster hardware test");

    loop {
        led.set_high();
        Timer::after(Duration::from_secs(1)).await;
        led.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}
