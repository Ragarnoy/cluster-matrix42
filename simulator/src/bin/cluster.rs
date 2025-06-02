use common::shared::types::{Floor, Zone};
use common::shared::{CURRENT_CLUSTER_INDEX, SHARED_CLUSTERS, get_motd, set_motd};
use common::visualization::{
    Cluster, ClusterRenderer, GridLayout, Seat, SeatState, SeatType, presets,
};
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

// Static storage for seats
const CLUSTER_SEATS: usize = 120;

fn main() {
    // Set MOTD
    set_motd("WELCOME TO F0");

    // Configure first cluster
    let cluster = &SHARED_CLUSTERS[0];
    cluster.id.store(0, Ordering::Relaxed);
    cluster.floor.store(Floor::Floor2 as u8, Ordering::Relaxed);
    cluster.valid.store(true, Ordering::Relaxed);
    cluster.seat_count.store(120, Ordering::Relaxed);
    cluster.layout_type.store(0, Ordering::Relaxed); // Grid layout

    // Set layout parameters (15 cols, 8 rows)
    cluster.layout_params[0].store(15, Ordering::Relaxed);
    cluster.layout_params[1].store(8, Ordering::Relaxed);

    // Configure zones
    cluster.zone_starts[0].store(0, Ordering::Relaxed);
    cluster.zone_ends[0].store(4, Ordering::Relaxed);
    cluster.zone_starts[1].store(5, Ordering::Relaxed);
    cluster.zone_ends[1].store(9, Ordering::Relaxed);
    cluster.zone_starts[2].store(10, Ordering::Relaxed);
    cluster.zone_ends[2].store(14, Ordering::Relaxed);
    cluster.active_zones.store(0b0111, Ordering::Relaxed); // Zones 1-3 active

    cluster.set_name("E0");

    // Initialize some seat states
    for i in 0..120 {
        let state = if i % 3 == 0 {
            SeatState::Occupied as u8
        } else {
            SeatState::Available as u8
        };

        let seat_type = match i / 40 {
            0 => SeatType::Imac as u8,
            1 => SeatType::Flex as u8,
            _ => SeatType::Dell as u8,
        };

        let zone = match i % 15 {
            0..=4 => Zone::Z1,
            5..=9 => Zone::Z2,
            _ => Zone::Z3,
        };

        cluster.seats[i].update(state, seat_type, zone);
    }

    // Set current cluster to display
    CURRENT_CLUSTER_INDEX.store(0, Ordering::Relaxed);

    // Initialize display components
    let renderer = ClusterRenderer::new();

    // Initialize seat storage in static memory
    static mut SEATS: [Seat; CLUSTER_SEATS] = [Seat {
        state: SeatState::Available,
        seat_type: SeatType::Dell,
        zone: Zone::Z1,
    }; CLUSTER_SEATS];
    let name = "cluster";

    // Create a static layout instance
    static LAYOUT: GridLayout = presets::LAYOUT_3X5X8;

    let mut frame_counter = 0u32;
    // Configure the simulator window with a pixel scale of 8 for better visibility
    let output_settings = OutputSettingsBuilder::new()
        .scale(8)
        .pixel_spacing(1)
        .build();
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(64, 64));
    let mut window = Window::new("Cluster Matrix Simulator", &output_settings);

    'running: loop {
        // Get current cluster index
        let cluster_idx = CURRENT_CLUSTER_INDEX.load(Ordering::Relaxed) as usize;
        let shared_cluster = &SHARED_CLUSTERS[cluster_idx];

        // Read cluster data from shared state
        let floor =
            Floor::from_u8(shared_cluster.floor.load(Ordering::Relaxed)).unwrap_or(Floor::Floor1);
        let seat_count = shared_cluster.seat_count.load(Ordering::Relaxed) as usize;

        // Update local seat array from shared state
        for (i, seat) in shared_cluster
            .seats
            .iter()
            .take(seat_count.min(CLUSTER_SEATS))
            .enumerate()
        {
            let (state, seat_type, zone) = seat.read();
            unsafe {
                SEATS[i] = Seat {
                    state: SeatState::from_u8(state),
                    seat_type: SeatType::from_u8(seat_type),
                    zone,
                };
            }
        }

        // Create cluster instance for rendering
        let cluster = Cluster::new(floor, name, &LAYOUT, unsafe {
            &SEATS[..seat_count.min(CLUSTER_SEATS)]
        });

        // Get MOTD
        let motd = get_motd();

        // Render to display (you'd pass your actual display here)
        renderer
            .render_frame(&mut display, &cluster, &motd, frame_counter)
            .unwrap();

        // Update the window with the contents of the display
        window.update(&display);

        // Check for events
        for event in window.events() {
            if event == SimulatorEvent::Quit {
                break 'running;
            }
        }

        println!(
            "Rendered frame {} - Cluster: {}, Floor: {:?}, Occupancy: {}%",
            frame_counter,
            name,
            floor,
            cluster.occupancy_percentage()
        );

        frame_counter = frame_counter.wrapping_add(1);

        // Control frame rate
        thread::sleep(Duration::from_millis(16));
    }
}
