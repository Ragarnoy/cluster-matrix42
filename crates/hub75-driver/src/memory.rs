//! Display memory management with double buffering

use crate::config::*;
use crate::lut::GAMMA8;
use core::mem::MaybeUninit;
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::RgbColor;

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
    /// Create a new display memory instance using MaybeUninit for safe initialization
    pub fn new() -> Self {
        unsafe {
            let mut memory = MaybeUninit::<Self>::uninit();
            let ptr = memory.as_mut_ptr();

            // Initialize framebuffers to zero
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!((*ptr).fb0).cast::<u8>(),
                0,
                FRAME_SIZE,
            );
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!((*ptr).fb1).cast::<u8>(),
                0,
                FRAME_SIZE,
            );

            // Initialize delays
            core::ptr::write(core::ptr::addr_of_mut!((*ptr).delays), compute_bcm_delays());

            // Initialize other fields
            core::ptr::write(
                core::ptr::addr_of_mut!((*ptr).fb_ptr),
                core::ptr::null_mut(),
            );
            core::ptr::write(
                core::ptr::addr_of_mut!((*ptr).delay_ptr),
                core::ptr::null_mut(),
            );
            core::ptr::write(core::ptr::addr_of_mut!((*ptr).current_buffer), false);

            memory.assume_init()
        }
    }

    /// Initialize pointers after creation
    pub const fn init_pointers(&mut self) {
        self.fb_ptr = self.fb0.as_mut_ptr();
        self.delay_ptr = self.delays.as_mut_ptr();
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
    const fn get_draw_buffer(&mut self) -> &mut [u8; FRAME_SIZE] {
        if self.current_buffer {
            &mut self.fb0
        } else {
            &mut self.fb1
        }
    }

    /// Get mutable access to the draw buffer for direct writes
    ///
    /// This provides low-level access to the internal framebuffer.
    /// Use with caution - you must write in the correct BCM format.
    ///
    /// # Returns
    /// Mutable reference to the draw buffer array
    pub const fn get_draw_buffer_mut(&mut self) -> &mut [u8; FRAME_SIZE] {
        self.get_draw_buffer()
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

        // Devkit PCB has address pins rotated: GPIO6→E, GPIO7→A, GPIO8→B, GPIO9→C, GPIO10→D
        // PIO bit 0 goes to GPIO6 (E) instead of A, so we rotate the row address left by 1
        // to compensate: when PIO outputs rotated value, the panel sees the correct row.
        // Rotate width is derived from ACTIVE_ROWS so this works for any panel size
        // (ACTIVE_ROWS must be a power of two, which is always the case for HUB75 panels).
        #[cfg(feature = "devkit_remap")]
        let y = {
            const ADDR_BITS: u32 = ACTIVE_ROWS.trailing_zeros();
            const ADDR_MASK: usize = ACTIVE_ROWS - 1;
            const _: () = assert!(
                ACTIVE_ROWS.is_power_of_two(),
                "devkit_remap requires ACTIVE_ROWS to be a power of two"
            );
            let half_row = y % ACTIVE_ROWS;
            let rotated = ((half_row << 1) | (half_row >> (ADDR_BITS - 1))) & ADDR_MASK;
            if y >= ACTIVE_ROWS {
                rotated + ACTIVE_ROWS
            } else {
                rotated
            }
        };

        let base_idx = x + ((y % (DISPLAY_HEIGHT / 2)) * DISPLAY_WIDTH * COLOR_BITS);

        c_r = GAMMA8[c_r as usize] as u16;
        c_g = GAMMA8[c_g as usize] as u16;
        c_b = GAMMA8[c_b as usize] as u16;

        // Devkit PCB swaps G/B through the level shifter channels:
        // GPIO1→B1, GPIO2→G1 (and GPIO4→B2, GPIO5→G2)
        // Swap c_r and c_g so bit0 carries red and bit1 carries blue
        #[cfg(feature = "devkit_remap")]
        {
            core::mem::swap(&mut c_r, &mut c_g);
        }

        for b in 0..COLOR_BITS {
            // Extract the n-th bit of each component of the color and pack them
            let cr = (c_r >> b) & 0b1;
            let cg = (c_g >> b) & 0b1;
            let cb = (c_b >> b) & 0b1;
            let packed_rgb = (cb << 2 | cg << 1 | cr) as u8;
            let idx = base_idx + b * DISPLAY_WIDTH;

            // Use current_buffer flag instead of pointer comparison
            let draw_buffer = if self.current_buffer {
                &mut self.fb0
            } else {
                &mut self.fb1
            };

            draw_buffer[idx] &= !(0b111 << shift);
            draw_buffer[idx] |= packed_rgb << shift;
        }
    }

    /// Clear the draw buffer
    pub fn clear(&mut self) {
        self.get_draw_buffer().fill(0);
    }

    /// Get pointer to active framebuffer (for DMA)
    pub const fn get_active_buffer_ptr(&self) -> *mut u8 {
        self.fb_ptr
    }

    /// Get pointer to delay array (for DMA)
    pub const fn get_delay_ptr(&self) -> *mut u32 {
        self.delay_ptr
    }

    /// Get pointer to the framebuffer pointer (for DMA chaining)
    pub const fn get_fb_ptr_addr(&self) -> *const *mut u8 {
        &raw const self.fb_ptr
    }

    /// Get pointer to the delay pointer (for DMA chaining)
    pub const fn get_delay_ptr_addr(&self) -> *const *mut u32 {
        &raw const self.delay_ptr
    }
}

// Safety: DisplayMemory contains only plain data and atomic operations
unsafe impl Send for DisplayMemory {}
unsafe impl Sync for DisplayMemory {}
