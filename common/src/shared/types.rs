/// Zone identifier - limited to 4 zones
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Zone {
    Z1 = 0,
    Z2 = 1,
    Z3 = 2,
    Z4 = 3,
}

impl Zone {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Zone::Z1),
            1 => Some(Zone::Z2),
            2 => Some(Zone::Z3),
            3 => Some(Zone::Z4),
            _ => None,
        }
    }

    pub const fn as_char(&self) -> char {
        match self {
            Zone::Z1 => '1',
            Zone::Z2 => '2',
            Zone::Z3 => '3',
            Zone::Z4 => '4',
        }
    }
}

/// Floor identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Floor {
    Floor1 = 0,
    Floor2 = 1,
    Floor3 = 2,
    Floor4 = 3,
    Floor5 = 4,
}

impl Floor {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Floor::Floor1),
            1 => Some(Floor::Floor2),
            2 => Some(Floor::Floor3),
            3 => Some(Floor::Floor4),
            4 => Some(Floor::Floor5),
            _ => None,
        }
    }

    pub const fn as_number(&self) -> u8 {
        (*self as u8) + 1
    }
}
