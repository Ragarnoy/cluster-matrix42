#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::cell::UnsafeCell;

/// Display dimensions
pub const DISPLAY_WIDTH: usize = 128;
pub const DISPLAY_HEIGHT: usize = 128;
pub const FRAMEBUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

/// Plugin magic number and version
pub const PLUGIN_MAGIC: u32 = 0x504C5547; // "PLUG" in hex
pub const PLUGIN_API_VERSION: u32 = 1;

// ============================================================================
// Core C-ABI Structures
// ============================================================================

/// Main API structure passed to plugins.
///
/// This struct contains raw pointers to the runtime-provided contexts.
/// The pointers are guaranteed valid for the duration of `init`, `update`,
/// and `cleanup` calls when used through the plugin system.
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
    /// Display width
    pub width: u32,
    /// Display height
    pub height: u32,
    /// Current frame counter
    pub frame_counter: u32,
}

/// Graphics helper functions (C function pointers)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GraphicsContext {
    pub set_pixel_fn: unsafe extern "C" fn(x: i32, y: i32, color: u16),
    pub get_pixel_fn: unsafe extern "C" fn(x: i32, y: i32) -> u16,
    pub clear_fn: unsafe extern "C" fn(color: u16),
    pub fill_rect_fn: unsafe extern "C" fn(x: i32, y: i32, w: i32, h: i32, color: u16),
    pub draw_line_fn: unsafe extern "C" fn(x0: i32, y0: i32, x1: i32, y1: i32, color: u16),
    pub draw_circle_fn: unsafe extern "C" fn(cx: i32, cy: i32, radius: i32, color: u16),
    pub blit_fn: unsafe extern "C" fn(x: i32, y: i32, w: i32, h: i32, data: *const u16),
}

/// System utilities (C function pointers and color constants)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SystemContext {
    pub random_fn: unsafe extern "C" fn() -> u32,
    pub millis_fn: unsafe extern "C" fn() -> u32,
    pub rgb_fn: unsafe extern "C" fn(r: u8, g: u8, b: u8) -> u16,
    pub color_red: u16,
    pub color_green: u16,
    pub color_blue: u16,
    pub color_white: u16,
    pub color_black: u16,
    pub color_yellow: u16,
    pub color_cyan: u16,
    pub color_magenta: u16,
}

/// Plugin header placed at start of binary
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

// ============================================================================
// Input Constants (C-compatible)
// ============================================================================

pub const INPUT_UP: u32 = 1 << 0;
pub const INPUT_DOWN: u32 = 1 << 1;
pub const INPUT_LEFT: u32 = 1 << 2;
pub const INPUT_RIGHT: u32 = 1 << 3;
pub const INPUT_A: u32 = 1 << 4;
pub const INPUT_B: u32 = 1 << 5;
pub const INPUT_START: u32 = 1 << 6;
pub const INPUT_SELECT: u32 = 1 << 7;

// ============================================================================
// Rust-Safe Wrappers
// ============================================================================

/// Type-safe input wrapper for Rust plugins
#[derive(Clone, Copy, Debug, Default)]
pub struct Inputs(u32);

impl Inputs {
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn up(self) -> bool {
        self.0 & INPUT_UP != 0
    }

    #[must_use]
    pub const fn down(self) -> bool {
        self.0 & INPUT_DOWN != 0
    }

    #[must_use]
    pub const fn left(self) -> bool {
        self.0 & INPUT_LEFT != 0
    }

    #[must_use]
    pub const fn right(self) -> bool {
        self.0 & INPUT_RIGHT != 0
    }

    #[must_use]
    pub const fn a(self) -> bool {
        self.0 & INPUT_A != 0
    }

    #[must_use]
    pub const fn b(self) -> bool {
        self.0 & INPUT_B != 0
    }

    #[must_use]
    pub const fn start(self) -> bool {
        self.0 & INPUT_START != 0
    }

    #[must_use]
    pub const fn select(self) -> bool {
        self.0 & INPUT_SELECT != 0
    }
}

impl PluginAPI {
    /// Get mutable reference to framebuffer.
    ///
    /// # Safety
    /// The caller must ensure this is only called during plugin callbacks
    /// (init, update) when the pointer is valid.
    #[must_use]
    pub fn framebuffer(&mut self) -> &mut FrameBuffer {
        // SAFETY: Plugin runtime guarantees pointer validity during callbacks
        unsafe { &mut *self.framebuffer }
    }

    /// Get reference to graphics context.
    #[must_use]
    pub fn gfx(&self) -> &GraphicsContext {
        // SAFETY: Plugin runtime guarantees pointer validity during callbacks
        unsafe { &*self.gfx }
    }

    /// Get reference to system context.
    #[must_use]
    pub fn sys(&self) -> &SystemContext {
        // SAFETY: Plugin runtime guarantees pointer validity during callbacks
        unsafe { &*self.sys }
    }
}

impl GraphicsContext {
    pub fn set_pixel(&self, x: i32, y: i32, color: u16) {
        // SAFETY: Function pointer set by runtime, always valid during plugin execution
        unsafe { (self.set_pixel_fn)(x, y, color) }
    }

    #[must_use]
    pub fn get_pixel(&self, x: i32, y: i32) -> u16 {
        unsafe { (self.get_pixel_fn)(x, y) }
    }

    pub fn clear(&self, color: u16) {
        unsafe { (self.clear_fn)(color) }
    }

    pub fn fill_rect(&self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        unsafe { (self.fill_rect_fn)(x, y, w, h, color) }
    }

    pub fn draw_line(&self, x0: i32, y0: i32, x1: i32, y1: i32, color: u16) {
        unsafe { (self.draw_line_fn)(x0, y0, x1, y1, color) }
    }

    pub fn draw_circle(&self, cx: i32, cy: i32, radius: i32, color: u16) {
        unsafe { (self.draw_circle_fn)(cx, cy, radius, color) }
    }

    pub fn blit(&self, x: i32, y: i32, w: i32, h: i32, data: &[u16]) {
        unsafe { (self.blit_fn)(x, y, w, h, data.as_ptr()) }
    }
}

impl SystemContext {
    #[must_use]
    pub fn random(&self) -> u32 {
        unsafe { (self.random_fn)() }
    }

    #[must_use]
    pub fn millis(&self) -> u32 {
        unsafe { (self.millis_fn)() }
    }

    #[must_use]
    pub fn rgb(&self, r: u8, g: u8, b: u8) -> u16 {
        unsafe { (self.rgb_fn)(r, g, b) }
    }

    #[must_use]
    pub const fn red(&self) -> u16 {
        self.color_red
    }
    #[must_use]
    pub const fn green(&self) -> u16 {
        self.color_green
    }
    #[must_use]
    pub const fn blue(&self) -> u16 {
        self.color_blue
    }
    #[must_use]
    pub const fn white(&self) -> u16 {
        self.color_white
    }
    #[must_use]
    pub const fn black(&self) -> u16 {
        self.color_black
    }
    #[must_use]
    pub const fn yellow(&self) -> u16 {
        self.color_yellow
    }
    #[must_use]
    pub const fn cyan(&self) -> u16 {
        self.color_cyan
    }
    #[must_use]
    pub const fn magenta(&self) -> u16 {
        self.color_magenta
    }
}

impl FrameBuffer {
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub const fn frame_count(&self) -> u32 {
        self.frame_counter
    }

    /// Set pixel with bounds checking (silent no-op if out of bounds)
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u16) {
        if x < DISPLAY_WIDTH && y < DISPLAY_HEIGHT {
            self.pixels[y * DISPLAY_WIDTH + x] = color;
        }
    }

    /// Get pixel with bounds checking
    #[must_use]
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<u16> {
        if x < DISPLAY_WIDTH && y < DISPLAY_HEIGHT {
            Some(self.pixels[y * DISPLAY_WIDTH + x])
        } else {
            None
        }
    }

    /// Direct pixel slice access
    #[must_use]
    pub fn pixels(&self) -> &[u16; FRAMEBUFFER_SIZE] {
        &self.pixels
    }

    /// Direct mutable pixel slice access
    #[must_use]
    pub fn pixels_mut(&mut self) -> &mut [u16; FRAMEBUFFER_SIZE] {
        &mut self.pixels
    }
}

// ============================================================================
// Plugin Instance Storage (for macro)
// ============================================================================

/// Thread-safe plugin instance storage.
/// Uses UnsafeCell since plugins run single-threaded on embedded.
#[doc(hidden)]
pub struct PluginInstance<T>(UnsafeCell<Option<T>>);

// SAFETY: Plugins are single-threaded on embedded targets
unsafe impl<T> Sync for PluginInstance<T> {}

impl<T> Default for PluginInstance<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PluginInstance<T> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(None))
    }

    /// # Safety
    /// Must only be called from single-threaded plugin context
    pub unsafe fn set(&self, value: T) {
        unsafe { *self.0.get() = Some(value) }
    }

    /// # Safety
    /// Must only be called from single-threaded plugin context.
    /// This uses UnsafeCell for interior mutability - returning &mut from &self is intentional.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut(&self) -> Option<&mut T> {
        unsafe { (*self.0.get()).as_mut() }
    }

    /// # Safety
    /// Must only be called from single-threaded plugin context
    pub unsafe fn take(&self) -> Option<T> {
        unsafe { (*self.0.get()).take() }
    }
}

// ============================================================================
// Plugin Trait
// ============================================================================

/// Trait that all Rust plugins must implement.
///
/// This provides compile-time checking that your plugin has the correct interface.
/// Use the `plugin_main!` macro to generate the C-ABI glue code.
pub trait PluginImpl {
    /// Create a new instance of the plugin
    fn new() -> Self
    where
        Self: Sized;

    /// Initialize the plugin. Return 0 for success, non-zero for failure.
    fn init(&mut self, api: &mut PluginAPI) -> i32;

    /// Update the plugin state (called every frame at ~60fps)
    fn update(&mut self, api: &mut PluginAPI, inputs: Inputs);

    /// Clean up any resources when the plugin is unloaded
    fn cleanup(&mut self);
}

// ============================================================================
// Plugin Macro
// ============================================================================

/// Macro to create a Rust plugin with minimal boilerplate.
///
/// Your struct must implement the `PluginImpl` trait.
///
/// # Example
/// ```ignore
/// use plugin_api::prelude::*;
///
/// struct MyPlugin { counter: u32 }
///
/// impl PluginImpl for MyPlugin {
///     fn new() -> Self { Self { counter: 0 } }
///     fn init(&mut self, _api: &mut PluginAPI) -> i32 { 0 }
///     fn update(&mut self, api: &mut PluginAPI, inputs: Inputs) {
///         self.counter += 1;
///         api.gfx().clear(api.sys().black());
///     }
///     fn cleanup(&mut self) {}
/// }
///
/// plugin_main!(MyPlugin, "my_plugin");
/// ```
#[macro_export]
macro_rules! plugin_main {
    ($plugin_type:ty, $name:expr) => {
        // Compile-time check that the type implements PluginImpl
        const _: () = {
            fn _assert_plugin_impl<T: $crate::PluginImpl>() {}
            fn _check() {
                _assert_plugin_impl::<$plugin_type>();
            }
        };
        static PLUGIN_INSTANCE: $crate::PluginInstance<$plugin_type> =
            $crate::PluginInstance::new();

        #[unsafe(link_section = ".plugin_header")]
        #[used]
        #[unsafe(no_mangle)]
        pub static PLUGIN_HEADER: $crate::PluginHeader = $crate::PluginHeader {
            magic: $crate::PLUGIN_MAGIC,
            api_version: $crate::PLUGIN_API_VERSION,
            name: {
                let mut name_arr = [0u8; 32];
                let name_bytes = $name.as_bytes();
                let len = if name_bytes.len() < 32 {
                    name_bytes.len()
                } else {
                    31
                };
                let mut i = 0;
                while i < len {
                    name_arr[i] = name_bytes[i];
                    i += 1;
                }
                name_arr
            },
            init: __plugin_init,
            update: __plugin_update,
            cleanup: __plugin_cleanup,
        };

        #[unsafe(no_mangle)]
        extern "C" fn __plugin_init(api: *const $crate::PluginAPI) -> i32 {
            // SAFETY: API pointer valid during callback, single-threaded execution
            unsafe {
                let api_mut = &mut *(api as *mut $crate::PluginAPI);
                let mut plugin = <$plugin_type>::new();
                let result = plugin.init(api_mut);
                PLUGIN_INSTANCE.set(plugin);
                result
            }
        }

        #[unsafe(no_mangle)]
        extern "C" fn __plugin_update(api: *const $crate::PluginAPI, inputs: u32) {
            // SAFETY: API pointer valid during callback, single-threaded execution
            unsafe {
                let api_mut = &mut *(api as *mut $crate::PluginAPI);
                let inputs = $crate::Inputs::from_raw(inputs);
                if let Some(plugin) = PLUGIN_INSTANCE.get_mut() {
                    plugin.update(api_mut, inputs);
                }
            }
        }

        #[unsafe(no_mangle)]
        extern "C" fn __plugin_cleanup() {
            // SAFETY: Single-threaded execution
            unsafe {
                if let Some(mut plugin) = PLUGIN_INSTANCE.take() {
                    plugin.cleanup();
                }
            }
        }
    };
}

// ============================================================================
// Prelude
// ============================================================================

pub mod prelude {
    pub use crate::{
        DISPLAY_HEIGHT, DISPLAY_WIDTH, FRAMEBUFFER_SIZE, FrameBuffer, GraphicsContext, INPUT_A,
        INPUT_B, INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_SELECT, INPUT_START, INPUT_UP, Inputs,
        PluginAPI, PluginImpl, SystemContext, plugin_main,
    };
}
