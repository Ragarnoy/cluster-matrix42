//! Display layout constants and structures

use embedded_graphics::{
    geometry::{Point, Size},
    primitives::Rectangle,
};

/// Display dimensions
pub const DISPLAY_WIDTH: u32 = 128;
pub const DISPLAY_HEIGHT: u32 = 128;

/// Header/MOTD constants
pub const HEADER_TOP_MARGIN: u32 = 2;
pub const MOTD_LINE_HEIGHT: u32 = 7;
pub const MOTD_LINE_SPACING: u32 = 2;
pub const MOTD_MAX_LINES: u32 = 3;
pub const HEADER_HEIGHT: u32 = HEADER_TOP_MARGIN
    + (MOTD_MAX_LINES * MOTD_LINE_HEIGHT)
    + ((MOTD_MAX_LINES - 1) * MOTD_LINE_SPACING); // 27px

/// Spacing from header to other elements
pub const HEADER_TO_FLOOR_TEXT_GAP: u32 = 7;
pub const HEADER_TO_CLUSTER_GAP: u32 = 18;

/// Floor info constants
pub const FLOOR_INFO_LEFT_MARGIN: u32 = 5;
pub const FLOOR_INFO_WIDTH: u32 = 24; // Estimated based on typical floor indicator size
pub const FLOOR_TEXT_TO_BARS_GAP: u32 = 5;
pub const FLOOR_BAR_SPACING: u32 = 3;
pub const FLOOR_INDICATOR_COUNT: usize = 5; // F0, F1, F1B, F2, F4, F6

/// Cluster area constants
pub const FLOOR_INFO_TO_CLUSTER_GAP: u32 = 6;
pub const CLUSTER_RIGHT_MARGIN: u32 = 6;

/// Status bar constants
pub const STATUS_BAR_HEIGHT: u32 = 8; // Estimated from image
pub const STATUS_BAR_BOTTOM_MARGIN: u32 = 3;
pub const STATUS_BAR_SIDE_MARGIN: u32 = 3;
pub const FLOOR_TO_STATUS_GAP: u32 = 14;

/// Calculated positions
pub const FLOOR_TEXT_Y: u32 = HEADER_HEIGHT + HEADER_TO_FLOOR_TEXT_GAP;
pub const CLUSTER_AREA_Y: u32 = HEADER_HEIGHT + HEADER_TO_CLUSTER_GAP;
pub const FLOOR_BARS_Y: u32 = FLOOR_TEXT_Y + MOTD_LINE_HEIGHT + FLOOR_TEXT_TO_BARS_GAP;
pub const CLUSTER_AREA_X: u32 =
    FLOOR_INFO_LEFT_MARGIN + FLOOR_INFO_WIDTH + FLOOR_INFO_TO_CLUSTER_GAP;
pub const CLUSTER_AREA_WIDTH: u32 = DISPLAY_WIDTH - CLUSTER_AREA_X - CLUSTER_RIGHT_MARGIN;

/// Status bar positioning
pub const STATUS_BAR_Y: u32 = DISPLAY_HEIGHT - STATUS_BAR_HEIGHT - STATUS_BAR_BOTTOM_MARGIN;
pub const STATUS_BAR_CONTENT_Y: u32 = FLOOR_BARS_Y
    + (FLOOR_INDICATOR_COUNT as u32 * (MOTD_LINE_HEIGHT + FLOOR_BAR_SPACING))
    + FLOOR_TO_STATUS_GAP;

/// Derived cluster area height
pub const CLUSTER_AREA_HEIGHT: u32 = STATUS_BAR_Y - CLUSTER_AREA_Y;

/// Text positioning helpers
pub const MOTD_TEXT_Y: i32 = (HEADER_TOP_MARGIN + MOTD_LINE_HEIGHT - 1) as i32; // Baseline position
pub const FLOOR_TEXT_X: i32 = (FLOOR_INFO_LEFT_MARGIN + 2) as i32;
pub const FLOOR_TEXT_BASELINE_Y: i32 = (FLOOR_TEXT_Y + MOTD_LINE_HEIGHT - 1) as i32; // Baseline position

/// Main display layout regions for the 128x128 matrix
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
                Point::new(FLOOR_INFO_LEFT_MARGIN as i32, FLOOR_TEXT_Y as i32),
                Size::new(FLOOR_INFO_WIDTH, STATUS_BAR_Y - FLOOR_TEXT_Y),
            ),
            cluster_area: Rectangle::new(
                Point::new(CLUSTER_AREA_X as i32, CLUSTER_AREA_Y as i32),
                Size::new(CLUSTER_AREA_WIDTH, CLUSTER_AREA_HEIGHT),
            ),
            status_bar: Rectangle::new(
                Point::new(STATUS_BAR_SIDE_MARGIN as i32, STATUS_BAR_Y as i32),
                Size::new(
                    DISPLAY_WIDTH - (2 * STATUS_BAR_SIDE_MARGIN),
                    STATUS_BAR_HEIGHT,
                ),
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
    pub const ZONE_GAP: u32 = 4;
}
