use embedded_graphics::pixelcolor::Rgb565;

// HSV to RGB conversion without std library
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h = h % 360.0; // Ensure hue is in 0-360 range
    let c = v * s; // Chroma
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - (c * 0.75);

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    [r_prime + m, g_prime + m, b_prime + m]
}

// Convert RGB [f32; 3] to RGB565
fn rgb_to_rgb565(rgb: [f32; 3]) -> Rgb565 {
    // Clamp values to 0.0-1.0 range
    let r = clamp_f32(rgb[0], 0.0, 1.0);
    let g = clamp_f32(rgb[1], 0.0, 1.0);
    let b = clamp_f32(rgb[2], 0.0, 1.0);

    // Convert to appropriate bit ranges
    let r5 = (r * 31.0 + 0.5) as u16; // Round to nearest
    let g6 = (g * 63.0 + 0.5) as u16;
    let b5 = (b * 31.0 + 0.5) as u16;

    // Pack into RGB565 format: RRRRRGGGGGGGBBBBB
    Rgb565::new(r5 as u8, g6 as u8, b5 as u8)
}

// Manual clamp function since we don't have std
fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

// Generate a color wheel
pub struct ColorWheel {
    saturation: f32,
    value: f32,
}

impl ColorWheel {
    pub(crate) fn new(saturation: f32, value: f32) -> Self {
        Self {
            saturation: clamp_f32(saturation, 0.0, 1.0),
            value: clamp_f32(value, 0.0, 1.0),
        }
    }
    pub(crate) fn get_color_at_hue(&self, hue: f32) -> Rgb565 {
        let rgb = hsv_to_rgb(hue, self.saturation, self.value);
        rgb_to_rgb565(rgb)
    }
}
