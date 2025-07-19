use cluster_core::models::{Cluster, Layout, SeatVec, Zone, ZoneVec};
use cluster_core::types::{Attribute, AttributeVec, ClusterString, Kind, MessageString, Status};
use cluster_core::{empty_cluster, seats};

/// Create sample cluster layout using no_std compatible types
pub fn create_sample_layout() -> Result<Layout, &'static str> {
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
        seats: all_seats,
        zones,
    };

    let mut f1 = empty_cluster!("F1");
    f1.message = make_message_string("Coucou c'est F1 vide")?;
    let mut f1b = empty_cluster!("F1B");
    f1b.message = make_message_string("Coucou c'est F1B VIP !")?;
    let mut f2 = empty_cluster!("F2");
    f2.message = make_message_string("Coucou c'est F2 vide :(")?;
    let mut f4 = empty_cluster!("F4");
    f4.message = make_message_string("Coucou c'est F4 chut")?;
    let mut f6 = empty_cluster!("F6");
    f6.message = make_message_string("Coucou c'est haut")?;

    // Create the complete layout
    let layout: Layout = Layout {
        f0,
        f1,
        f1b,
        f2,
        f4,
        f6,
    };

    Ok(layout)
}
