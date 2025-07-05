use crate::constants::{MAX_CLUSTERS, MAX_SEATS_PER_CLUSTER};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use heapless::String;
use types::Zone;

pub mod types;

/// Atomic seat state for lock-free updates
#[repr(C)]
pub struct AtomicSeatState {
    /// Packed byte: [state(2 bits)][type(2 bits)][zone(2 bits)][reserved(2 bits)]
    packed: AtomicU8,
}

impl Default for AtomicSeatState {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicSeatState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            packed: AtomicU8::new(0),
        }
    }

    pub fn update(&self, state: u8, seat_type: u8, zone: Zone) {
        let packed = (state & 0x3) | ((seat_type & 0x3) << 2) | ((zone as u8 & 0x3) << 4);
        self.packed.store(packed, Ordering::Relaxed);
    }

    pub fn read(&self) -> (u8, u8, Zone) {
        let packed = self.packed.load(Ordering::Relaxed);
        let state = packed & 0x3;
        let seat_type = (packed >> 2) & 0x3;
        let zone = Zone::from_u8((packed >> 4) & 0x3).unwrap_or(Zone::Z1);
        (state, seat_type, zone)
    }
}

/// Shared cluster state that can be updated by Core 1 and read by Core 0
#[repr(C)]
pub struct SharedClusterState {
    /// Cluster identifier
    pub id: AtomicU8,

    /// Floor this cluster is on
    pub floor: AtomicU8,

    /// Whether this cluster data is valid
    pub valid: AtomicBool,

    /// Number of seats in this cluster
    pub seat_count: AtomicU8,

    /// Layout type (0=Grid, 1=Custom, etc.)
    pub layout_type: AtomicU8,

    /// Layout-specific parameters (cols, rows, etc.)
    pub layout_params: [AtomicU8; 8],

    /// Seat states
    pub seats: [AtomicSeatState; MAX_SEATS_PER_CLUSTER],

    /// Zone boundaries (`start_col` for each zone)
    pub zone_starts: [AtomicU8; 4],

    /// Zone ends (`end_col` for each zone)
    pub zone_ends: [AtomicU8; 4],

    /// Which zones are active
    pub active_zones: AtomicU8, // Bitmask

    /// Cluster name (short)
    pub name_chars: [AtomicU8; 16],
}

impl Default for SharedClusterState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedClusterState {
    pub const fn new() -> Self {
        Self {
            id: AtomicU8::new(0),
            floor: AtomicU8::new(0),
            valid: AtomicBool::new(false),
            seat_count: AtomicU8::new(0),
            layout_type: AtomicU8::new(0),
            layout_params: [const { AtomicU8::new(0) }; 8],
            seats: [const { AtomicSeatState::new() }; MAX_SEATS_PER_CLUSTER],
            zone_starts: [const { AtomicU8::new(0) }; 4],
            zone_ends: [const { AtomicU8::new(0) }; 4],
            active_zones: AtomicU8::new(0),
            name_chars: [const { AtomicU8::new(0) }; 16],
        }
    }

    pub fn set_name(&self, name: &str) {
        for (i, ch) in name.bytes().enumerate().take(16) {
            self.name_chars[i].store(ch, Ordering::Relaxed);
        }
        // Clear remaining chars
        for i in name.len()..16 {
            self.name_chars[i].store(0, Ordering::Relaxed);
        }
    }

    pub fn get_name(&self) -> String<16> {
        let mut name = String::new();
        for atomic_char in &self.name_chars {
            let ch = atomic_char.load(Ordering::Relaxed);
            if ch == 0 {
                break;
            }
            let _ = name.push(ch as char);
        }
        name
    }
}

/// Global shared state for all clusters
pub static SHARED_CLUSTERS: [SharedClusterState; MAX_CLUSTERS] =
    [const { SharedClusterState::new() }; MAX_CLUSTERS];

/// Currently selected cluster for display
pub static CURRENT_CLUSTER_INDEX: AtomicU8 = AtomicU8::new(0);

/// Message of the day
pub static MOTD_CHARS: [AtomicU8; 64] = [const { AtomicU8::new(0) }; 64];

pub fn set_motd(motd: &str) {
    for (i, ch) in motd.bytes().enumerate().take(64) {
        MOTD_CHARS[i].store(ch, Ordering::Relaxed);
    }
    // Clear remaining
    for motd_char in MOTD_CHARS.iter().skip(motd.len()) {
        motd_char.store(0, Ordering::Relaxed);
    }
}

#[must_use]
pub fn get_motd() -> String<64> {
    let mut motd = String::new();
    for atomic_char in &MOTD_CHARS {
        let ch = atomic_char.load(Ordering::Relaxed);
        if ch == 0 {
            break;
        }
        let _ = motd.push(ch as char);
    }
    motd
}
