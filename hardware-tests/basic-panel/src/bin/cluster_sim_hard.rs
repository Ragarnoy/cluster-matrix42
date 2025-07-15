//! Hardware test for cluster visualization system
//! Combines basic-panel hardware setup with cluster simulation data

#![no_std]
#![no_main]

use basic_panel::{CORE1_STACK, DISPLAY_MEMORY, DmaChannels, EXECUTOR1, Hub75Pins};
use cluster_core::builders::AttributeVec;
use cluster_core::models::{Cluster, Layout, Seat, SeatVec, Zone, ZoneVec};
use cluster_core::types::{Attribute, ClusterString, Kind, MessageString, SeatId, Status};
use cluster_core::visualization::ClusterRenderer;
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
    let mut seats = SeatVec::new();

    // Create a realistic cluster layout - 42 Paris style
    // Row 1 (6 seats)
    let _ = seats.push(Seat {
        id: make_seat_id("f0r1s1")?,
        kind: Kind::Mac,
        status: Status::Free,
        x: 0,
        y: 0,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r1s2")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 3,
        y: 1,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r1s3")?,
        kind: Kind::Mac,
        status: Status::Free,
        x: 6,
        y: 0,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r1s4")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 9,
        y: 1,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r1s5")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 12,
        y: 0,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r1s6")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 15,
        y: 1,
    });

    // Row 2 (7 seats)
    let _ = seats.push(Seat {
        id: make_seat_id("f0r2s1")?,
        kind: Kind::Mac,
        status: Status::Free,
        x: 0,
        y: 5,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r2s2")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 3,
        y: 6,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r2s3")?,
        kind: Kind::Mac,
        status: Status::Broken,
        x: 6,
        y: 5,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r2s4")?,
        kind: Kind::Flex,
        status: Status::Taken,
        x: 9,
        y: 6,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r2s5")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 12,
        y: 5,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r2s6")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 15,
        y: 6,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r2s7")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 18,
        y: 5,
    });

    // Row 3 (7 seats)
    let _ = seats.push(Seat {
        id: make_seat_id("f0r3s1")?,
        kind: Kind::Mac,
        status: Status::Free,
        x: 0,
        y: 10,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r3s2")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 3,
        y: 11,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r3s3")?,
        kind: Kind::Dell,
        status: Status::Broken,
        x: 6,
        y: 10,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r3s4")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 9,
        y: 11,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r3s5")?,
        kind: Kind::Flex,
        status: Status::Taken,
        x: 12,
        y: 10,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r3s6")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 15,
        y: 11,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r3s7")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 18,
        y: 10,
    });

    // Row 4
    let _ = seats.push(Seat {
        id: make_seat_id("f0r4s1")?,
        kind: Kind::Mac,
        status: Status::Free,
        x: 0,
        y: 15,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r4s2")?,
        kind: Kind::Lenovo,
        status: Status::Taken,
        x: 3,
        y: 16,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r4s3")?,
        kind: Kind::Mac,
        status: Status::Free,
        x: 6,
        y: 15,
    });
    let _ = seats.push(Seat {
        id: make_seat_id("f0r4s4")?,
        kind: Kind::Mac,
        status: Status::Taken,
        x: 9,
        y: 16,
    });

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

    // Create the F0 cluster using builder pattern (no_std compatible)
    let f0: Cluster = Cluster::builder()
        .message(make_message_string("Welcome to 42!")?)
        .attributes(cluster_attrs)
        .name(make_cluster_string("F0")?)
        .seats(seats)
        .zones(zones)
        .try_into()
        .map_err(|_| "clustr failed")?;

    // Create empty clusters for other floors
    let empty_seats = SeatVec::new();
    let empty_zones = ZoneVec::new();
    let empty_attrs = AttributeVec::new();

    let empty_cluster: Cluster = Cluster::builder()
        .message(make_message_string("")?)
        .attributes(empty_attrs)
        .name(make_cluster_string("")?)
        .seats(empty_seats)
        .zones(empty_zones)
        .try_into()
        .map_err(|_| "mpt clustr failed")?;

    // Create more empty clusters (we need 5 more)
    let empty_cluster1: Cluster = Cluster::builder()
        .message(make_message_string("")?)
        .attributes(AttributeVec::new())
        .name(make_cluster_string("")?)
        .seats(SeatVec::new())
        .zones(ZoneVec::new())
        .try_into()
        .map_err(|_| "mpt clustr failed")?;

    let empty_cluster2: Cluster = Cluster::builder()
        .message(make_message_string("")?)
        .attributes(AttributeVec::new())
        .name(make_cluster_string("")?)
        .seats(SeatVec::new())
        .zones(ZoneVec::new())
        .try_into()
        .map_err(|_| "mpt clustr failed")?;

    let empty_cluster3: Cluster = Cluster::builder()
        .message(make_message_string("")?)
        .attributes(AttributeVec::new())
        .name(make_cluster_string("")?)
        .seats(SeatVec::new())
        .zones(ZoneVec::new())
        .try_into()
        .map_err(|_| "mpt clustr failed")?;

    let empty_cluster4: Cluster = Cluster::builder()
        .message(make_message_string("")?)
        .attributes(AttributeVec::new())
        .name(make_cluster_string("")?)
        .seats(SeatVec::new())
        .zones(ZoneVec::new())
        .try_into()
        .map_err(|_| "mpt clustr failed")?;

    // Create the complete layout
    let layout: Layout = Layout {
        f0,
        f1: empty_cluster,
        f1b: empty_cluster1,
        f2: empty_cluster2,
        f4: empty_cluster3,
        f6: empty_cluster4,
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
