//! Common star animation module for 128x128 displays
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
    display.clear(Rgb565::new(0, 0, 12))?;

    // Calculate animation parameters
    let t = frame as f32 * 0.05;

    // Background gradient
    for y in 0..128 {
        let gradient_color = Rgb565::new(0, 0, ((y + 4) as f32 * 0.135) as u8);

        Rectangle::new(Point::new(0, y), Size::new(128, 1))
            .into_styled(PrimitiveStyle::with_fill(gradient_color))
            .draw(display)?;
    }

    // Draw stars - properly distributed around center (64,64)
    let star_positions = [
        (8, 8),
        (24, 13),
        (40, 19),
        (72, 11),
        (88, 24),
        (13, 40),
        (27, 56),
        (45, 72),
        (77, 40),
        (93, 56),
        (16, 88),
        (32, 72),
        (64, 88),
        (83, 83),
        (61, 48),
        (6, 28),
        (20, 36),
        (36, 28),
        (52, 20),
        (68, 36),
        (84, 28),
        (12, 68),
        (28, 84),
        (44, 60),
        (60, 76),
        (76, 68),
        (92, 76),
        (18, 96),
        (34, 91),
        (50, 99),
        (66, 96),
        (82, 91),
        (98, 99),
        (4, 52),
        (100, 32),
        (96, 64),
        (4, 80),
        (100, 96),
        (48, 4),
        (80, 6),
    ];

    for (i, (x, y)) in star_positions.iter().enumerate() {
        // Each star blinks at a different rate
        let star_time = t + (i as f32 * 0.3);
        let brightness = ((libm::sin(f64::from(star_time)) * 0.5 + 0.5) * 32.0) as u8;

        if brightness > 5 {
            let star_color = Rgb565::new(brightness, brightness << 1, (y >> 2) as u8);

            // Draw larger stars occasionally
            if i % 7 == 0 {
                // Bigger star
                Circle::new(Point::new(*x - 1, *y - 1), 3)
                    .into_styled(PrimitiveStyle::with_fill(star_color))
                    .draw(display)?;
            } else {
                // Regular star
                Pixel(Point::new(*x, *y), star_color).draw(display)?;
            }
        }
    }

    // Center point for the solar system (stays centered in 128x128)
    let center = Point::new(64, 64);

    // Draw sun in the center - scaled down 20%
    Circle::new(Point::new(61, 61), 6)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_ORANGE))
        .draw(display)?;

    // Draw planet orbits - scaled down 20%
    Circle::new(Point::new(45, 45), 38)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    Circle::new(Point::new(32, 32), 64)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    Circle::new(Point::new(19, 19), 90)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    // Add an outer orbit for the larger display - scaled down 20%
    Circle::new(Point::new(6, 6), 116)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 1))
        .draw(display)?;

    // Inner planet (fastest) - scaled down 20%
    let inner_angle = t * 1.5;
    let inner_x = center.x + (libm::cos(f64::from(inner_angle)) * 19.0) as i32;
    let inner_y = center.y + (libm::sin(f64::from(inner_angle)) * 19.0) as i32;

    Circle::new(Point::new(inner_x - 2, inner_y - 2), 3)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_RED))
        .draw(display)?;

    // Middle planet - scaled down 20%
    let middle_angle = t * 0.8;
    let middle_x = center.x + (libm::cos(f64::from(middle_angle)) * 32.0) as i32;
    let middle_y = center.y + (libm::sin(f64::from(middle_angle)) * 32.0) as i32;

    Circle::new(Point::new(middle_x - 3, middle_y - 3), 5)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_BLUE))
        .draw(display)?;

    // Outer planet - scaled down 20%
    let outer_angle = t * 0.5;
    let outer_x = center.x + (libm::cos(f64::from(outer_angle)) * 45.0) as i32;
    let outer_y = center.y + (libm::sin(f64::from(outer_angle)) * 45.0) as i32;

    Circle::new(Point::new(outer_x - 3, outer_y - 3), 5)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_GREEN))
        .draw(display)?;

    // Far outer planet - scaled down 20%
    let far_angle = t * 0.3;
    let far_x = center.x + (libm::cos(f64::from(far_angle)) * 58.0) as i32;
    let far_y = center.y + (libm::sin(f64::from(far_angle)) * 58.0) as i32;

    // Only draw if within bounds
    if (0..128).contains(&far_x) && (0..128).contains(&far_y) {
        Circle::new(Point::new(far_x - 3, far_y - 3), 6)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_PURPLE))
            .draw(display)?;
    }

    // Occasionally show a comet - scaled down 20%
    if (frame % 240) < 80 {
        let progress = (frame % 240) as f32 / 80.0;
        // Move from top-right to bottom-left - scaled down 20%
        let comet_x = 128.0 - progress * 115.2;
        let comet_y = progress * 76.8;

        if (0.0..128.0).contains(&comet_x) && (0.0..128.0).contains(&comet_y) {
            // Comet head - scaled down 20%
            Circle::new(Point::new(comet_x as i32, comet_y as i32), 3)
                .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_WHITE))
                .draw(display)?;

            // Comet tail - scaled down 20%
            for i in 1..8 {
                let tail_x = comet_x + i as f32 * 2.4;
                let tail_y = comet_y - i as f32 * 2.4;

                if (0.0..128.0).contains(&tail_x) && (0.0..128.0).contains(&tail_y) {
                    let alpha = 1.0 - (i as f32 / 8.0);
                    let tail_color = Rgb565::new(
                        (alpha * 32.0) as u8,
                        (alpha * 64.0) as u8,
                        (alpha * 32.0) as u8,
                    );

                    Circle::new(Point::new(tail_x as i32, tail_y as i32), 2)
                        .into_styled(PrimitiveStyle::with_fill(tail_color))
                        .draw(display)?;
                }
            }
        }
    }

    // Display frame count - positioned for 128x128, scaled down 20%
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CSS_WHITE);
    let text_alignment = TextStyleBuilder::new().alignment(Alignment::Center).build();

    let mut text = heapless::String::<8>::new();
    text.write_fmt(format_args!("{:03}", frame % 360)).unwrap();
    Text::with_text_style(&text, Point::new(99, 106), text_style, text_alignment).draw(display)?;

    Ok(())
}
