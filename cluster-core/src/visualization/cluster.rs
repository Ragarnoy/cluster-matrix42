//! Cluster structures and traits

use crate::shared::types::{Floor, Zone};
use crate::visualization::seats::{Seat, SeatState};

/// Maximum dimensions for any cluster layout
pub const MAX_CLUSTER_WIDTH: usize = 20;
pub const MAX_CLUSTER_HEIGHT: usize = 12;

/// Position of a seat within the cluster grid
#[derive(Clone, Copy, Debug)]
pub struct SeatPosition {
    pub x: u8,
    pub y: u8,
}

impl SeatPosition {
    #[must_use]
    pub const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

/// Zone definition within a cluster
#[derive(Clone, Copy, Debug)]
pub struct ZoneInfo {
    pub zone: Zone,
    pub start_col: u8,
    pub end_col: u8,
}

impl ZoneInfo {
    #[must_use]
    pub const fn new(zone: Zone, start_col: u8, end_col: u8) -> Self {
        Self {
            zone,
            start_col,
            end_col,
        }
    }
}

/// Custom layout for a cluster
pub trait ClusterLayout {
    /// Get the position of a seat by its index
    fn seat_position(&self, index: usize) -> Option<SeatPosition>;

    /// Get the total number of seats
    fn seat_count(&self) -> usize;

    /// Get grid dimensions (width, height)
    fn grid_size(&self) -> (u8, u8);

    /// Get active zones in this cluster
    fn zones(&self) -> &'static [ZoneInfo];

    /// Get the zone for a given seat index
    fn seat_zone(&self, index: usize) -> Option<Zone> {
        if let Some(pos) = self.seat_position(index) {
            for zone_info in self.zones() {
                if pos.x >= zone_info.start_col && pos.x <= zone_info.end_col {
                    return Some(zone_info.zone);
                }
            }
        }
        None
    }
}

/// Generic cluster with custom layout
pub struct Cluster<L: ClusterLayout + 'static> {
    pub floor: Floor,
    pub name: &'static str,
    pub layout: &'static L,
    pub seats: &'static [Seat],
}

impl<L: ClusterLayout> Cluster<L> {
    pub const fn new(
        floor: Floor,
        name: &'static str,
        layout: &'static L,
        seats: &'static [Seat],
    ) -> Self {
        Self {
            floor,
            name,
            layout,
            seats,
        }
    }

    /// Calculate overall occupancy percentage
    pub fn occupancy_percentage(&self) -> u8 {
        let occupied = self
            .seats
            .iter()
            .filter(|s| s.state == SeatState::Occupied)
            .count();

        if self.seats.is_empty() {
            0
        } else {
            ((occupied * 100) / self.seats.len()) as u8
        }
    }

    /// Calculate occupancy for a specific zone
    pub fn zone_occupancy(&self, zone: Zone) -> u8 {
        let zone_seats: heapless::Vec<_, 64> = self
            .seats
            .iter()
            .enumerate()
            .filter(|(idx, _)| self.layout.seat_zone(*idx) == Some(zone))
            .map(|(_, seat)| seat)
            .collect();

        if zone_seats.is_empty() {
            return 0;
        }

        let occupied = zone_seats
            .iter()
            .filter(|s| s.state == SeatState::Occupied)
            .count();

        ((occupied * 100) / zone_seats.len()) as u8
    }

    /// Get statistics for the cluster
    pub fn get_stats(&self) -> ClusterStats {
        let mut stats = ClusterStats::default();

        for seat in self.seats {
            match seat.state {
                SeatState::Available => stats.available += 1,
                SeatState::Occupied => stats.occupied += 1,
                SeatState::OutOfOrder => stats.out_of_order += 1,
                SeatState::Reserved => stats.reserved += 1,
            }
        }

        stats.total = self.seats.len() as u16;
        stats
    }
}

/// Cluster statistics
#[derive(Clone, Copy, Debug, Default)]
pub struct ClusterStats {
    pub total: u16,
    pub available: u16,
    pub occupied: u16,
    pub out_of_order: u16,
    pub reserved: u16,
}

impl ClusterStats {
    pub const fn occupancy_percentage(&self) -> u8 {
        if self.total == 0 {
            0
        } else {
            ((self.occupied as u32 * 100) / self.total as u32) as u8
        }
    }
}
