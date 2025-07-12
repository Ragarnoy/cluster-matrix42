//! Display layout constants and structures

use embedded_graphics::{
    geometry::{Point, Size},
    primitives::Rectangle,
};

/// Display dimensions
pub const DISPLAY_WIDTH: u32 = 128;
pub const DISPLAY_HEIGHT: u32 = 128;

/// Layout region heights
pub const HEADER_HEIGHT: u32 = 8;
pub const STATUS_BAR_HEIGHT: u32 = 8;
pub const FLOOR_INFO_WIDTH: u32 = 32;

/// Main display layout regions for the 64x64 matrix
#[derive(Clone, Copy, Debug)]
pub struct DisplayLayout {
    pub header: Rectangle,       // MOTD area
    pub floor_info: Rectangle,   // Left side floor indicator
    pub cluster_area: Rectangle, // Main cluster visualization
    pub status_bar: Rectangle,   // Bottom occupancy bar
}

impl DisplayLayout {
    pub const fn new() -> Self {
        Self {
            header: Rectangle::new(Point::new(0, 0), Size::new(DISPLAY_WIDTH, HEADER_HEIGHT)),
            floor_info: Rectangle::new(
                Point::new(0, HEADER_HEIGHT as i32 * 3),
                Size::new(
                    FLOOR_INFO_WIDTH,
                    DISPLAY_HEIGHT - HEADER_HEIGHT - STATUS_BAR_HEIGHT,
                ),
            ),
            cluster_area: Rectangle::new(
                Point::new(FLOOR_INFO_WIDTH as i32, HEADER_HEIGHT as i32 * 3),
                Size::new(
                    DISPLAY_WIDTH - FLOOR_INFO_WIDTH,
                    DISPLAY_HEIGHT - HEADER_HEIGHT - STATUS_BAR_HEIGHT,
                ),
            ),
            status_bar: Rectangle::new(
                Point::new(0, (DISPLAY_HEIGHT - STATUS_BAR_HEIGHT) as i32),
                Size::new(DISPLAY_WIDTH, STATUS_BAR_HEIGHT),
            ),
        }
    }
}

impl Default for DisplayLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Static instance of the default layout
pub const DEFAULT_LAYOUT: DisplayLayout = DisplayLayout::new();

/// Visual constants
pub mod visual {
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics::prelude::{RgbColor, WebColors};

    /// Background colors
    pub const BACKGROUND: Rgb565 = Rgb565::BLACK;
    pub const HEADER_BG: Rgb565 = Rgb565::BLACK;
    pub const FLOOR_INDICATOR_BG: Rgb565 = Rgb565::new(2, 2, 2);

    /// UI element colors
    pub const TEXT_COLOR: Rgb565 = Rgb565::WHITE;
    pub const FLOOR_INACTIVE: Rgb565 = Rgb565::CSS_GRAY;
    pub const FLOOR_SELECTED: Rgb565 = Rgb565::WHITE;
    pub const FLOOR_UNSELECTED: Rgb565 = Rgb565::WHITE;
    pub const ZONE_SEPARATOR: Rgb565 = Rgb565::YELLOW;

    /// Status bar colors
    pub const STATUS_BAR_BG: Rgb565 = Rgb565::new(8, 8, 8);
    pub const OCCUPANCY_LOW: Rgb565 = Rgb565::GREEN;
    pub const OCCUPANCY_MEDIUM: Rgb565 = Rgb565::YELLOW;
    pub const OCCUPANCY_HIGH: Rgb565 = Rgb565::RED;

    /// Seat rendering constants
    pub const SEAT_SIZE: u32 = 2;
    pub const SEAT_SPACING: u32 = 2;
    pub const ZONE_GAP: u32 = 4;
}
