//! Layout implementations for different cluster configurations

use crate::visualization::cluster::{ClusterLayout, SeatPosition, ZoneInfo};

/// Standard rectangular grid layout
pub struct GridLayout {
    pub cols: u8,
    pub rows: u8,
    pub zones: &'static [ZoneInfo],
    pub gaps: &'static [(u8, u8)], // (after_column, gap_size)
}

impl GridLayout {
    pub const fn new(
        cols: u8,
        rows: u8,
        zones: &'static [ZoneInfo],
        gaps: &'static [(u8, u8)],
    ) -> Self {
        Self {
            cols,
            rows,
            zones,
            gaps,
        }
    }
}

impl ClusterLayout for GridLayout {
    fn seat_position(&self, index: usize) -> Option<SeatPosition> {
        if index >= (self.cols * self.rows) as usize {
            return None;
        }

        let base_x = (index as u8) % self.cols;
        let y = (index as u8) / self.cols;

        // Adjust x position for gaps between zones
        let mut x = base_x;
        for &(gap_after_col, gap_size) in self.gaps {
            if base_x > gap_after_col {
                x += gap_size;
            }
        }

        Some(SeatPosition::new(x, y))
    }

    fn seat_count(&self) -> usize {
        (self.cols * self.rows) as usize
    }

    fn grid_size(&self) -> (u8, u8) {
        let total_gaps: u8 = self.gaps.iter().map(|(_, size)| size).sum();
        (self.cols + total_gaps, self.rows)
    }

    fn zones(&self) -> &'static [ZoneInfo] {
        self.zones
    }
}

/// Custom layout with explicit seat positions
pub struct CustomLayout {
    pub positions: &'static [SeatPosition],
    pub zones: &'static [ZoneInfo],
    pub grid_width: u8,
    pub grid_height: u8,
}

impl CustomLayout {
    pub const fn new(
        positions: &'static [SeatPosition],
        zones: &'static [ZoneInfo],
        grid_width: u8,
        grid_height: u8,
    ) -> Self {
        Self {
            positions,
            zones,
            grid_width,
            grid_height,
        }
    }
}

impl ClusterLayout for CustomLayout {
    fn seat_position(&self, index: usize) -> Option<SeatPosition> {
        self.positions.get(index).copied()
    }

    fn seat_count(&self) -> usize {
        self.positions.len()
    }

    fn grid_size(&self) -> (u8, u8) {
        (self.grid_width, self.grid_height)
    }

    fn zones(&self) -> &'static [ZoneInfo] {
        self.zones
    }
}

/// Common layout presets
pub mod presets {
    use super::{CustomLayout, GridLayout, SeatPosition};
    use crate::shared::types::Zone;
    use crate::visualization::cluster::ZoneInfo;

    /// Standard 3-zone layout (15 seats wide, 8 seats tall)
    pub const ZONES_3X5X8: [ZoneInfo; 3] = [
        ZoneInfo::new(Zone::Z1, 0, 4),
        ZoneInfo::new(Zone::Z2, 5, 9),
        ZoneInfo::new(Zone::Z3, 10, 14),
    ];

    pub const GAPS_3X5: [(u8, u8); 2] = [
        (4, 1), // Gap after column 4
        (9, 1), // Gap after column 9
    ];

    pub const LAYOUT_3X5X8: GridLayout = GridLayout::new(15, 8, &ZONES_3X5X8, &GAPS_3X5);

    /// Standard 2-zone layout (12 seats wide, 10 seats tall)
    pub const ZONES_2X6X10: [ZoneInfo; 2] = [
        ZoneInfo::new(Zone::Z1, 0, 5),
        ZoneInfo::new(Zone::Z2, 6, 11),
    ];

    pub const GAPS_2X6: [(u8, u8); 1] = [
        (5, 1), // Gap after column 5
    ];

    pub const LAYOUT_2X6X10: GridLayout = GridLayout::new(12, 10, &ZONES_2X6X10, &GAPS_2X6);

    /// Standard 4-zone layout (16 seats wide, 6 seats tall)
    pub const ZONES_4X4X6: [ZoneInfo; 4] = [
        ZoneInfo::new(Zone::Z1, 0, 3),
        ZoneInfo::new(Zone::Z2, 4, 7),
        ZoneInfo::new(Zone::Z3, 8, 11),
        ZoneInfo::new(Zone::Z4, 12, 15),
    ];

    pub const GAPS_4X4: [(u8, u8); 3] = [
        (3, 1),  // Gap after column 3
        (7, 1),  // Gap after column 7
        (11, 1), // Gap after column 11
    ];

    pub const LAYOUT_4X4X6: GridLayout = GridLayout::new(16, 6, &ZONES_4X4X6, &GAPS_4X4);

    /// U-shaped custom layout positions
    pub const U_SHAPE_POSITIONS: [SeatPosition; 24] = [
        // Left column
        SeatPosition::new(0, 0),
        SeatPosition::new(0, 1),
        SeatPosition::new(0, 2),
        SeatPosition::new(0, 3),
        SeatPosition::new(0, 4),
        SeatPosition::new(0, 5),
        // Bottom row
        SeatPosition::new(1, 5),
        SeatPosition::new(2, 5),
        SeatPosition::new(3, 5),
        SeatPosition::new(4, 5),
        SeatPosition::new(5, 5),
        SeatPosition::new(6, 5),
        SeatPosition::new(7, 5),
        SeatPosition::new(8, 5),
        SeatPosition::new(9, 5),
        SeatPosition::new(10, 5),
        SeatPosition::new(11, 5),
        SeatPosition::new(12, 5),
        // Right column
        SeatPosition::new(12, 0),
        SeatPosition::new(12, 1),
        SeatPosition::new(12, 2),
        SeatPosition::new(12, 3),
        SeatPosition::new(12, 4),
        SeatPosition::new(12, 5),
    ];

    pub const U_SHAPE_ZONES: [ZoneInfo; 3] = [
        ZoneInfo::new(Zone::Z1, 0, 0),   // Left
        ZoneInfo::new(Zone::Z2, 1, 11),  // Bottom
        ZoneInfo::new(Zone::Z3, 12, 12), // Right
    ];

    pub const LAYOUT_U_SHAPE: CustomLayout =
        CustomLayout::new(&U_SHAPE_POSITIONS, &U_SHAPE_ZONES, 13, 6);
}
