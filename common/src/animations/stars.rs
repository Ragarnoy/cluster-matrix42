//! Common star animation module
//!
//! This module contains the core animation logic for the solar system animation
//! that can be used in both simulator and hardware environments.

use core::fmt::Write;
use core::format_args;
use core::iter::Iterator;
use core::result::Result;
use core::result::Result::Ok;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::{Alignment, Text, TextStyleBuilder},
};
use heapless;

/// Draws a frame of the solar system animation
///
/// This function is designed to work with any `DrawTarget` that supports Rgb565 colors.
/// It can be used in both std and no-std environments.
pub fn draw_animation_frame<D>(display: &mut D, frame: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    // Clear display with dark blue background
    display.clear(Rgb565::new(0, 0, 16))?;

    // Calculate animation parameters
    let t = frame as f32 * 0.05;

    // Background gradient
    for y in 0..64 {
        let gradient_color = Rgb565::new(0, 0, (y >> 1) as u8);

        Rectangle::new(Point::new(0, y), Size::new(64, 1))
            .into_styled(PrimitiveStyle::with_fill(gradient_color))
            .draw(display)?;
    }

    // Draw stars
    let star_positions = [
        (5, 5),
        (15, 8),
        (25, 12),
        (45, 7),
        (55, 15),
        (8, 25),
        (17, 35),
        (28, 45),
        (48, 25),
        (58, 35),
        (10, 55),
        (20, 45),
        (40, 55),
        (52, 52),
        (38, 30),
    ];

    for (i, (x, y)) in star_positions.iter().enumerate() {
        // Each star blinks at a different rate
        let star_time = t + (i as f32 * 0.5);
        let brightness = ((libm::sin(f64::from(star_time)) * 0.5 + 0.5) * 32.0) as u8;

        if brightness > 5 {
            let star_color = Rgb565::new(brightness, brightness, brightness);

            // Draw the star as a single pixel
            Pixel(Point::new(*x, *y), star_color).draw(display)?;
        }
    }

    // Center point for the solar system
    let center = Point::new(32, 32);

    // Draw sun in the center
    Circle::new(Point::new(30, 30), 4)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_ORANGE))
        .draw(display)?;

    // Draw planet orbits
    Circle::new(Point::new(20, 20), 24)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    Circle::new(Point::new(12, 12), 40)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    Circle::new(Point::new(4, 4), 56)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    // Inner planet (fastest)
    let inner_angle = t * 1.5;
    let inner_x = center.x + (libm::cos(f64::from(inner_angle)) * 12.0) as i32;
    let inner_y = center.y + (libm::sin(f64::from(inner_angle)) * 12.0) as i32;

    Circle::new(Point::new(inner_x - 1, inner_y - 1), 2)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_RED))
        .draw(display)?;

    // Middle planet
    let middle_angle = t * 0.8;
    let middle_x = center.x + (libm::cos(f64::from(middle_angle)) * 20.0) as i32;
    let middle_y = center.y + (libm::sin(f64::from(middle_angle)) * 20.0) as i32;

    Circle::new(Point::new(middle_x - 2, middle_y - 2), 4)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_BLUE))
        .draw(display)?;

    // Outer planet (slowest)
    let outer_angle = t * 0.5;
    let outer_x = center.x + (libm::cos(f64::from(outer_angle)) * 28.0) as i32;
    let outer_y = center.y + (libm::sin(f64::from(outer_angle)) * 28.0) as i32;

    Circle::new(Point::new(outer_x - 2, outer_y - 2), 4)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_GREEN))
        .draw(display)?;

    // Occasionally show a comet
    if (frame % 180) < 60 {
        let progress = (frame % 180) as f32 / 60.0;
        // Move from top-right to bottom-left
        let comet_x = 64.0 - progress * 72.0;
        let comet_y = progress * 48.0;

        if (0.0..64.0).contains(&comet_x) && (0.0..64.0).contains(&comet_y) {
            // Comet head
            Circle::new(Point::new(comet_x as i32, comet_y as i32), 2)
                .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_WHITE))
                .draw(display)?;

            // Comet tail
            for i in 1..5 {
                let tail_x = comet_x + i as f32 * 2.0;
                let tail_y = comet_y - i as f32 * 2.0;

                if (0.0..64.0).contains(&tail_x) && (0.0..64.0).contains(&tail_y) {
                    let alpha = 1.0 - (i as f32 / 5.0);
                    let tail_color = Rgb565::new(
                        (alpha * 32.0) as u8,
                        (alpha * 64.0) as u8,
                        (alpha * 32.0) as u8,
                    );

                    Circle::new(Point::new(tail_x as i32, tail_y as i32), 1)
                        .into_styled(PrimitiveStyle::with_fill(tail_color))
                        .draw(display)?;
                }
            }
        }
    }

    // Display frame count
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CSS_WHITE);
    let text_alignment = TextStyleBuilder::new().alignment(Alignment::Center).build();

    let mut text = heapless::String::<8>::new();
    text.write_fmt(format_args!("{:03}", frame % 360)).unwrap();
    Text::with_text_style(&text, Point::new(54, 58), text_style, text_alignment).draw(display)?;

    Ok(())
}
