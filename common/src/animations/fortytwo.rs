use embedded_graphics::geometry::Size;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Drawable, Point, Primitive, RgbColor},
    primitives::{PrimitiveStyle, Triangle},
};

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

// Manual absolute value function
fn abs_f32(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

// Generate a color wheel
struct ColorWheel {
    saturation: f32,
    value: f32,
}

impl ColorWheel {
    fn new(saturation: f32, value: f32) -> Self {
        Self {
            saturation: clamp_f32(saturation, 0.0, 1.0),
            value: clamp_f32(value, 0.0, 1.0),
        }
    }
    fn get_color_at_hue(&self, hue: f32) -> Rgb565 {
        let rgb = hsv_to_rgb(hue, self.saturation, self.value);
        rgb_to_rgb565(rgb)
    }
}

fn project(v: Vec3, d: f32, scale: f32) -> Vec3 {
    let denominator = v.z + d;
    if denominator.abs() < f32::EPSILON {
        return Vec3::new(0.0, 0.0, 0.0);
    }
    let factor = scale / denominator;

    Vec3::new(v.x * factor, v.y * factor, 0.)
}

fn draw_fortytwo<D>(
    display: &mut D,
    vert: [Vec3; 42],
    frame: u32,
    d: f32,
    scale: f32,
    x_offset: i32,
    y_offset: i32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let faces = [
        (0, 7, 8),
        (0, 1, 7),
        (1, 2, 7),
        (2, 3, 7),
        (5, 6, 7),
        (3, 4, 5),
        (9, 16, 17),
        (9, 10, 16),
        (10, 11, 16),
        (11, 12, 16),
        (14, 15, 16),
        (12, 13, 14),
        (0, 1, 9),
        (9, 10, 1),
        (10, 11, 1),
        (1, 2, 11),
        (14, 15, 6),
        (5, 6, 14),
        (12, 13, 3),
        (3, 4, 13),
        (7, 8, 16),
        (16, 17, 8),
        (18, 19, 20),
        (20, 21, 29),
        (21, 28, 29),
        (21, 22, 27),
        (21, 27, 28),
        (22, 23, 27),
        (23, 26, 27),
        (24, 25, 26),
        (30, 31, 32),
        (32, 33, 41),
        (33, 40, 41),
        (33, 34, 39),
        (33, 39, 40),
        (34, 35, 39),
        (35, 38, 39),
        (36, 37, 38),
        (18, 19, 30),
        (30, 31, 19),
        (22, 23, 34),
        (34, 35, 23),
        (21, 22, 33),
        (33, 34, 22),
        (27, 28, 40),
        (39, 40, 27),
        (28, 29, 41),
        (40, 41, 28),
        (24, 25, 37),
        (36, 37, 24),
        (18, 20, 30),
        (30, 32, 20),
    ];

    // let color = Rgb565::new((frame / 7 % 255) as u8, (frame / 9 % 255) as u8, (frame / 12 % 255) as u8);
    let wheel = ColorWheel::new(1., 1.);
    let color = wheel.get_color_at_hue((frame / 2) as f32 % 360.);
    for (i, j, k) in faces {
        // let color = Rgb565::new(((vert[i].z + 2.) / 4. * 256.) as u8, 0, 0);
        let p1 = project(vert[i], d, scale);
        let p2 = project(vert[j], d, scale);
        let p3 = project(vert[k], d, scale);
        Triangle::new(
            Point::new(p1.x as i32 + x_offset, p1.y as i32 + y_offset),
            Point::new(p2.x as i32 + x_offset, p2.y as i32 + y_offset),
            Point::new(p3.x as i32 + x_offset, p3.y as i32 + y_offset),
        )
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(display)?
    }

    Ok(())
}

#[derive(Copy, Clone)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

fn rotate_y(v: &mut Vec3, angle: f32) {
    let cos_a = libm::cosf(angle);
    let sin_a = libm::sinf(angle);
    let x = v.x;
    let z = v.z;

    v.x = x * cos_a + z * sin_a;
    v.z = -x * sin_a + z * cos_a;
}

pub fn draw_animation_frame<D>(display: &mut D, frame: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    display.clear(Rgb565::WHITE)?;
    for y in 0..128 {
        Rectangle::new(Point::new(0, y), Size::new(128, 1))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(display)?;
    }

    let t = frame as f32 * 0.03;

    let mut vertices: [Vec3; 42] = [
        Vec3::new(-4., -10., -2.),
        Vec3::new(-14., 0., -2.),
        Vec3::new(-14., 5., -2.),
        Vec3::new(-4., 5., -2.),
        Vec3::new(-4., 10., -2.),
        Vec3::new(1., 10., -2.),
        Vec3::new(1., 0., -2.),
        Vec3::new(-9., 0., -2.),
        Vec3::new(1., -10., -2.),
        Vec3::new(-4., -10., 2.),
        Vec3::new(-14., 0., 2.),
        Vec3::new(-14., 5., 2.),
        Vec3::new(-4., 5., 2.),
        Vec3::new(-4., 10., 2.),
        Vec3::new(1., 10., 2.),
        Vec3::new(1., 0., 2.),
        Vec3::new(-9., 0., 2.),
        Vec3::new(1., -10., 2.),
        Vec3::new(4., -10., -2.),
        Vec3::new(4., -5., -2.),
        Vec3::new(9., -10., -2.),
        Vec3::new(9., -5., -2.),
        Vec3::new(4., 0., -2.),
        Vec3::new(4., 5., -2.),
        Vec3::new(14., 5., -2.),
        Vec3::new(14., 0., -2.),
        Vec3::new(9., 5., -2.),
        Vec3::new(9., 0., -2.),
        Vec3::new(14., -5., -2.),
        Vec3::new(14., -10., -2.),
        Vec3::new(4., -10., 2.),
        Vec3::new(4., -5., 2.),
        Vec3::new(9., -10., 2.),
        Vec3::new(9., -5., 2.),
        Vec3::new(4., 0., 2.),
        Vec3::new(4., 5., 2.),
        Vec3::new(14., 5., 2.),
        Vec3::new(14., 0., 2.),
        Vec3::new(9., 5., 2.),
        Vec3::new(9., 0., 2.),
        Vec3::new(14., -5., 2.),
        Vec3::new(14., -10., 2.),
    ];

    for v in &mut vertices {
        rotate_y(v, t - libm::sinf(t));
    }

    draw_fortytwo(display, vertices, frame, 50., 192., 64, 64)?;
    Ok(())
}
