//! Display memory management with double buffering

use crate::config::*;
use crate::lut::GAMMA8;
use embedded_graphics_core::pixelcolor::Rgb565;

/// Double-buffered framebuffer with hardware-optimized layout
///
/// The memory layout is optimized for the PIO+DMA scanning pattern:
/// - Data is arranged as \[row]\[bit_plane]\[column]
/// - Each byte contains packed RGB data for 2 pixels (top/bottom half)
/// - Double buffering allows drawing while previous frame displays
pub struct DisplayMemory {
    /// Primary framebuffer
    pub fb0: [u8; FRAME_SIZE],

    /// Secondary framebuffer  
    pub fb1: [u8; FRAME_SIZE],

    /// Pointer to the currently active buffer (read by DMA)
    pub fb_ptr: *mut u8,

    /// Binary Color Modulation delay values
    pub delays: [u32; COLOR_BITS],

    /// Pointer to delay array (read by DMA)
    pub delay_ptr: *mut u32,

    /// Which buffer is currently active (false = fb0, true = fb1)
    current_buffer: bool,
}

impl Default for DisplayMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayMemory {
    /// Create a new display memory instance
    pub const fn new() -> Self {
        let fb0 = [0u8; FRAME_SIZE];
        let fb1 = [0u8; FRAME_SIZE];
        let delays = compute_bcm_delays();

        Self {
            fb_ptr: core::ptr::null_mut(), // Will be initialized properly later
            fb0,
            fb1,
            delays,
            delay_ptr: core::ptr::null_mut(), // Will be initialized properly later
            current_buffer: false,
        }
    }

    /// Commit the drawn buffer and make it active for display
    ///
    /// This swaps the buffers so the newly drawn frame becomes visible
    /// while the old frame buffer becomes available for drawing
    pub fn commit(&mut self) {
        // Switch buffers
        self.current_buffer = !self.current_buffer;

        // Update pointer for DMA to read from newly committed buffer
        self.fb_ptr = if self.current_buffer {
            self.fb1.as_mut_ptr()
        } else {
            self.fb0.as_mut_ptr()
        };

        // Clear the new draw buffer for next frame
        self.get_draw_buffer().fill(0);
    }

    /// Get the currently inactive buffer for drawing
    fn get_draw_buffer(&mut self) -> &mut [u8; FRAME_SIZE] {
        if self.current_buffer {
            &mut self.fb0
        } else {
            &mut self.fb1
        }
    }

    /// Set a pixel in the draw buffer
    ///
    /// # Arguments
    /// * `x` - X coordinate (0 to DISPLAY_WIDTH-1)
    /// * `y` - Y coordinate (0 to DISPLAY_HEIGHT-1)
    /// * `color` - RGB565 color value
    /// * `brightness` - Global brightness multiplier (0-255)
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb565, brightness: u8) {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return;
        }

        // Half of the screen
        let h = y > (DISPLAY_HEIGHT / 2) - 1;
        let shift = if h { 3 } else { 0 };

        // CRITICAL: Original color channel mapping (swapped!)
        let mut c_r: u16;
        let mut c_b: u16;
        let mut c_g: u16;

        #[cfg(feature = "color_rgb")]
        {
            c_r = (((color.r() << 3) as f32) * (brightness as f32 / 255f32)) as u16;
            c_g = (((color.g() << 2) as f32) * (brightness as f32 / 255f32)) as u16;
            c_b = (((color.b() << 3) as f32) * (brightness as f32 / 255f32)) as u16;
        }

        #[cfg(feature = "color_gbr")]
        {
            c_g = (((color.r() << 3) as f32) * (brightness as f32 / 255f32)) as u16;
            c_b = (((color.g() << 2) as f32) * (brightness as f32 / 255f32)) as u16;
            c_r = (((color.b() << 3) as f32) * (brightness as f32 / 255f32)) as u16;
        }

        let base_idx = x + ((y % (DISPLAY_HEIGHT / 2)) * DISPLAY_WIDTH * COLOR_BITS);

        c_r = GAMMA8[c_r as usize] as u16;
        c_g = GAMMA8[c_g as usize] as u16;
        c_b = GAMMA8[c_b as usize] as u16;

        for b in 0..COLOR_BITS {
            // Extract the n-th bit of each component of the color and pack them
            let cr = c_r >> b & 0b1;
            let cg = c_g >> b & 0b1;
            let cb = c_b >> b & 0b1;
            let packed_rgb = (cb << 2 | cg << 1 | cr) as u8;
            let idx = base_idx + b * DISPLAY_WIDTH;

            // CRITICAL: Original buffer selection logic (inverted!)
            if self.fb_ptr == self.fb0.as_mut_ptr() {
                self.fb1[idx] &= !(0b111 << shift);
                self.fb1[idx] |= packed_rgb << shift;
            } else {
                self.fb0[idx] &= !(0b111 << shift);
                self.fb0[idx] |= packed_rgb << shift;
            }
        }
    }

    /// Clear the draw buffer
    pub fn clear(&mut self) {
        self.get_draw_buffer().fill(0);
    }

    /// Get pointer to active framebuffer (for DMA)
    pub fn get_active_buffer_ptr(&self) -> *mut u8 {
        self.fb_ptr
    }

    /// Get pointer to delay array (for DMA)
    pub fn get_delay_ptr(&self) -> *mut u32 {
        self.delay_ptr
    }

    /// Get pointer to the framebuffer pointer (for DMA chaining)
    pub fn get_fb_ptr_addr(&self) -> *const *mut u8 {
        &self.fb_ptr as *const _
    }

    /// Get pointer to the delay pointer (for DMA chaining)
    pub fn get_delay_ptr_addr(&self) -> *const *mut u32 {
        &self.delay_ptr as *const _
    }
}

// Safety: DisplayMemory contains only plain data and atomic operations
unsafe impl Send for DisplayMemory {}
unsafe impl Sync for DisplayMemory {}
