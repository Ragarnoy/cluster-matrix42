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
/// This function is designed to work with any DrawTarget that supports Rgb565 colors.
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
        let gradient_color = Rgb565::new(0, 0, ((y as f32 / 64.0) * 31.0) as u8);

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
        let brightness = ((libm::sin(star_time as f64) * 0.5 + 0.5) * 31.0) as u8;

        if brightness > 5 {
            let star_color = Rgb565::new(brightness, brightness, brightness);

            // Draw the star as a single pixel
            Pixel(Point::new(*x, *y), star_color).draw(display)?;
        }
    }

    // Center point for the solar system
    let center = Point::new(32, 32);

    // Draw sun in the center
    Circle::new(center, 4)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_ORANGE))
        .draw(display)?;

    // Glowing effect around the sun
    let pulse = (libm::cos(t as f64) * 0.5 + 1.5) as u32;
    Circle::new(center, 4 + pulse)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_YELLOW, 1))
        .draw(display)?;

    // Draw planet orbits
    Circle::new(center, 12)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    Circle::new(center, 20)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    Circle::new(center, 27)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    // Inner planet (fastest)
    let inner_angle = t * 1.5;
    let inner_x = center.x + (libm::cos(inner_angle as f64) * 12.0) as i32;
    let inner_y = center.y + (libm::sin(inner_angle as f64) * 12.0) as i32;

    Circle::new(Point::new(inner_x, inner_y), 2)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_RED))
        .draw(display)?;

    // Middle planet
    let middle_angle = t * 0.8;
    let middle_x = center.x + (libm::cos(middle_angle as f64) * 20.0) as i32;
    let middle_y = center.y + (libm::sin(middle_angle as f64) * 20.0) as i32;

    Circle::new(Point::new(middle_x, middle_y), 3)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_BLUE))
        .draw(display)?;

    // Outer planet (slowest)
    let outer_angle = t * 0.5;
    let outer_x = center.x + (libm::cos(outer_angle as f64) * 27.0) as i32;
    let outer_y = center.y + (libm::sin(outer_angle as f64) * 27.0) as i32;

    Circle::new(Point::new(outer_x, outer_y), 4)
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
                        (alpha * 31.0) as u8,
                        (alpha * 63.0) as u8,
                        (alpha * 31.0) as u8,
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
