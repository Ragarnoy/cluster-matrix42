use cluster_core::models::{Cluster, Layout, Seat, Zone};
use cluster_core::types::{Attribute, Kind, Status};
use cluster_core::visualization::draw_cluster_frame;
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
            x: 3,
            y: 1,
        },
        Seat {
            id: "f0r1s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 0,
        },
        Seat {
            id: "f0r1s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 1,
        },
        Seat {
            id: "f0r1s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 0,
        },
        Seat {
            id: "f0r1s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 1,
        },
        // Row 2
        Seat {
            id: "f0r2s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 5,
        },
        Seat {
            id: "f0r2s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 6,
        },
        Seat {
            id: "f0r2s3".to_string(),
            kind: Kind::Mac,
            status: Status::Broken,
            x: 6,
            y: 5,
        },
        Seat {
            id: "f0r2s4".to_string(),
            kind: Kind::Flex,
            status: Status::Taken,
            x: 9,
            y: 6,
        },
        Seat {
            id: "f0r2s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 5,
        },
        Seat {
            id: "f0r2s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 6,
        },
        Seat {
            id: "f0r2s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 5,
        },
        // Row 3
        Seat {
            id: "f0r3s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 10,
        },
        Seat {
            id: "f0r3s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 11,
        },
        Seat {
            id: "f0r3s3".to_string(),
            kind: Kind::Mac,
            status: Status::Broken,
            x: 6,
            y: 10,
        },
        Seat {
            id: "f0r3s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 11,
        },
        Seat {
            id: "f0r3s5".to_string(),
            kind: Kind::Flex,
            status: Status::Taken,
            x: 12,
            y: 10,
        },
        Seat {
            id: "f0r3s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 11,
        },
        Seat {
            id: "f0r3s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 10,
        },
        // Row 4
        Seat {
            id: "f0r4s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 15,
        },
        Seat {
            id: "f0r4s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 16,
        },
        Seat {
            id: "f0r4s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 15,
        },
        Seat {
            id: "f0r4s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 16,
        },
        Seat {
            id: "f0r4s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 15,
        },
        Seat {
            id: "f0r4s6".to_string(),
            kind: Kind::Mac,
            status: Status::Broken,
            x: 15,
            y: 16,
        },
        Seat {
            id: "f0r4s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 15,
        },
        // Row 5
        Seat {
            id: "f0r5s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 20,
        },
        Seat {
            id: "f0r5s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 21,
        },
        Seat {
            id: "f0r5s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 20,
        },
        Seat {
            id: "f0r5s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 21,
        },
        Seat {
            id: "f0r5s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 20,
        },
        Seat {
            id: "f0r5s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 21,
        },
        Seat {
            id: "f0r5s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 20,
        },
        // Row 6
        Seat {
            id: "f0r6s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 25,
        },
        Seat {
            id: "f0r6s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 26,
        },
        Seat {
            id: "f0r6s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 25,
        },
        Seat {
            id: "f0r6s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 26,
        },
        Seat {
            id: "f0r6s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 25,
        },
        Seat {
            id: "f0r6s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 26,
        },
        // Row 7
        Seat {
            id: "f0r7s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 30,
        },
        Seat {
            id: "f0r7s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 31,
        },
        Seat {
            id: "f0r7s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 30,
        },
        Seat {
            id: "f0r7s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 31,
        },
        Seat {
            id: "f0r7s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 30,
        },
        Seat {
            id: "f0r7s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 31,
        },
        // Row 8
        Seat {
            id: "f0r8s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 35,
        },
        Seat {
            id: "f0r8s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 36,
        },
        Seat {
            id: "f0r8s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 35,
        },
        Seat {
            id: "f0r8s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 36,
        },
        Seat {
            id: "f0r8s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 35,
        },
        Seat {
            id: "f0r8s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 36,
        },
        Seat {
            id: "f0r8s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 35,
        },
        // Row 9
        Seat {
            id: "f0r9s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 40,
        },
        Seat {
            id: "f0r9s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 41,
        },
        Seat {
            id: "f0r9s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 40,
        },
        Seat {
            id: "f0r9s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 41,
        },
        Seat {
            id: "f0r9s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 40,
        },
        Seat {
            id: "f0r9s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 41,
        },
        Seat {
            id: "f0r9s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 40,
        },
        // Row 10
        Seat {
            id: "f0r10s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 45,
        },
        Seat {
            id: "f0r10s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 46,
        },
        Seat {
            id: "f0r10s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 45,
        },
        Seat {
            id: "f0r10s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 46,
        },
        Seat {
            id: "f0r10s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 45,
        },
        Seat {
            id: "f0r10s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 46,
        },
        Seat {
            id: "f0r10s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 45,
        },
        // Row 11
        Seat {
            id: "f0r11s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 50,
        },
        Seat {
            id: "f0r11s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 51,
        },
        Seat {
            id: "f0r11s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 50,
        },
        Seat {
            id: "f0r11s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 51,
        },
        Seat {
            id: "f0r11s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 50,
        },
        Seat {
            id: "f0r11s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 51,
        },
        Seat {
            id: "f0r11s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 50,
        },
        // Row 12
        Seat {
            id: "f0r12s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 55,
        },
        Seat {
            id: "f0r12s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 56,
        },
        Seat {
            id: "f0r12s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 55,
        },
        Seat {
            id: "f0r12s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 56,
        },
        Seat {
            id: "f0r12s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 55,
        },
        Seat {
            id: "f0r12s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 56,
        },
        Seat {
            id: "f0r12s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 55,
        },
        // Row 13
        Seat {
            id: "f0r13s1".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 0,
            y: 60,
        },
        Seat {
            id: "f0r13s2".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 3,
            y: 61,
        },
        Seat {
            id: "f0r13s3".to_string(),
            kind: Kind::Mac,
            status: Status::Free,
            x: 6,
            y: 60,
        },
        Seat {
            id: "f0r13s4".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 9,
            y: 61,
        },
        Seat {
            id: "f0r13s5".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 12,
            y: 60,
        },
        Seat {
            id: "f0r13s6".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 15,
            y: 61,
        },
        Seat {
            id: "f0r13s7".to_string(),
            kind: Kind::Mac,
            status: Status::Taken,
            x: 18,
            y: 60,
        },
    ]
}

fn create_sample_layout() -> Result<Layout, Box<dyn std::error::Error>> {
    let seats = create_sample_seats();

    let zones = vec![
        Zone {
            attributes: vec![],
            name: "Z0".to_string(),
            x: 4,
            y: 0,
        },
        Zone {
            attributes: vec![Attribute::Silent],
            name: "".to_string(),
            x: 0,
            y: 1,
        },
    ];

    // Create the F0 cluster
    let f0 = Cluster {
        message: "Hello World!".to_string(),
        attributes: vec![Attribute::Piscine],
        name: "F0".to_string(),
        seats,
        zones,
    };

    // Create empty clusters for other floors
    let empty_cluster = Cluster {
        message: "".to_string(),
        attributes: vec![],
        name: "".to_string(),
        seats: vec![],
        zones: vec![],
    };

    // Create the complete layout
    let layout = Layout {
        f0,
        f1: empty_cluster.clone(),
        f1b: empty_cluster.clone(),
        f2: empty_cluster.clone(),
        f4: empty_cluster.clone(),
        f6: empty_cluster,
    };

    Ok(layout)
}
