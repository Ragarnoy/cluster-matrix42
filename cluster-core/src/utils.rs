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
                #[cfg(feature = "std")]
                {
                    std::vec![$($attr),*]
                }
                #[cfg(not(feature = "std"))]
                {
                    {
                        let mut attrs = heapless::Vec::new();
                        $(attrs.push($attr).expect("Too many attributes");)*
                        attrs
                    }
                }
            },
            x: $x,
            y: $y,
        }
    };
}

/// Create a cluster with the given parameters
///
/// # Example
/// ```
/// use cluster_core::{cluster, seat, zone, types::{Kind, Status, Attribute}};
///
/// let c = cluster! {
///     message: "Welcome!",
///     name: "F0",
///     attributes: [Attribute::Piscine],
///     seats: [
///         seat!("f0r1s1", Kind::Mac, Status::Free, 0, 0),
///         seat!("f0r1s2", Kind::Mac, Status::Taken, 3, 1),
///     ],
///     zones: [
///         zone!("Z1", [Attribute::Silent], 4, 0),
///     ]
/// };
/// ```
#[macro_export]
macro_rules! cluster {
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
                #[cfg(feature = "std")]
                {
                    std::vec![$($attr),*]
                }
                #[cfg(not(feature = "std"))]
                {
                    {
                        let mut attrs = heapless::Vec::new();
                        $(attrs.push($attr).expect("Too many attributes");)*
                        attrs
                    }
                }
            },
            seats: {
                #[cfg(feature = "std")]
                {
                    std::vec![$($seat),*]
                }
                #[cfg(not(feature = "std"))]
                {
                    {
                        let mut seats = heapless::Vec::new();
                        $(seats.push($seat).expect("Too many seats");)*
                        seats
                    }
                }
            },
            zones: {
                #[cfg(feature = "std")]
                {
                    std::vec![$($zone),*]
                }
                #[cfg(not(feature = "std"))]
                {
                    {
                        let mut zones = heapless::Vec::new();
                        $(zones.push($zone).expect("Too many zones");)*
                        zones
                    }
                }
            },
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
        $crate::cluster! {
            message: "",
            name: $name,
            attributes: [],
            seats: [],
            zones: []
        }
    };
}

/// Create a layout from the given clusters
///
/// # Example
/// ```
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
/// let row1 = seats![
///     pattern: "f0r1s{}", 1..=6;
///     kind: Kind::Mac;
///     status: [Status::Free, Status::Taken]; // Alternates
///     positions: (0, 0), (3, 1), (6, 0), (9, 1), (12, 0), (15, 1)
/// ];
///
/// // Create seats with same status
/// let row2 = seats![
///     pattern: "f0r2s{}", 1..=4;
///     kind: Kind::Mac;
///     status: Status::Free; // All same
///     positions: (0, 5), (3, 5), (6, 5), (9, 5)
/// ];
/// ```
#[macro_export]
macro_rules! seats {
    // Pattern with alternating status
    [
        pattern: $pattern:expr, $range:expr;
        kind: $kind:expr;
        status: [$($status:expr),+];
        positions: $(($x:expr, $y:expr)),+
    ] => {
        {
            let positions = [$(($x, $y)),+];
            let statuses = [$($status),+];
            let mut seats = {
                #[cfg(feature = "std")]
                { std::vec::Vec::new() }
                #[cfg(not(feature = "std"))]
                { heapless::Vec::new() }
            };

            for (i, (x, y)) in positions.iter().enumerate() {
                let id = {
                    #[cfg(feature = "std")]
                    {
                        use std::format;
                        format!($pattern, $range.start() + i) }
                    #[cfg(not(feature = "std"))]
                    {
                        let mut s = heapless::String::<16>::new();
                        use core::fmt::Write;
                        write!(&mut s, $pattern, $range.start + i).expect("Format error");
                        s
                    }
                };
                let status = statuses[i % statuses.len()];
                #[cfg(feature = "std")]
                seats.push($crate::seat!(id, $kind, status, *x, *y));
                #[cfg(not(feature = "std"))]
                seats.push($crate::seat!(id, $kind, status, *x, *y)).expect("Too many seats");
            }
            seats
        }
    };

    // Pattern with same status for all
    [
        pattern: $pattern:expr, $range:expr;
        kind: $kind:expr;
        status: $status:expr;
        positions: $(($x:expr, $y:expr)),+
    ] => {
        {
            let positions = [$(($x, $y)),+];
            let mut seats = {
                #[cfg(feature = "std")]
                { std::vec::Vec::new() }
                #[cfg(not(feature = "std"))]
                { heapless::Vec::new() }
            };

            for (i, (x, y)) in positions.iter().enumerate() {
                let id = {
                    #[cfg(feature = "std")]
                    {
                        use std::format;
                        format!($pattern, $range.start() + i) }
                    #[cfg(not(feature = "std"))]
                    {
                        let mut s = heapless::String::<16>::new();
                        use core::fmt::Write;
                        write!(&mut s, $pattern, $range.start + i).expect("Format error");
                        s
                    }
                };
                #[cfg(feature = "std")]
                seats.push($crate::seat!(id, $kind, $status, *x, *y));
                #[cfg(not(feature = "std"))]
                seats.push($crate::seat!(id, $kind, $status, *x, *y)).expect("Too many seats");
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
/// let mut all_seats = seats![
///     pattern: "f0r1s{}", 1..=3;
///     kind: Kind::Mac;
///     status: Status::Free;
///     positions: (0, 0), (3, 0), (6, 0)
/// ];
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
            #[cfg(feature = "std")]
            {
                $vec.extend(std::vec![$($seat),*]);
            }
            #[cfg(not(feature = "std"))]
            {
                $(
                    $vec.push($seat).expect("Too many seats in extension");
                )*
            }
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
