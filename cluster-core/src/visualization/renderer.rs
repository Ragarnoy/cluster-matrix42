//! Cluster visualization renderer

use crate::constants::MAX_FLOORS;
use crate::shared::types::{Floor, Zone};
use crate::visualization::{
    cluster::{Cluster, ClusterLayout},
    display::{visual, DisplayLayout, DEFAULT_LAYOUT},
};
use core::fmt::Write;
use embedded_graphics::mono_font::ascii::FONT_4X6;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;

/// Main cluster renderer
pub struct ClusterRenderer {
    layout: DisplayLayout,
}

impl ClusterRenderer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            layout: DEFAULT_LAYOUT,
        }
    }

    /// Render a complete frame
    pub fn render_frame<D, L>(
        &self,
        display: &mut D,
        cluster: &Cluster<L>,
        motd: &str,
        frame: u32,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        L: ClusterLayout,
    {
        // Clear display
        display.clear(visual::BACKGROUND)?;

        // Render each component
        Self::render_header(display, motd, frame)?;
        self.render_floor_info(display, cluster.floor)?;
        self.render_cluster(display, cluster)?;
        self.render_status_bar(display, cluster.occupancy_percentage())?;

        Ok(())
    }

    fn render_header<D>(display: &mut D, motd: &str, frame: u32) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Scrolling text for MOTD
        let text_width = motd.len() * 6; // Approximate width with FONT_6X10
        let total_scroll_width = text_width + 64;
        let scroll_pos = ((frame / 2) as usize) % total_scroll_width;
        let x_offset = 64i32 - scroll_pos as i32;

        let style = MonoTextStyle::new(&FONT_6X10, visual::TEXT_COLOR);
        Text::new(motd, Point::new(x_offset, 6), style).draw(display)?;

        // Draw the message again for seamless scrolling
        if x_offset + (text_width as i32) < 64 {
            Text::new(
                motd,
                Point::new(x_offset + text_width as i32 + 20, 6),
                style,
            )
            .draw(display)?;
        }

        Ok(())
    }

    fn render_floor_info<D>(&self, display: &mut D, floor: Floor) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Background for floor indicator area
        self.layout
            .floor_info
            .into_styled(PrimitiveStyle::with_fill(visual::FLOOR_INDICATOR_BG))
            .draw(display)?;

        // Draw floor indicator bars
        let bar_height = self.layout.floor_info.size.height / MAX_FLOORS;
        let current_floor = floor as u32;

        for i in 0..MAX_FLOORS {
            let y = self.layout.floor_info.top_left.y + (i * bar_height) as i32;
            let color = if i == current_floor {
                visual::FLOOR_ACTIVE
            } else {
                visual::FLOOR_INACTIVE
            };

            // Draw floor bar with margins
            Rectangle::new(
                Point::new(self.layout.floor_info.top_left.x + 2, y + 2),
                Size::new(12, bar_height - 4),
            )
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(display)?;

            // Draw floor number
            let mut floor_num: String<2> = String::new();
            write!(&mut floor_num, "F{}", MAX_FLOORS - i).unwrap();
            let text_style = MonoTextStyle::new(&FONT_4X6, visual::TEXT_COLOR);
            Text::new(
                &floor_num,
                Point::new(
                    self.layout.floor_info.top_left.x + 4,
                    y + bar_height as i32 - 4,
                ),
                text_style,
            )
            .draw(display)?;
        }

        Ok(())
    }

    fn render_cluster<D, L>(&self, display: &mut D, cluster: &Cluster<L>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
        L: ClusterLayout,
    {
        let (grid_width, grid_height) = cluster.layout.grid_size();
        let area_width = self.layout.cluster_area.size.width;
        let area_height = self.layout.cluster_area.size.height;

        // Calculate total size needed
        let total_width = grid_width as u32 * (visual::SEAT_SIZE + visual::SEAT_SPACING);
        let total_height = grid_height as u32 * (visual::SEAT_SIZE + visual::SEAT_SPACING);

        // Center the cluster in the display area
        let offset_x = self.layout.cluster_area.top_left.x
            + ((area_width.saturating_sub(total_width)) / 2) as i32;
        let offset_y = self.layout.cluster_area.top_left.y
            + ((area_height.saturating_sub(total_height)) / 2) as i32;

        // Draw zone labels at the top
        let zones = cluster.layout.zones();
        for zone_info in zones {
            let zone_center = (zone_info.start_col + zone_info.end_col) / 2;
            let x =
                offset_x + (zone_center as i32 * (visual::SEAT_SIZE + visual::SEAT_SPACING) as i32);
            let y = offset_y - 8;

            let zone_label = match zone_info.zone {
                Zone::Z1 => "Z1",
                Zone::Z2 => "Z2",
                Zone::Z3 => "Z3",
                Zone::Z4 => "Z4",
            };

            let text_style = MonoTextStyle::new(&FONT_6X10, visual::TEXT_COLOR);
            Text::new(zone_label, Point::new(x, y + 6), text_style).draw(display)?;
        }

        // Render each seat
        for (index, seat) in cluster.seats.iter().enumerate() {
            if let Some(pos) = cluster.layout.seat_position(index) {
                let x =
                    offset_x + (pos.x as i32 * (visual::SEAT_SIZE + visual::SEAT_SPACING) as i32);
                let y =
                    offset_y + (pos.y as i32 * (visual::SEAT_SIZE + visual::SEAT_SPACING) as i32);

                Rectangle::new(
                    Point::new(x, y),
                    Size::new(visual::SEAT_SIZE, visual::SEAT_SIZE),
                )
                .into_styled(PrimitiveStyle::with_fill(seat.color()))
                .draw(display)?;
            }
        }

        // Draw zone separators (vertical lines between zones)
        for zone in zones.iter().take(zones.len().saturating_sub(1)) {
            let gap_x = zone.end_col + 1;
            let x = offset_x + (gap_x as i32 * (visual::SEAT_SIZE + visual::SEAT_SPACING) as i32)
                - (visual::ZONE_GAP / 2) as i32;

            for y_grid in 0..grid_height {
                let y =
                    offset_y + (y_grid as i32 * (visual::SEAT_SIZE + visual::SEAT_SPACING) as i32);

                // Draw a vertical separator line
                Rectangle::new(Point::new(x, y), Size::new(1, visual::SEAT_SIZE))
                    .into_styled(PrimitiveStyle::with_fill(visual::ZONE_SEPARATOR))
                    .draw(display)?;
            }
        }

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

        // Calculate bar width based on occupancy
        let bar_area_width = self.layout.status_bar.size.width - 4;
        let bar_width = (bar_area_width * occupancy as u32) / 100;

        // Determine color based on occupancy level
        let fill_color = match occupancy {
            0..=50 => visual::OCCUPANCY_LOW,
            51..=80 => visual::OCCUPANCY_MEDIUM,
            _ => visual::OCCUPANCY_HIGH,
        };

        // Draw the occupancy bar
        if bar_width > 0 {
            Rectangle::new(
                Point::new(
                    self.layout.status_bar.top_left.x + 2,
                    self.layout.status_bar.top_left.y + 2,
                ),
                Size::new(bar_width, 4),
            )
            .into_styled(PrimitiveStyle::with_fill(fill_color))
            .draw(display)?;
        }

        // Draw occupancy percentage text
        let mut percentage_text = String::<8>::new();
        write!(&mut percentage_text, "{occupancy}").unwrap();
        let mut full_text = percentage_text;
        let _ = full_text.push('%');

        let text_style = MonoTextStyle::new(&FONT_6X10, visual::TEXT_COLOR);
        Text::new(
            &full_text,
            Point::new(
                self.layout.status_bar.top_left.x + bar_area_width as i32 - 20,
                self.layout.status_bar.top_left.y + 6,
            ),
            text_style,
        )
        .draw(display)?;

        Ok(())
    }
}

impl Default for ClusterRenderer {
    fn default() -> Self {
        Self::new()
    }
}
