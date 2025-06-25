//! Chevron arrow animation for 256x64 displays
//!
//! Creates a scrolling chevron pattern that moves from left to right and loops back.
//! Designed for the 256x64 Hub75 display with coordinate transformation.

use crate::utilities::color::*;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Triangle},
};

/// Chevron dimensions and styling constants
const CHEVRON_WIDTH: i32 = 20;
const CHEVRON_HEIGHT: i32 = 24;
const CHEVRON_SPACING: i32 = 4;
const NUM_CHEVRONS: usize = 8;
const PATTERN_WIDTH: i32 = NUM_CHEVRONS as i32 * (CHEVRON_WIDTH + CHEVRON_SPACING);

/// Effective display dimensions (accounting for coordinate transformation)
const DISPLAY_WIDTH: i32 = 128;
const DISPLAY_HEIGHT: i32 = 64;

/// Animation parameters
const ARROW_SPEED: i32 = 1; // pixels per frame
const FRAMES_PER_MOVE: u32 = 2; // Move every N frames for smooth animation
const LOOP_CYCLE: i32 = DISPLAY_WIDTH + PATTERN_WIDTH - 60; // Total frames for one complete cycle

/// Arrow colors (bright for LED matrix)
const BACKGROUND_COLOR: Rgb565 = Rgb565::BLACK;

/// Calculate the pattern's X position based on the current frame
fn get_pattern_x_position(frame: u32) -> i32 {
    let movement_frame = frame / FRAMES_PER_MOVE;
    let cycle_frame = (movement_frame as i32) % LOOP_CYCLE;
    -PATTERN_WIDTH + (cycle_frame * ARROW_SPEED)
}

/// Draw a single chevron (arrow shape) at the specified position
fn draw_chevron<D>(display: &mut D, x: i32, y: i32, color: Rgb565) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    // Create chevron shape using two triangles
    // This creates a ">" shape pointing right
    let half_height = CHEVRON_HEIGHT / 2;
    let quarter_width = CHEVRON_WIDTH / 4;

    // Only draw if the chevron is potentially visible
    if x + CHEVRON_WIDTH >= 0 && x < DISPLAY_WIDTH {
        // Upper triangle of the chevron
        Triangle::new(
            Point::new(x, y - half_height),   // Top left
            Point::new(x + quarter_width, y), // Center left
            Point::new(x + CHEVRON_WIDTH, y), // Right tip
        )
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(display)?;

        // Lower triangle of the chevron
        Triangle::new(
            Point::new(x + quarter_width, y), // Center left
            Point::new(x, y + half_height),   // Bottom left
            Point::new(x + CHEVRON_WIDTH, y), // Right tip
        )
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(display)?;
    }

    Ok(())
}

/// Draw the complete chevron pattern
fn draw_chevron_pattern<D>(
    display: &mut D,
    start_x: i32,
    y: i32,
    color: Rgb565,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    for i in 0..NUM_CHEVRONS {
        let chevron_x = start_x + (i as i32 * (CHEVRON_WIDTH + CHEVRON_SPACING));
        draw_chevron(display, chevron_x, y, color)?;
    }
    Ok(())
}

/// Draws a frame of the chevron arrow animation
///
/// The chevron pattern starts off-screen on the left, scrolls across the display,
/// and loops back when it completely exits on the right side.
///
/// Note: This accounts for the Hub75 coordinate transformation where
/// Y coordinates >= 64 get transformed to the upper panel.
pub fn draw_animation_frame<D>(display: &mut D, frame: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    // Clear display
    display.clear(BACKGROUND_COLOR)?;

    // Calculate pattern position
    let pattern_x = get_pattern_x_position(frame);

    let arrow_y = DISPLAY_HEIGHT / 2 - 1;

    let wheel = ColorWheel::new(1., 1.);
    let color = wheel.get_color_at_hue((frame / 2) as f32 % 360.);

    // Draw the main chevron pattern
    draw_chevron_pattern(display, pattern_x, arrow_y, color)?;
    Ok(())
}
