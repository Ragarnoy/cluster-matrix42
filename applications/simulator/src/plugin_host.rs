//! Plugin host for the simulator
//!
//! This module provides a native plugin runtime that can run plugins
//! compiled for the host platform, bridging between the plugin API
//! and the embedded-graphics simulator.

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::SimulatorDisplay;
use plugin_api::*;
use std::cell::RefCell;
use std::time::Instant;

// Thread-local storage for the runtime pointer (used by C-style callbacks)
thread_local! {
    static RUNTIME_PTR: RefCell<Option<*mut SimulatorPluginRuntime>> = const { RefCell::new(None) };
}

/// Trait for native plugins that can be statically linked
pub trait Plugin: Send {
    /// Create a new instance of the plugin
    fn new() -> Self
    where
        Self: Sized;

    /// Initialize the plugin with the API
    fn init(&mut self, api: &mut PluginAPI) -> i32;

    /// Update the plugin state (called every frame)
    fn update(&mut self, api: &mut PluginAPI, inputs: Inputs);

    /// Clean up plugin resources
    fn cleanup(&mut self);

    /// Get the plugin name
    fn name(&self) -> &'static str;
}

/// Plugin runtime for the simulator
pub struct SimulatorPluginRuntime {
    framebuffer: FrameBuffer,
    graphics_ctx: GraphicsContext,
    system_ctx: SystemContext,
    api: PluginAPI,
    start_time: Instant,
    rng_state: u32,
}

impl SimulatorPluginRuntime {
    /// Create a new simulator plugin runtime
    pub fn new() -> Self {
        let mut runtime = Self {
            framebuffer: FrameBuffer {
                pixels: [0; FRAMEBUFFER_SIZE],
                width: DISPLAY_WIDTH as u32,
                height: DISPLAY_HEIGHT as u32,
                frame_counter: 0,
            },
            graphics_ctx: GraphicsContext {
                set_pixel_fn: gfx_set_pixel,
                get_pixel_fn: gfx_get_pixel,
                clear_fn: gfx_clear,
                fill_rect_fn: gfx_fill_rect,
                draw_line_fn: gfx_draw_line,
                draw_circle_fn: gfx_draw_circle,
                blit_fn: gfx_blit,
            },
            system_ctx: SystemContext {
                random_fn: sys_random,
                millis_fn: sys_millis,
                rgb_fn: sys_rgb,
                color_red: 0xF800,
                color_green: 0x07E0,
                color_blue: 0x001F,
                color_white: 0xFFFF,
                color_black: 0x0000,
                color_yellow: 0xFFE0,
                color_cyan: 0x07FF,
                color_magenta: 0xF81F,
            },
            api: PluginAPI {
                framebuffer: std::ptr::null_mut(),
                gfx: std::ptr::null(),
                sys: std::ptr::null(),
            },
            start_time: Instant::now(),
            rng_state: 0xDEADBEEF,
        };

        // Set up API pointers
        runtime.api.framebuffer = &mut runtime.framebuffer as *mut _;
        runtime.api.gfx = &runtime.graphics_ctx as *const _;
        runtime.api.sys = &runtime.system_ctx as *const _;

        runtime
    }

    /// Update API pointers to current memory location
    /// Required because the struct may have moved since new()
    fn refresh_api_pointers(&mut self) {
        self.api.framebuffer = &mut self.framebuffer as *mut _;
        self.api.gfx = &self.graphics_ctx as *const _;
        self.api.sys = &self.system_ctx as *const _;
    }

    /// Initialize a plugin
    pub fn init_plugin<P: Plugin>(&mut self, plugin: &mut P) -> i32 {
        // Refresh API pointers in case struct was moved
        self.refresh_api_pointers();

        // Set up thread-local runtime pointer for callbacks
        RUNTIME_PTR.with(|ptr| {
            *ptr.borrow_mut() = Some(self as *mut _);
        });

        plugin.init(&mut self.api)
    }

    /// Run one update cycle
    pub fn update<P: Plugin>(&mut self, plugin: &mut P, inputs: u32) {
        // Refresh API pointers in case struct was moved
        self.refresh_api_pointers();

        // Ensure runtime pointer is set
        RUNTIME_PTR.with(|ptr| {
            *ptr.borrow_mut() = Some(self as *mut _);
        });

        plugin.update(&mut self.api, Inputs::from_raw(inputs));
        self.framebuffer.frame_counter = self.framebuffer.frame_counter.wrapping_add(1);
    }

    /// Get elapsed milliseconds since runtime creation
    pub fn millis(&self) -> u32 {
        self.start_time.elapsed().as_millis() as u32
    }

    /// Get a random number using xorshift
    pub fn random(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        self.rng_state
    }

    /// Copy the framebuffer to a simulator display
    pub fn render_to_display(&self, display: &mut SimulatorDisplay<Rgb565>) {
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let color = self.framebuffer.pixels[y * DISPLAY_WIDTH + x];
                let point = Point::new(x as i32, y as i32);
                let rgb = Rgb565::from(RawU16::new(color));
                Pixel(point, rgb).draw(display).ok();
            }
        }
    }

    /// Get reference to framebuffer
    pub fn framebuffer(&self) -> &FrameBuffer {
        &self.framebuffer
    }

    /// Get mutable reference to framebuffer
    pub fn framebuffer_mut(&mut self) -> &mut FrameBuffer {
        &mut self.framebuffer
    }
}

impl Default for SimulatorPluginRuntime {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Internal graphics functions
// ============================================================================

fn with_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&mut SimulatorPluginRuntime) -> R,
    R: Default,
{
    RUNTIME_PTR.with(|ptr| {
        if let Some(runtime_ptr) = *ptr.borrow() {
            unsafe { f(&mut *runtime_ptr) }
        } else {
            R::default()
        }
    })
}

fn set_pixel_internal(runtime: &mut SimulatorPluginRuntime, x: i32, y: i32, color: u16) {
    if x >= 0 && x < DISPLAY_WIDTH as i32 && y >= 0 && y < DISPLAY_HEIGHT as i32 {
        let idx = (y as usize) * DISPLAY_WIDTH + (x as usize);
        runtime.framebuffer.pixels[idx] = color;
    }
}

fn get_pixel_internal(runtime: &SimulatorPluginRuntime, x: i32, y: i32) -> u16 {
    if x >= 0 && x < DISPLAY_WIDTH as i32 && y >= 0 && y < DISPLAY_HEIGHT as i32 {
        let idx = (y as usize) * DISPLAY_WIDTH + (x as usize);
        runtime.framebuffer.pixels[idx]
    } else {
        0
    }
}

fn clear_internal(runtime: &mut SimulatorPluginRuntime, color: u16) {
    runtime.framebuffer.pixels.fill(color);
}

fn fill_rect_internal(
    runtime: &mut SimulatorPluginRuntime,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: u16,
) {
    let x_start = x.max(0) as usize;
    let y_start = y.max(0) as usize;
    let x_end = ((x + w).min(DISPLAY_WIDTH as i32) as usize).min(DISPLAY_WIDTH);
    let y_end = ((y + h).min(DISPLAY_HEIGHT as i32) as usize).min(DISPLAY_HEIGHT);

    if x_start >= x_end || y_start >= y_end {
        return;
    }

    for py in y_start..y_end {
        for px in x_start..x_end {
            runtime.framebuffer.pixels[py * DISPLAY_WIDTH + px] = color;
        }
    }
}

fn draw_line_internal(
    runtime: &mut SimulatorPluginRuntime,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: u16,
) {
    // Bresenham's line algorithm
    let mut x = x0;
    let mut y = y0;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        set_pixel_internal(runtime, x, y, color);

        if x == x1 && y == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn draw_circle_internal(
    runtime: &mut SimulatorPluginRuntime,
    cx: i32,
    cy: i32,
    radius: i32,
    color: u16,
) {
    if radius < 0 {
        return;
    }

    // Midpoint circle algorithm
    let mut x = radius;
    let mut y = 0;
    let mut decision = 1 - radius;

    while x >= y {
        set_pixel_internal(runtime, cx + x, cy + y, color);
        set_pixel_internal(runtime, cx - x, cy + y, color);
        set_pixel_internal(runtime, cx + x, cy - y, color);
        set_pixel_internal(runtime, cx - x, cy - y, color);
        set_pixel_internal(runtime, cx + y, cy + x, color);
        set_pixel_internal(runtime, cx - y, cy + x, color);
        set_pixel_internal(runtime, cx + y, cy - x, color);
        set_pixel_internal(runtime, cx - y, cy - x, color);

        y += 1;

        if decision <= 0 {
            decision += 2 * y + 1;
        } else {
            x -= 1;
            decision += 2 * (y - x) + 1;
        }
    }
}

fn blit_internal(
    runtime: &mut SimulatorPluginRuntime,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    data: *const u16,
) {
    if data.is_null() || w <= 0 || h <= 0 || w > 1024 || h > 1024 {
        return;
    }

    unsafe {
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx;
                let py = y + dy;

                if px >= 0 && px < DISPLAY_WIDTH as i32 && py >= 0 && py < DISPLAY_HEIGHT as i32 {
                    let src_idx = (dy * w + dx) as usize;
                    let dst_idx = (py as usize) * DISPLAY_WIDTH + (px as usize);
                    runtime.framebuffer.pixels[dst_idx] = *data.add(src_idx);
                }
            }
        }
    }
}

// ============================================================================
// C-style callback functions for the plugin API
// ============================================================================

unsafe extern "C" fn gfx_set_pixel(x: i32, y: i32, color: u16) {
    with_runtime(|runtime| set_pixel_internal(runtime, x, y, color));
}

unsafe extern "C" fn gfx_get_pixel(x: i32, y: i32) -> u16 {
    with_runtime(|runtime| get_pixel_internal(runtime, x, y))
}

unsafe extern "C" fn gfx_clear(color: u16) {
    with_runtime(|runtime| clear_internal(runtime, color));
}

unsafe extern "C" fn gfx_fill_rect(x: i32, y: i32, w: i32, h: i32, color: u16) {
    with_runtime(|runtime| fill_rect_internal(runtime, x, y, w, h, color));
}

unsafe extern "C" fn gfx_draw_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u16) {
    with_runtime(|runtime| draw_line_internal(runtime, x0, y0, x1, y1, color));
}

unsafe extern "C" fn gfx_draw_circle(cx: i32, cy: i32, radius: i32, color: u16) {
    with_runtime(|runtime| draw_circle_internal(runtime, cx, cy, radius, color));
}

unsafe extern "C" fn gfx_blit(x: i32, y: i32, w: i32, h: i32, data: *const u16) {
    with_runtime(|runtime| blit_internal(runtime, x, y, w, h, data));
}

unsafe extern "C" fn sys_random() -> u32 {
    with_runtime(|runtime| runtime.random())
}

unsafe extern "C" fn sys_millis() -> u32 {
    with_runtime(|runtime| runtime.millis())
}

unsafe extern "C" fn sys_rgb(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | ((b as u16 & 0xF8) >> 3)
}
