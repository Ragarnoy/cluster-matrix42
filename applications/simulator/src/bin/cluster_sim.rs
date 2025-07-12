use cluster_core::parsing::{Attribute, Cluster, Kind, Layout, Seat, Status, Zone};
use cluster_core::visualization::draw_cluster_frame;
// Your existing function
use simulator::create_128x128_simulator;
use std::vec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sim = create_128x128_simulator()?;

    // Create the cluster layout
    let layout = create_sample_layout()?;

    // Use your existing draw_cluster_frame function
    sim.run_with_callback(|display, frame| draw_cluster_frame(display, &layout, frame))
}

fn create_sample_seats() -> Vec<Seat> {
    vec![
        // Row 1
        Seat {
            id: "f0r1s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 0,
        },
        Seat {
            id: "f0r1s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 1,
            y: 0,
        },
        Seat {
            id: "f0r1s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 2,
            y: 0,
        },
        Seat {
            id: "f0r1s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 0,
        },
        // Row 2
        Seat {
            id: "f0r2s1".to_string(),
            kind: Kind::Dell,
            status: Status::Taken,
            x: 0,
            y: 1,
        },
        Seat {
            id: "f0r2s2".to_string(),
            kind: Kind::Dell,
            status: Status::Free,
            x: 1,
            y: 1,
        },
        Seat {
            id: "f0r2s3".to_string(),
            kind: Kind::Dell,
            status: Status::Broken,
            x: 2,
            y: 1,
        },
        Seat {
            id: "f0r2s4".to_string(),
            kind: Kind::Dell,
            status: Status::Free,
            x: 3,
            y: 1,
        },
        // Row 3
        Seat {
            id: "f0r3s1".to_string(),
            kind: Kind::Lenovo,
            status: Status::Free,
            x: 0,
            y: 2,
        },
        Seat {
            id: "f0r3s2".to_string(),
            kind: Kind::Lenovo,
            status: Status::Taken,
            x: 1,
            y: 2,
        },
        Seat {
            id: "f0r3s3".to_string(),
            kind: Kind::Lenovo,
            status: Status::Reported,
            x: 2,
            y: 2,
        },
        Seat {
            id: "f0r3s4".to_string(),
            kind: Kind::Lenovo,
            status: Status::Free,
            x: 3,
            y: 2,
        },
        // Row 4
        Seat {
            id: "f0r4s1".to_string(),
            kind: Kind::Flex,
            status: Status::Taken,
            x: 0,
            y: 3,
        },
        Seat {
            id: "f0r4s2".to_string(),
            kind: Kind::Flex,
            status: Status::Free,
            x: 1,
            y: 3,
        },
        Seat {
            id: "f0r4s3".to_string(),
            kind: Kind::Flex,
            status: Status::Taken,
            x: 2,
            y: 3,
        },
        Seat {
            id: "f0r4s4".to_string(),
            kind: Kind::Flex,
            status: Status::Free,
            x: 3,
            y: 3,
        },
    ]
}

fn create_sample_layout() -> Result<Layout, Box<dyn std::error::Error>> {
    let seats = create_sample_seats();

    let zones = vec![
        Zone {
            attributes: vec![],
            name: "Mac Zone".to_string(),
            x: 0,
            y: 0,
        },
        Zone {
            attributes: vec![Attribute::Silent],
            name: "Study Zone".to_string(),
            x: 0,
            y: 1,
        },
    ];

    // Create the F0 cluster
    let f0: Cluster = Cluster::builder()
        .message("Welcome to 42 School!".to_string())
        .attributes(vec![Attribute::Piscine])
        .name("F0".to_string())
        .seats(seats)
        .zones(zones)
        .try_into()?;

    // Create empty clusters for other floors
    let empty_cluster: Cluster = Cluster::builder()
        .message("".to_string())
        .attributes(vec![])
        .name("".to_string())
        .seats(vec![])
        .zones(vec![])
        .try_into()?;

    // Create the complete layout
    let layout: Layout = Layout::builder()
        .f0(f0)
        .f1(empty_cluster.clone())
        .f1b(empty_cluster.clone())
        .f2(empty_cluster.clone())
        .f4(empty_cluster.clone())
        .f6(empty_cluster)
        .try_into()?;

    Ok(layout)
}
