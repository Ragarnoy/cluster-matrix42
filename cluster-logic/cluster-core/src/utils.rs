//! Utility macros for creating cluster data structures
//!
//! This module provides convenient macros for creating seats, zones, and clusters
//! without the verbosity of the builder pattern. Particularly useful for test data
//! and repetitive structure creation.

/// Create a seat with the given parameters
///
/// # Example
/// ```
/// use cluster_core::{seat, types::{Kind, Status}};
///
/// let s = seat!("f0r1s1", Kind::Mac, Status::Free, 0, 0);
/// ```
#[macro_export]
macro_rules! seat {
    ($id:expr, $kind:expr, $status:expr, $x:expr, $y:expr) => {
        $crate::models::Seat {
            id: $id.try_into().expect("Invalid seat ID"),
            kind: $kind,
            status: $status,
            x: $x,
            y: $y,
        }
    };
}

/// Create a zone with the given parameters
///
/// # Example
/// ```
/// use cluster_core::{zone, types::Attribute};
///
/// let z = zone!("Z1", [Attribute::Silent], 4, 0);
/// let empty_z = zone!("Z2", [], 8, 5);
/// ```
#[macro_export]
macro_rules! zone {
    ($name:expr, [$($attr:expr),*], $x:expr, $y:expr) => {
        $crate::models::Zone {
            name: $name.try_into().expect("Invalid zone name"),
            attributes: {
                use $crate::types::AttributeVec;
                #[allow(unused_mut)]
                let mut attrs = AttributeVec::new();
                $(
                    #[allow(unused_must_use)]
                    {
                        attrs.push($attr);
                    }
                )*
                attrs
            },
            x: $x,
            y: $y,
        }
    };
}

/// Create a cluster with the given parameters
///
/// # Example
/// ```no_run
/// use cluster_core::{cluster, seat, zone, types::{Kind, Status, Attribute}};
///
/// let c = cluster! {
///     message: "Welcome!",
///     name: "F0",
///     attributes: [Attribute::Piscine],
///     seats: [
///         seat!("f0r1s1", Kind::Mac, Status::Free, 0, 0),
///         seat!("f0r1s2", Kind::Mac, Status::Taken, 3, 1),
///     ].into(),
///     zones: [
///         zone!("Z1", [Attribute::Silent], 4, 0),
///     ].into()
/// };
/// ```
#[macro_export]
macro_rules! cluster {
    // Variant 1: Literal arrays
    {
        message: $message:expr,
        name: $name:expr,
        attributes: [$($attr:expr),*],
        seats: [$($seat:expr),*],
        zones: [$($zone:expr),*]
    } => {
        $crate::models::Cluster {
            message: $message.try_into().expect("Invalid message"),
            name: $name.try_into().expect("Invalid cluster name"),
            attributes: {
                use $crate::types::AttributeVec;
                #[allow(unused_mut)]
                let mut attrs = AttributeVec::new();
                $(
                    #[allow(unused_must_use)]
                    {
                        attrs.push($attr);
                    }
                )*
                attrs
            },
            seats: {
                use $crate::models::SeatVec;
                let mut seats = SeatVec::new();
                $(
                    #[allow(unused_must_use)]
                    {
                        seats.push($seat);
                    }
                )*
                seats
            },
            zones: {
                use $crate::models::ZoneVec;
                #[allow(unused_mut)]
                let mut zones = ZoneVec::new();
                $(
                    #[allow(unused_must_use)]
                    {
                        zones.push($zone);
                    }
                )*
                zones
            },
        }
    };

    // Variant 2: Variables (for when you have dynamic collections)
    {
        message: $message:expr,
        name: $name:expr,
        attributes: [$($attr:expr),*],
        seats: $seats:expr,
        zones: $zones:expr
    } => {
        $crate::models::Cluster {
            message: $message.try_into().expect("Invalid message"),
            name: $name.try_into().expect("Invalid cluster name"),
            attributes: {
                use $crate::types::AttributeVec;
                let mut attrs = AttributeVec::new();
                $(
                    #[allow(unused_must_use)]
                    {
                        attrs.push($attr);
                    }
                )*
                attrs
            },
            seats: $seats,
            zones: $zones,
        }
    };

    // Variant 3: All variables
    {
        message: $message:expr,
        name: $name:expr,
        attributes: $attributes:expr,
        seats: $seats:expr,
        zones: $zones:expr
    } => {
        $crate::models::Cluster {
            message: $message.try_into().expect("Invalid message"),
            name: $name.try_into().expect("Invalid cluster name"),
            attributes: $attributes,
            seats: $seats,
            zones: $zones,
        }
    };
}

/// Create an empty cluster with the given name
///
/// # Example
/// ```
/// use cluster_core::empty_cluster;
///
/// let c = empty_cluster!("F1");
/// ```
#[macro_export]
macro_rules! empty_cluster {
    ($name:expr) => {
        $crate::models::Cluster {
            message: "".try_into().expect("Invalid empty message"),
            name: $name.try_into().expect("Invalid cluster name"),
            attributes: $crate::types::AttributeVec::new(),
            seats: $crate::models::SeatVec::new(),
            zones: $crate::models::ZoneVec::new(),
        }
    };
}

/// Create a layout from the given clusters
///
/// # Example
/// ```no_run
/// use cluster_core::{layout, empty_cluster, cluster, seat, types::{Kind, Status}};
///
/// let l = layout! {
///     f0: cluster! {
///         message: "Hello",
///         name: "F0",
///         attributes: [],
///         seats: [seat!("f0r1s1", Kind::Mac, Status::Free, 0, 0)],
///         zones: []
///     },
///     f1: empty_cluster!("F1"),
///     f1b: empty_cluster!("F1B"),
///     f2: empty_cluster!("F2"),
///     f4: empty_cluster!("F4"),
///     f6: empty_cluster!("F6")
/// };
/// ```
#[macro_export]
macro_rules! layout {
    {
        f0: $f0:expr,
        f1: $f1:expr,
        f1b: $f1b:expr,
        f2: $f2:expr,
        f4: $f4:expr,
        f6: $f6:expr
    } => {
        $crate::models::Layout {
            f0: $f0,
            f1: $f1,
            f1b: $f1b,
            f2: $f2,
            f4: $f4,
            f6: $f6,
        }
    };
}

/// Generate multiple seats with a pattern
///
/// This macro helps create repetitive seat layouts common in cluster arrangements.
///
/// # Example
/// ```
/// use cluster_core::{seats, types::{Kind, Status}};
///
/// // Create a row of 6 Mac seats, alternating Free/Taken status
/// let row1 = seats! {
///     pattern: "f0r1s{}", 1..=6;
///     kind: Kind::Mac;
///     status: [Status::Free, Status::Taken]; // Alternates
///     positions: (0, 0), (3, 1), (6, 0), (9, 1), (12, 0), (15, 1)
/// };
///
/// // Create seats with same status
/// let row2 = seats! {
///     pattern: "f0r2s{}", 1..=4;
///     kind: Kind::Mac;
///     status: Status::Free; // All same
///     positions: (0, 5), (3, 5), (6, 5), (9, 5)
/// };
/// ```
#[macro_export]
macro_rules! seats {
    // Pattern with alternating status
    {
        pattern: $pattern:expr, $range:expr;
        kind: $kind:expr;
        status: [$($status:expr),+];
        positions: $(($x:expr, $y:expr)),+
    } => {
        {
            use $crate::models::SeatVec;
            let positions = [$(($x, $y)),+];
            let statuses = [$($status),+];
            let mut seats = SeatVec::new();

            for (i, (x, y)) in positions.iter().enumerate() {
                // Create the ID string
                let mut id_string = $crate::types::SeatId::default();
                {
                    use core::fmt::Write;
                    write!(&mut id_string, $pattern, $range.start() + i).expect("Format error");
                }

                let status = statuses[i % statuses.len()];
                let seat = $crate::models::Seat {
                    id: id_string,
                    kind: $kind,
                    status,
                    x: *x,
                    y: *y,
                };

                // Use the appropriate push method based on the vector type
                #[allow(unused_must_use)]
                {
                    seats.push(seat); // For std::vec::Vec, returns ()
                                     // For heapless::Vec, returns Result
                }
            }
            seats
        }
    };

    // Pattern with same status for all
    {
        pattern: $pattern:expr, $range:expr;
        kind: $kind:expr;
        status: $status:expr;
        positions: $(($x:expr, $y:expr)),+
    } => {
        {
            use $crate::models::SeatVec;
            let positions = [$(($x, $y)),+];
            let mut seats = SeatVec::new();

            for (i, (x, y)) in positions.iter().enumerate() {
                // Create the ID string
                let mut id_string = $crate::types::SeatId::default();
                {
                    use core::fmt::Write;
                    write!(&mut id_string, $pattern, $range.start() + i).expect("Format error");
                }

                let seat = $crate::models::Seat {
                    id: id_string,
                    kind: $kind,
                    status: $status,
                    x: *x,
                    y: *y,
                };

                // Use the appropriate push method based on the vector type
                #[allow(unused_must_use)]
                {
                    seats.push(seat); // For std::vec::Vec, returns ()
                                     // For heapless::Vec, returns Result
                }
            }
            seats
        }
    };
}

/// Extend a vector of seats with additional seats
///
/// # Example
/// ```
/// use cluster_core::{seats, extend_seats, seat, types::{Kind, Status}};
///
/// let mut all_seats = seats! {
///     pattern: "f0r1s{}", 1..=3;
///     kind: Kind::Mac;
///     status: Status::Free;
///     positions: (0, 0), (3, 0), (6, 0)
/// };
///
/// extend_seats!(all_seats, [
///     seat!("f0r1s4", Kind::Flex, Status::Taken, 9, 0),
///     seat!("f0r1s5", Kind::Dell, Status::Free, 12, 0)
/// ]);
/// ```
#[macro_export]
macro_rules! extend_seats {
    ($vec:expr, [$($seat:expr),*]) => {
        {
            // For std::vec::Vec, extend returns ()
            // For heapless::Vec, push returns Result, which we ignore
            $(
                #[allow(unused_must_use)]
                {
                    $vec.push($seat);
                }
            )*
        }
    };
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use crate::types::{Attribute, Kind, Status};
    use std::vec;

    #[test]
    fn test_seat_macro() {
        let s = seat!("f0r1s1", Kind::Mac, Status::Free, 10, 20);
        assert_eq!(s.id, "f0r1s1");
        assert_eq!(s.kind, Kind::Mac);
        assert_eq!(s.status, Status::Free);
        assert_eq!(s.x, 10);
        assert_eq!(s.y, 20);
    }

    #[test]
    fn test_zone_macro() {
        let z = zone!("Z1", [Attribute::Silent, Attribute::Exam], 5, 10);
        assert_eq!(z.name, "Z1");
        assert_eq!(z.attributes, vec![Attribute::Silent, Attribute::Exam]);
        assert_eq!(z.x, 5);
        assert_eq!(z.y, 10);

        let empty_z = zone!("Z2", [], 0, 0);
        assert_eq!(empty_z.attributes.len(), 0);
    }

    #[test]
    fn test_cluster_macro() {
        let c = cluster! {
            message: "Test message",
            name: "TestCluster",
            attributes: [Attribute::Piscine],
            seats: [
                seat!("s1", Kind::Mac, Status::Free, 0, 0),
                seat!("s2", Kind::Dell, Status::Taken, 1, 1)
            ],
            zones: [
                zone!("Z1", [Attribute::Silent], 2, 2)
            ]
        };

        assert_eq!(c.message, "Test message");
        assert_eq!(c.name, "TestCluster");
        assert_eq!(c.attributes, vec![Attribute::Piscine]);
        assert_eq!(c.seats.len(), 2);
        assert_eq!(c.zones.len(), 1);
    }

    #[test]
    fn test_empty_cluster_macro() {
        let c = empty_cluster!("EmptyTest");
        assert_eq!(c.name, "EmptyTest");
        assert_eq!(c.message, "");
        assert_eq!(c.attributes.len(), 0);
        assert_eq!(c.seats.len(), 0);
        assert_eq!(c.zones.len(), 0);
    }

    #[test]
    fn test_layout_macro() {
        let l = layout! {
            f0: cluster! {
                message: "F0 message",
                name: "F0",
                attributes: [],
                seats: [seat!("s1", Kind::Mac, Status::Free, 0, 0)],
                zones: []
            },
            f1: empty_cluster!("F1"),
            f1b: empty_cluster!("F1B"),
            f2: empty_cluster!("F2"),
            f4: empty_cluster!("F4"),
            f6: empty_cluster!("F6")
        };

        assert_eq!(l.f0.name, "F0");
        assert_eq!(l.f1.name, "F1");
        assert_eq!(l.f0.seats.len(), 1);
        assert_eq!(l.f1.seats.len(), 0);
    }

    #[test]
    fn test_seats_macro_alternating() {
        let seats = seats![
            pattern: "f0r1s{}", 1..=4;
            kind: Kind::Mac;
            status: [Status::Free, Status::Taken];
            positions: (0, 0), (3, 1), (6, 0), (9, 1)
        ];

        assert_eq!(seats.len(), 4);
        assert_eq!(seats[0].id, "f0r1s1");
        assert_eq!(seats[0].status, Status::Free);
        assert_eq!(seats[1].status, Status::Taken);
        assert_eq!(seats[2].status, Status::Free);
        assert_eq!(seats[3].status, Status::Taken);
    }

    #[test]
    fn test_seats_macro_same_status() {
        let seats = seats![
            pattern: "f0r2s{}", 1..=3;
            kind: Kind::Dell;
            status: Status::Broken;
            positions: (0, 5), (3, 5), (6, 5)
        ];

        assert_eq!(seats.len(), 3);
        assert_eq!(seats[0].status, Status::Broken);
        assert_eq!(seats[1].status, Status::Broken);
        assert_eq!(seats[2].status, Status::Broken);
        assert_eq!(seats[0].kind, Kind::Dell);
    }

    #[test]
    fn test_extend_seats_macro() {
        let mut seats = seats![
            pattern: "f0r1s{}", 1..=2;
            kind: Kind::Mac;
            status: Status::Free;
            positions: (0, 0), (3, 0)
        ];

        extend_seats!(
            seats,
            [
                seat!("manual1", Kind::Flex, Status::Taken, 6, 0),
                seat!("manual2", Kind::Dell, Status::Free, 9, 0)
            ]
        );

        assert_eq!(seats.len(), 4);
        assert_eq!(seats[2].id, "manual1");
        assert_eq!(seats[2].kind, Kind::Flex);
        assert_eq!(seats[3].id, "manual2");
        assert_eq!(seats[3].kind, Kind::Dell);
    }
}
