//! Cluster visualization renderer

use crate::constants::MAX_FLOORS;
use crate::parsing::Cluster;
use crate::visualization::display::{DEFAULT_LAYOUT, DisplayLayout, visual};
use core::fmt::Write;
use embedded_graphics::mono_font::ascii::FONT_4X6;
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
}

impl ClusterRenderer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            layout: DEFAULT_LAYOUT,
        }
    }

    /// Render a complete frame
    pub fn render_frame<D>(
        &self,
        display: &mut D,
        cluster: &Cluster,
        frame: u32,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Clear display
        display.clear(visual::BACKGROUND)?;

        // Render each component
        Self::render_header(display, &cluster.message, frame)?;
        self.render_floor_info(display, &cluster)?;
        self.render_cluster::<D>(display, &cluster)?;
        self.render_status_bar(display, 0)?;

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

    fn render_floor_info<D>(&self, display: &mut D, _cluster: &Cluster) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // Background for floor indicator area
        self.layout
            .floor_info
            .into_styled(PrimitiveStyle::with_fill(visual::FLOOR_INDICATOR_BG))
            .draw(display)?;

        // Draw floor indicator bars
        let bar_height = self.layout.floor_info.size.height / MAX_FLOORS as u32;
        // TODO change to value from app state
        let current_floor = 0;

        for i in 0..MAX_FLOORS {
            let y = self.layout.floor_info.top_left.y + (i as u32 * bar_height) as i32;
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

    fn render_cluster<D>(&self, display: &mut D, cluster: &Cluster) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let (grid_width, grid_height) = cluster.grid_size();
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
        let zones = &cluster.zones;
        let text_style = MonoTextStyle::new(&FONT_6X10, visual::TEXT_COLOR);

        for zone in zones {
            Text::new(
                &zone.name,
                Point::new(zone.x as i32, zone.y as i32),
                text_style,
            )
            .draw(display)?;
        }

        // Render each seat
        for seat in &cluster.seats {
            Rectangle::new(
                Point::new(seat.x as i32 + offset_x, seat.y as i32 + offset_y),
                Size::new(visual::SEAT_SIZE, visual::SEAT_SIZE),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(display)?;
        }

        // // Draw zone separators (vertical lines between zones)
        // for zone in zones.iter().take(zones.len().saturating_sub(1)) {
        //     let gap_x = zone.end_col + 1;
        //     let x = offset_x + (gap_x as i32 * (visual::SEAT_SIZE + visual::SEAT_SPACING) as i32)
        //         - (visual::ZONE_GAP / 2) as i32;
        //
        //     for y_grid in 0..grid_height {
        //         let y =
        //             offset_y + (y_grid as i32 * (visual::SEAT_SIZE + visual::SEAT_SPACING) as i32);
        //
        //         // Draw a vertical separator line
        //         Rectangle::new(Point::new(x, y), Size::new(1, visual::SEAT_SIZE))
        //             .into_styled(PrimitiveStyle::with_fill(visual::ZONE_SEPARATOR))
        //             .draw(display)?;
        //     }
        // }

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
