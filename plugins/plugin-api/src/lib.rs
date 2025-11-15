#![no_std]

/// Display dimensions
pub const DISPLAY_WIDTH: usize = 128;
pub const DISPLAY_HEIGHT: usize = 128;
pub const FRAMEBUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

/// Plugin magic number and version
pub const PLUGIN_MAGIC: u32 = 0x504C5547; // "PLUG" in hex
pub const PLUGIN_API_VERSION: u32 = 1;

/// Main API structure passed to plugins
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginAPI {
    /// Direct framebuffer access
    pub framebuffer: *mut FrameBuffer,
    /// Graphics context with drawing helpers
    pub gfx: *const GraphicsContext,
    /// System utilities
    pub sys: *const SystemContext,
}

/// Direct framebuffer access structure
#[repr(C)]
pub struct FrameBuffer {
    /// Raw pixel data in RGB565 format
    pub pixels: [u16; FRAMEBUFFER_SIZE],
    /// Display width (always 128)
    pub width: u32,
    /// Display height (always 128)
    pub height: u32,
    /// Current frame counter
    pub frame_counter: u32,
}

/// Graphics helper functions
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GraphicsContext {
    pub set_pixel: unsafe extern "C" fn(x: i32, y: i32, color: u16),
    pub get_pixel: unsafe extern "C" fn(x: i32, y: i32) -> u16,
    pub clear: unsafe extern "C" fn(color: u16),
    pub fill_rect: unsafe extern "C" fn(x: i32, y: i32, w: i32, h: i32, color: u16),
    pub draw_line: unsafe extern "C" fn(x0: i32, y0: i32, x1: i32, y1: i32, color: u16),
    pub draw_circle: unsafe extern "C" fn(cx: i32, cy: i32, radius: i32, color: u16),
    pub blit: unsafe extern "C" fn(x: i32, y: i32, w: i32, h: i32, data: *const u16),
}

/// System utilities
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SystemContext {
    pub random: unsafe extern "C" fn() -> u32,
    pub millis: unsafe extern "C" fn() -> u32,
    pub rgb: unsafe extern "C" fn(r: u8, g: u8, b: u8) -> u16,
    pub color_red: u16,
    pub color_green: u16,
    pub color_blue: u16,
    pub color_white: u16,
    pub color_black: u16,
    pub color_yellow: u16,
    pub color_cyan: u16,
    pub color_magenta: u16,
}

/// Plugin header
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginHeader {
    pub magic: u32,
    pub api_version: u32,
    pub name: [u8; 32],
    pub init: unsafe extern "C" fn(api: *const PluginAPI) -> i32,
    pub update: unsafe extern "C" fn(api: *const PluginAPI, inputs: u32),
    pub cleanup: unsafe extern "C" fn(),
}

// Constants for C compatibility
pub const INPUT_UP: u32 = 1 << 0;
pub const INPUT_DOWN: u32 = 1 << 1;
pub const INPUT_LEFT: u32 = 1 << 2;
pub const INPUT_RIGHT: u32 = 1 << 3;
pub const INPUT_A: u32 = 1 << 4;
pub const INPUT_B: u32 = 1 << 5;
pub const INPUT_START: u32 = 1 << 6;
pub const INPUT_SELECT: u32 = 1 << 7;
