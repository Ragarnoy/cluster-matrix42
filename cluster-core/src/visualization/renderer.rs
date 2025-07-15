//! Cluster visualization renderer

use crate::models::{Cluster, Layout, Seat};
use crate::types::ClusterId::F0;
use crate::types::{ClusterId, Kind, Status};
use crate::visualization::display::{
    DEFAULT_LAYOUT, DISPLAY_WIDTH, DisplayLayout, FLOOR_BAR_SPACING, FLOOR_BARS_Y,
    FLOOR_INFO_LEFT_MARGIN, FLOOR_INFO_WIDTH, FLOOR_TEXT_BASELINE_Y, FLOOR_TEXT_X,
    MOTD_LINE_HEIGHT, MOTD_TEXT_Y, SPLIT_FLOOR_GAP, STATUS_BAR_HEIGHT, STATUS_BAR_SIDE_MARGIN,
    ZONE_TEXT_Y_OFFSET, visual,
};
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;

/// Main cluster renderer
pub struct ClusterRenderer {
    layout: DisplayLayout,
    selected_cluster: ClusterId,
}

impl ClusterRenderer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            layout: DEFAULT_LAYOUT,
            selected_cluster: F0,
        }
    }

    pub fn set_selected_cluster(&mut self, selected_cluster: ClusterId) {
        self.selected_cluster = selected_cluster;
    }

    /// Render a complete frame
    pub fn render_frame<D>(
        &self,
        display: &mut D,
        layout: &Layout,
        selected_cluster: &Cluster,
        frame: u32,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Clear display
        display.clear(visual::BACKGROUND)?;

        // Render each component
        Self::render_header(display, &selected_cluster.message, frame)?;
        self.render_floors_info(display, layout)?;
        self.render_cluster::<D>(display, selected_cluster)?;
        self.render_status_bar(display, 33)?;

        Ok(())
    }

    fn render_header<D>(display: &mut D, motd: &str, frame: u32) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Scrolling text for MOTD
        let text_width = motd.len() * 6; // Approximate width with FONT_6X10
        let total_scroll_width = text_width + DISPLAY_WIDTH as usize;
        let scroll_pos = ((frame / 2) as usize) % total_scroll_width;
        let x_offset = DISPLAY_WIDTH as i32 - scroll_pos as i32;

        let style = MonoTextStyle::new(&FONT_6X10, visual::TEXT_COLOR);
        Text::new(motd, Point::new(x_offset, MOTD_TEXT_Y), style).draw(display)?;

        // Draw the message again for seamless scrolling
        if x_offset + (text_width as i32) < DISPLAY_WIDTH as i32 {
            Text::new(
                motd,
                Point::new(x_offset + text_width as i32 + 20, MOTD_TEXT_Y),
                style,
            )
            .draw(display)?;
        }

        Ok(())
    }

    fn render_floor_info<D>(
        &self,
        display: &mut D,
        cluster: &Cluster,
        origin: Point,
        width: u32,
        is_selected: bool,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let stats = cluster.get_stats();
        let occupancy = (stats.available as f32 / stats.total as f32) * 100.0;

        let bar_color = if is_selected {
            visual::FLOOR_SELECTED
        } else {
            visual::FLOOR_UNSELECTED
        };

        Rectangle::new(origin, Size::new(width, MOTD_LINE_HEIGHT))
            .into_styled(PrimitiveStyle::with_stroke(bar_color, 1))
            .draw(display)?;

        // Draw occupancy bar inside the hollow rectangle
        let bar_width = ((FLOOR_INFO_WIDTH - 4) * occupancy as u32) / 100; // Leave 2px margin on each side
        Rectangle::new(
            Point::new(origin.x + 1, origin.y + 1),
            Size::new(bar_width, MOTD_LINE_HEIGHT - 2), // Leave 2px margin top/bottom
        )
        .into_styled(PrimitiveStyle::with_fill(bar_color))
        .draw(display)?;

        Ok(())
    }

    fn render_floors_info<D>(&self, display: &mut D, layout: &Layout) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Background for floor indicator area
        self.layout
            .floor_info
            .into_styled(PrimitiveStyle::with_fill(visual::FLOOR_INDICATOR_BG))
            .draw(display)?;

        // Draw current floor text
        let mut floor_num: String<3> = String::new();
        write!(&mut floor_num, "F{}", 0).unwrap();
        let text_style = MonoTextStyle::new(&FONT_6X10, visual::TEXT_COLOR);
        Text::new(
            &floor_num,
            Point::new(FLOOR_TEXT_X, FLOOR_TEXT_BASELINE_Y),
            text_style,
        )
        .draw(display)?;

        self.render_floor_info(
            display,
            &layout.f0,
            Point::new(
                FLOOR_INFO_LEFT_MARGIN as i32,
                FLOOR_BARS_Y as i32 + (6i32 * (MOTD_LINE_HEIGHT + FLOOR_BAR_SPACING) as i32),
            ),
            FLOOR_INFO_WIDTH,
            self.selected_cluster == F0,
        )?;

        let f1_width = ((FLOOR_INFO_WIDTH - SPLIT_FLOOR_GAP) * 60) / 100;
        self.render_floor_info(
            display,
            &layout.f1,
            Point::new(
                FLOOR_INFO_LEFT_MARGIN as i32,
                FLOOR_BARS_Y as i32 + (5i32 * (MOTD_LINE_HEIGHT + FLOOR_BAR_SPACING) as i32),
            ),
            f1_width,
            false,
        )?;

        self.render_floor_info(
            display,
            &layout.f1b,
            Point::new(
                FLOOR_INFO_LEFT_MARGIN as i32 + f1_width as i32 + SPLIT_FLOOR_GAP as i32,
                FLOOR_BARS_Y as i32 + (5i32 * (MOTD_LINE_HEIGHT + FLOOR_BAR_SPACING) as i32),
            ),
            FLOOR_INFO_WIDTH - SPLIT_FLOOR_GAP - f1_width,
            false,
        )?;

        self.render_floor_info(
            display,
            &layout.f2,
            Point::new(
                FLOOR_INFO_LEFT_MARGIN as i32,
                FLOOR_BARS_Y as i32 + (4i32 * (MOTD_LINE_HEIGHT + FLOOR_BAR_SPACING) as i32),
            ),
            FLOOR_INFO_WIDTH,
            false,
        )?;

        // Inactive floor - grey filled rectangle
        Rectangle::new(
            Point::new(
                FLOOR_INFO_LEFT_MARGIN as i32,
                FLOOR_BARS_Y as i32 + (3i32 * (MOTD_LINE_HEIGHT + FLOOR_BAR_SPACING) as i32),
            ),
            Size::new(FLOOR_INFO_WIDTH, MOTD_LINE_HEIGHT),
        )
        .into_styled(PrimitiveStyle::with_fill(visual::FLOOR_INACTIVE))
        .draw(display)?;

        self.render_floor_info(
            display,
            &layout.f4,
            Point::new(
                FLOOR_INFO_LEFT_MARGIN as i32,
                FLOOR_BARS_Y as i32 + (2i32 * (MOTD_LINE_HEIGHT + FLOOR_BAR_SPACING) as i32),
            ),
            FLOOR_INFO_WIDTH,
            false,
        )?;

        // Inactive floor - grey filled rectangle
        Rectangle::new(
            Point::new(
                FLOOR_INFO_LEFT_MARGIN as i32,
                FLOOR_BARS_Y as i32 + ((MOTD_LINE_HEIGHT + FLOOR_BAR_SPACING) as i32),
            ),
            Size::new(FLOOR_INFO_WIDTH, MOTD_LINE_HEIGHT),
        )
        .into_styled(PrimitiveStyle::with_fill(visual::FLOOR_INACTIVE))
        .draw(display)?;

        self.render_floor_info(
            display,
            &layout.f6,
            Point::new(
                FLOOR_INFO_LEFT_MARGIN as i32,
                FLOOR_BARS_Y as i32, // At the top
            ),
            FLOOR_INFO_WIDTH,
            false,
        )?;

        Ok(())
    }

    fn render_status_bar<D>(&self, display: &mut D, occupancy: u8) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Background for status bar
        self.layout
            .status_bar
            .into_styled(PrimitiveStyle::with_fill(visual::STATUS_BAR_BG))
            .draw(display)?;

        // Calculate bar width based on occupancy, accounting for side margins
        let bar_area_width = self.layout.status_bar.size.width - (2 * STATUS_BAR_SIDE_MARGIN);
        let bar_width = (bar_area_width * occupancy as u32) / 100;

        // Determine color based on occupancy level
        let fill_color = match occupancy {
            0..=50 => visual::OCCUPANCY_LOW,
            51..=80 => visual::OCCUPANCY_MEDIUM,
            _ => visual::OCCUPANCY_HIGH,
        };

        // Draw the occupancy bar with precise positioning
        if bar_width > 0 {
            Rectangle::new(
                Point::new(
                    self.layout.status_bar.top_left.x + STATUS_BAR_SIDE_MARGIN as i32,
                    self.layout.status_bar.top_left.y + 2, // Small vertical centering
                ),
                Size::new(bar_width, STATUS_BAR_HEIGHT - 4), // Leave some vertical padding
            )
            .into_styled(PrimitiveStyle::with_fill(fill_color))
            .draw(display)?;
        }
        Ok(())
    }

    fn render_cluster<D>(&self, display: &mut D, cluster: &Cluster) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        if cluster.seats.is_empty() {
            return Ok(());
        }

        // Find the minimum coordinates to normalize the cluster position
        let min_x = cluster.seats.iter().map(|s| s.x).min().unwrap_or(0);
        let min_y = cluster.seats.iter().map(|s| s.y).min().unwrap_or(0);

        // Position cluster at the start of the cluster area (left-aligned, top-aligned)
        let offset_x = self.layout.cluster_area.top_left.x - min_x as i32;
        let offset_y = self.layout.cluster_area.top_left.y - min_y as i32;

        // Draw zone labels at the top of cluster area
        let zones = &cluster.zones;
        let text_style = MonoTextStyle::new(&FONT_6X10, visual::TEXT_COLOR);

        for zone in zones {
            Text::new(
                &zone.name,
                Point::new(
                    self.layout.cluster_area.top_left.x + zone.x as i32,
                    self.layout.cluster_area.top_left.y + zone.y as i32 - ZONE_TEXT_Y_OFFSET,
                ),
                text_style,
            )
            .draw(display)?;
        }

        // Render each seat at its exact coordinates (no centering, just offset to cluster area)
        for seat in &cluster.seats {
            Rectangle::new(
                Point::new(seat.x as i32 + offset_x, seat.y as i32 + offset_y),
                Size::new(visual::SEAT_SIZE, visual::SEAT_SIZE),
            )
            .into_styled(PrimitiveStyle::with_fill(Self::seat_to_color(seat)))
            .draw(display)?;
        }

        Ok(())
    }

    fn seat_to_color(seat: &Seat) -> Rgb565 {
        match (seat.kind, seat.status) {
            (Kind::Dell | Kind::Lenovo | Kind::Mac, Status::Free) => Rgb565::GREEN,
            (Kind::Dell | Kind::Lenovo | Kind::Mac, Status::Taken) => Rgb565::BLUE,
            (Kind::Dell | Kind::Lenovo | Kind::Mac, Status::Broken) => Rgb565::RED,
            (Kind::Flex, _) => Rgb565::CSS_PURPLE,
            _ => Rgb565::CSS_GRAY,
        }
    }
}

impl Default for ClusterRenderer {
    fn default() -> Self {
        Self::new()
    }
}
