//! Lookup tables for color correction and processing

/// Gamma correction lookup table for better color representation on LED matrices
///
/// LED matrices have non-linear brightness curves, so we need gamma correction
/// to make colors appear more natural to human eyes. This table converts
/// linear RGB values (0-255) to gamma-corrected values.
pub static GAMMA8: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5,
    5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14,
    14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25, 25, 26, 27,
    27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72,
    73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104,
    105, 107, 109, 110, 112, 114, 115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137,
    138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175,
    177, 180, 182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
    223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
];

/// Apply gamma correction to a color component
#[inline]
pub fn gamma_correct(value: u8) -> u8 {
    GAMMA8[value as usize]
}

/// Apply gamma correction to RGB565 color components
/// Returns (r, g, b) as gamma-corrected 8-bit values
#[inline]
pub fn gamma_correct_rgb565(color: embedded_graphics_core::pixelcolor::Rgb565) -> (u8, u8, u8) {
    use embedded_graphics_core::pixelcolor::RgbColor;

    // Convert RGB565 to 8-bit values
    let r8 = (color.r() << 3) | (color.r() >> 2); // 5-bit to 8-bit
    let g8 = (color.g() << 2) | (color.g() >> 4); // 6-bit to 8-bit  
    let b8 = (color.b() << 3) | (color.b() >> 2); // 5-bit to 8-bit

    (gamma_correct(r8), gamma_correct(g8), gamma_correct(b8))
}
