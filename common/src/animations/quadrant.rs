//! Quadrant panel animation for 128x128 displays
//!
//! Creates four 64x64 panels that cycle through different colors
//! every 60 frames over a 360-frame loop.

use core::fmt::Write;
use embedded_graphics::mono_font::iso_8859_16::FONT_9X18_BOLD;
use embedded_graphics::{mono_font::MonoTextStyle, text::Text};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless;

/// Color palette for the animation
const COLORS: [Rgb565; 6] = [
    Rgb565::CSS_CRIMSON,
    Rgb565::GREEN,
    Rgb565::BLUE,
    Rgb565::CSS_ORANGE,
    Rgb565::CSS_DEEP_PINK,
    Rgb565::MAGENTA,
];

/// Get color for a panel at a given frame
fn get_panel_color(panel_id: usize, frame: u32) -> Rgb565 {
    // Determine which color cycle we're in (0-5 over 360 frames)
    let cycle = (frame / 60) % 6;

    // Each panel has an offset to ensure they're always different colors
    let color_index = (cycle as usize + panel_id) % 6;

    COLORS[color_index]
}

/// Draws a frame of the quadrant panel animation
///
/// This function divides the 128x128 display into four 64x64 panels:
/// - Panel 1 (bottom left): (0, 64) to (64, 128)
/// - Panel 2 (bottom right): (64, 64) to (128, 128)
/// - Panel 3 (top left): (0, 0) to (64, 64)
/// - Panel 4 (top right): (64, 0) to (128, 64)
///
/// Each panel cycles through different colors every 60 frames.
pub fn draw_animation_frame<D>(display: &mut D, frame: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    // Ensure we loop every 360 frames
    let frame = frame % 360;

    // Clear display
    display.clear(Rgb565::BLACK)?;

    // Define panel boundaries (x, y, width, height)
    let panels = [
        (0, 64, 64, 64),  // Panel 1: Bottom left
        (64, 64, 64, 64), // Panel 2: Bottom right
        (0, 0, 64, 64),   // Panel 3: Top left
        (64, 0, 64, 64),  // Panel 4: Top right
    ];

    // Draw each panel with its current color
    for (panel_id, &(x, y, width, height)) in panels.iter().enumerate() {
        let color = get_panel_color(panel_id, frame);

        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(display)?;
    }

    // Add subtle border lines between panels for visual separation
    let border_color = Rgb565::WHITE;

    // Vertical center line
    Rectangle::new(Point::new(63, 0), Size::new(2, 128))
        .into_styled(PrimitiveStyle::with_fill(border_color))
        .draw(display)?;

    // Horizontal center line
    Rectangle::new(Point::new(0, 63), Size::new(128, 2))
        .into_styled(PrimitiveStyle::with_fill(border_color))
        .draw(display)?;

    // Add panel numbers and frame counter for debugging
    let text_style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);

    // Panel numbers
    Text::new("3", Point::new(30, 35), text_style).draw(display)?;
    Text::new("4", Point::new(94, 35), text_style).draw(display)?;
    Text::new("1", Point::new(30, 100), text_style).draw(display)?;
    Text::new("2", Point::new(94, 100), text_style).draw(display)?;

    // Frame counter
    let mut frame_text = heapless::String::<16>::new();
    write!(&mut frame_text, "F:{}", frame).unwrap();
    Text::new(&frame_text, Point::new(4, 10), text_style).draw(display)?;

    Ok(())
}
