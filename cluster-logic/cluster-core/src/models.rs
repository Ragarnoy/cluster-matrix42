//! Main data models for cluster representation

use crate::types::AttributeVec;
use crate::types::{ClusterId, ClusterString, Kind, MessageString, SeatId, Status};
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
pub type SeatVec = std::vec::Vec<Seat>;
#[cfg(not(feature = "std"))]
pub type SeatVec = heapless::Vec<Seat, { crate::constants::MAX_SEATS_PER_CLUSTER }>;

#[cfg(feature = "std")]
pub type ZoneVec = std::vec::Vec<Zone>;
#[cfg(not(feature = "std"))]
pub type ZoneVec = heapless::Vec<Zone, { crate::constants::MAX_ZONES }>;

#[doc = "`ClusterUpdate`"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ClusterUpdate {
    pub attributes: AttributeVec,
    pub id: ClusterId,
    pub name: ClusterString,
    pub zones: ZoneVec,
}

#[doc = "`Layout`"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Layout {
    pub f0: Cluster,
    pub f1: Cluster,
    pub f1b: Cluster,
    pub f2: Cluster,
    pub f4: Cluster,
    pub f6: Cluster,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Seat {
    pub id: SeatId,
    pub kind: Kind,
    pub status: Status,
    pub x: usize,
    pub y: usize,
}

impl Seat {
    /// Get the display color for this seat based on its status and kind
    pub const fn color(&self) -> embedded_graphics::pixelcolor::Rgb565 {
        match self.status {
            Status::Free => self.status.color(),
            Status::Taken => self.kind.taken_color(),
            Status::Broken | Status::Reported => self.status.color(),
        }
    }
}

#[doc = "`Zone`"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Zone {
    pub attributes: AttributeVec,
    pub name: ClusterString,
    pub x: usize,
    pub y: usize,
}

#[doc = "`Cluster`"]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Cluster {
    pub message: MessageString,
    pub attributes: AttributeVec,
    pub name: ClusterString,
    pub seats: SeatVec,
    pub zones: ZoneVec,
}

impl Cluster {
    /// Get the grid dimensions based on seat positions
    pub fn grid_size(&self) -> (usize, usize) {
        if self.seats.is_empty() {
            return (0, 0);
        }

        let min_x = self.seats.iter().map(|p| p.x).min().unwrap_or(0);
        let max_x = self.seats.iter().map(|p| p.x).max().unwrap_or(0);
        let min_y = self.seats.iter().map(|p| p.y).min().unwrap_or(0);
        let max_y = self.seats.iter().map(|p| p.y).max().unwrap_or(0);

        (max_x - min_x + 1, max_y - min_y + 1)
    }

    /// Calculate overall occupancy percentage
    pub fn occupancy_percentage(&self) -> u8 {
        let occupied = self
            .seats
            .iter()
            .filter(|s| s.status == Status::Taken)
            .count();

        if self.seats.is_empty() {
            0
        } else {
            ((occupied * 100) / self.seats.len()) as u8
        }
    }

    /// Get statistics for the cluster
    pub fn get_stats(&self) -> ClusterStats {
        let mut stats = ClusterStats::default();

        for seat in &self.seats {
            match seat.status {
                Status::Free => stats.available += 1,
                Status::Taken => stats.occupied += 1,
                Status::Broken => stats.out_of_order += 1,
                Status::Reported => {}
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
