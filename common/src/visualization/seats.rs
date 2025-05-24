//! Seat types and states

use crate::shared::types::Zone;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;

/// Seat state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SeatState {
    Available = 0,
    Occupied = 1,
    OutOfOrder = 2,
    Reserved = 3,
}

impl SeatState {
    pub const fn from_u8(value: u8) -> Self {
        match value & 0x3 {
            0 => SeatState::Available,
            1 => SeatState::Occupied,
            2 => SeatState::OutOfOrder,
            3 => SeatState::Reserved,
            _ => SeatState::Available,
        }
    }
}

/// Seat type/computer type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SeatType {
    Imac = 0,
    Flex = 1,
    Dell = 2,
    Lenovo = 3,
}

impl SeatType {
    pub const fn from_u8(value: u8) -> Self {
        match value & 0x3 {
            0 => SeatType::Imac,
            1 => SeatType::Flex,
            2 => SeatType::Dell,
            3 => SeatType::Lenovo,
            _ => SeatType::Dell,
        }
    }
}

/// Individual seat information
#[derive(Clone, Copy, Debug)]
pub struct Seat {
    pub state: SeatState,
    pub seat_type: SeatType,
    pub zone: Zone,
}

impl Seat {
    /// Create a new seat
    pub const fn new(state: SeatState, seat_type: SeatType, zone: Zone) -> Self {
        Self {
            state,
            seat_type,
            zone,
        }
    }

    /// Get the display color for this seat
    pub const fn color(&self) -> Rgb565 {
        match self.state {
            SeatState::Available => Rgb565::WHITE,
            SeatState::Occupied => match self.seat_type {
                SeatType::Imac => Rgb565::new(0, 0, 31),    // Blue
                SeatType::Flex => Rgb565::new(31, 31, 0),   // Yellow
                SeatType::Dell => Rgb565::new(0, 20, 31),   // Cyan-ish
                SeatType::Lenovo => Rgb565::new(20, 0, 31), // Purple-ish
            },
            SeatState::OutOfOrder => Rgb565::new(31, 0, 0), // Red
            SeatState::Reserved => Rgb565::new(31, 16, 0),  // Orange
        }
    }

    /// Pack seat data into a byte for efficient storage
    pub const fn pack(&self) -> u8 {
        (self.state as u8) | ((self.seat_type as u8) << 2) | ((self.zone as u8) << 4)
    }

    /// Unpack seat data from a byte
    pub const fn unpack(packed: u8) -> Self {
        Self {
            state: SeatState::from_u8(packed & 0x3),
            seat_type: SeatType::from_u8((packed >> 2) & 0x3),
            zone: match (packed >> 4) & 0x3 {
                0 => Zone::Z1,
                1 => Zone::Z2,
                2 => Zone::Z3,
                3 => Zone::Z4,
                _ => Zone::Z1,
            },
        }
    }
}

/// Default seat configuration
impl Default for Seat {
    fn default() -> Self {
        Self {
            state: SeatState::Available,
            seat_type: SeatType::Dell,
            zone: Zone::Z1,
        }
    }
}

/// Seat color constants for quick access
pub mod colors {
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics::prelude::RgbColor;

    pub const AVAILABLE: Rgb565 = Rgb565::WHITE;
    pub const OUT_OF_ORDER: Rgb565 = Rgb565::RED;

    // Occupied colors by type
    pub const OCCUPIED_IMAC: Rgb565 = Rgb565::new(0, 0, 31);
    pub const OCCUPIED_FLEX: Rgb565 = Rgb565::new(31, 31, 0);
    pub const OCCUPIED_DELL: Rgb565 = Rgb565::new(0, 20, 31);
    pub const OCCUPIED_LENOVO: Rgb565 = Rgb565::new(20, 0, 31);
}
