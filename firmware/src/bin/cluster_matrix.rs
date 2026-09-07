//! Production firmware for the cluster-matrix display
//!
//! Core 0: HUB75 display only (deterministic frame timing)
//! Core 1: WS2812 heartbeat + W6100 ethernet + network polling

#![no_std]
#![no_main]

use cluster_core::types::ClusterId;
use cluster_core::visualization::ClusterRenderer;
use core::ptr::addr_of_mut;
use defmt::{Debug2Format, info, unwrap, warn};
use embassy_executor::{Executor, Spawner};
use embassy_futures::yield_now;
use embassy_net::StackResources;
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net_wiznet::chip::W6100;
use embassy_net_wiznet::{Device, Runner, State as WiznetState};
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::multicore::spawn_core1;
use embassy_rp::peripherals::*;
use embassy_rp::pio::Pio;
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_rp::spi::{Async, Config as SpiConfig, Spi};
use embassy_rp::{Peri, bind_interrupts, dma, pio};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::rwlock::RwLock;
use embassy_time::{Delay, Duration, Timer, block_for};
use embedded_hal_bus::spi::ExclusiveDevice;
use firmware::{
    CORE1_STACK, DISPLAY_MEMORY, DmaChannels, EXECUTOR1, Hub75Pins, LAYOUT, LayoutLock,
    SELECTED_CLUSTER, helpers,
};
use hub75_driver::{DisplayMemory, Hub75};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs1 {
    PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH4>,
                 dma::InterruptHandler<DMA_CH5>,
                 dma::InterruptHandler<DMA_CH6>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut clk_config = embassy_rp::clocks::ClockConfig::rosc();
    clk_config.core_voltage = embassy_rp::clocks::CoreVoltage::V1_20;
    let p = embassy_rp::init(embassy_rp::config::Config::new(clk_config));

    // Boot with sample data; network polling will overwrite later
    let layout = helpers::create_sample_layout().unwrap_or_else(|_| {
        panic!("Failed to create sample cluster layout");
    });
    info!(
        "Sample cluster layout created, size: {}",
        size_of_val(&layout)
    );

    let layout = &*LAYOUT.init(RwLock::new(layout));
    let selected_cluster = &*SELECTED_CLUSTER.init(Channel::new());
    let rx = selected_cluster.receiver();
    let tx = selected_cluster.sender();

    // Core 1: WS2812 + ethernet + network polling
    spawn_core1(
        p.CORE1,
        unsafe { &mut *addr_of_mut!(CORE1_STACK) },
        move || {
            info!("Core 1 alive");

            // W6100 needs a clean reset before SPI will work
            info!("  W6100 reset...");
            let mut w6100_reset = Output::new(p.PIN_29, Level::Low);
            // Hold RST low for 10ms
            block_for(Duration::from_millis(10));
            w6100_reset.set_high();
            // Wait 50ms for W6100 to initialize after reset
            block_for(Duration::from_millis(50));
            info!("  W6100 reset done");

            // SPI + W6100 init
            let mut spi_cfg = SpiConfig::default();
            spi_cfg.frequency = 1_000_000; // Start slow for debugging
            info!("  SPI init...");
            let spi = Spi::new(
                p.SPI0, p.PIN_34, p.PIN_35, p.PIN_36, p.DMA_CH5, p.DMA_CH6, Irqs1, spi_cfg,
            );
            let cs = Output::new(p.PIN_33, Level::High);
            let w6100_int = Input::new(p.PIN_28, Pull::Up);
            let spi_dev = ExclusiveDevice::new(spi, cs, Delay).unwrap();
            info!("  SPI ready");

            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.spawn(unwrap!(ws2812_task(
                    p.PIO1, p.DMA_CH4, p.PIN_39, layout, tx
                )));
                spawner.spawn(unwrap!(ethernet_init_task(
                    spawner,
                    spi_dev,
                    w6100_int,
                    w6100_reset,
                    layout
                )));
            });
        },
    );

    // Core 0: HUB75 display only
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

    spawner.spawn(unwrap!(display_task(
        p.PIO0,
        dma_channels,
        pins,
        layout,
        rx
    )));
}

// ---------------------------------------------------------------------------
// Display (Core 0 only)
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn display_task(
    pio: Peri<'static, PIO0>,
    dma_channels: DmaChannels,
    pins: Hub75Pins,
    layout: &'static LayoutLock,
    receiver: Receiver<'static, CriticalSectionRawMutex, ClusterId, 8>,
) {
    info!("Starting Hub75 LED matrix");

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
    let mut renderer = ClusterRenderer::new();

    loop {
        let current_time = embassy_time::Instant::now();
        let elapsed = current_time.duration_since(last_time);
        let micros = elapsed.as_micros();
        let fps = if micros > 0 { 1_000_000 / micros } else { 0 };
        last_time = current_time;

        if frame_counter.is_multiple_of(60) {
            info!("FPS: {}", fps);
            if let Ok(cluster_id) = receiver.try_receive() {
                info!("Selected cluster: {:?}", Debug2Format(&cluster_id));
                renderer.set_selected_cluster(cluster_id);
            }
        }

        let anim_start = embassy_time::Instant::now();

        if let Ok(layout) = layout.try_read() {
            match renderer.render_frame(&mut display, &layout, frame_counter) {
                Ok(()) => {}
                Err(_) => {
                    info!("Failed to draw cluster frame");
                    display.draw_test_pattern();
                }
            }

            let anim_time = anim_start.elapsed();

            let commit_start = embassy_time::Instant::now();
            display.commit();
            let commit_time = commit_start.elapsed();

            if frame_counter.is_multiple_of(60) {
                info!(
                    "Draw: {}us, Commit: {}us",
                    anim_time.as_micros(),
                    commit_time.as_micros()
                );
            }
        } else {
            warn!("Failed to read layout");
        }

        frame_counter = frame_counter.wrapping_add(1);
    }
}

// ---------------------------------------------------------------------------
// Ethernet + network polling (Core 1)
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W6100,
        ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static>, Delay>,
        Input<'static>,
        Output<'static>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

/// Async W6100 init — does the async handshake, spawns driver tasks
/// and the polling task, then exits. Kept small so the future fits.
#[embassy_executor::task]
async fn ethernet_init_task(
    spawner: Spawner,
    spi_dev: ExclusiveDevice<Spi<'static, SPI0, Async>, Output<'static>, Delay>,
    w6100_int: Input<'static>,
    w6100_reset: Output<'static>,
    layout: &'static LayoutLock,
) {
    info!("Core 1 - W6100 init...");

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];
    static WIZNET_STATE: StaticCell<WiznetState<8, 8>> = StaticCell::new();
    let state = WIZNET_STATE.init(WiznetState::<8, 8>::new());

    let (device, runner) =
        embassy_net_wiznet::new(mac_addr, state, spi_dev, w6100_int, w6100_reset)
            .await
            .unwrap();
    info!("  W6100 ready");

    spawner.spawn(unwrap!(ethernet_task(runner)));

    let mut rng = RoscRng;
    let seed = rng.next_u64();

    info!("Initializing network stack...");
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, net_runner) = embassy_net::new(
        device,
        embassy_net::Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        seed,
    );

    spawner.spawn(unwrap!(net_task(net_runner)));
    spawner.spawn(unwrap!(network_poll_task(stack, layout)));
    // This task exits here — its future is freed.
}

/// Waits for DHCP, then polls the cluster API periodically.
#[embassy_executor::task]
async fn network_poll_task(stack: embassy_net::Stack<'static>, layout: &'static LayoutLock) {
    info!("Waiting for DHCP...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Network configured!");
            info!("  IP: {:?}", config.address.address());
            info!("  GW: {:?}", config.gateway);
            break;
        }
        yield_now().await;
    }

    Timer::after_secs(2).await;

    static TCP_STATE: StaticCell<TcpClientState<1, 4096, 4096>> = StaticCell::new();
    let tcp = TcpClient::new(stack, TCP_STATE.init(TcpClientState::new()));
    let dns = DnsSocket::new(stack);

    info!("Starting network polling...");
    loop {
        match poll_cluster_data(&tcp, &dns, layout).await {
            Ok(()) => info!("Poll successful"),
            Err(()) => warn!("Poll failed"),
        }
        Timer::after_secs(30).await;
    }
}

async fn poll_cluster_data<T: embedded_nal_async::TcpConnect, D: embedded_nal_async::Dns>(
    tcp: &T,
    dns: &D,
    layout: &LayoutLock,
) -> Result<(), ()> {
    use cluster_net::client::{Client, ClientConfig};
    use cluster_net::endpoints::Endpoints;

    // TODO: make this configurable
    let config = ClientConfig::new("https://example.com").map_err(|_| ())?;
    let mut client: Client<'_, _, _, 8192> = Client::new(config, tcp, dns);

    let mut buffer = [0u8; 8192];
    let new_layout = Endpoints::get_layout(&mut client, &mut buffer)
        .await
        .map_err(|_| ())?;

    let mut lock = layout.write().await;
    *lock = new_layout;
    info!("Layout updated from network");

    Ok(())
}

// ---------------------------------------------------------------------------
// WS2812 heartbeat (Core 1)
// ---------------------------------------------------------------------------

#[embassy_executor::task]
async fn ws2812_task(
    pio1: Peri<'static, PIO1>,
    dma_ch: Peri<'static, DMA_CH4>,
    ws_pin: Peri<'static, PIN_39>,
    layout: &'static LayoutLock,
    sender: Sender<'static, CriticalSectionRawMutex, ClusterId, 8>,
) {
    info!("Core 1 - WS2812 heartbeat");

    let Pio {
        mut common, sm0, ..
    } = Pio::new(pio1, Irqs1);
    let program = PioWs2812Program::new(&mut common);
    let mut ws = PioWs2812::new(&mut common, sm0, dma_ch, Irqs1, ws_pin, &program);

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
            ws.write(&[smart_leds::RGB8 { r: 0, g: 32, b: 0 }]).await;
            Timer::after(Duration::from_millis(500)).await;
            ws.write(&[smart_leds::RGB8 { r: 0, g: 0, b: 0 }]).await;
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
