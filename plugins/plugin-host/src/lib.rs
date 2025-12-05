#![no_std]

use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut};
use plugin_api::*;
use static_cell::StaticCell;

include!(concat!(env!("OUT_DIR"), "/plugin_includes.rs"));

static PLUGIN_RUNTIME: StaticCell<PluginRuntime> = StaticCell::new();

// 64KB RAM buffer for plugin code (must be 4-byte aligned for ARM execution)
#[repr(align(4))]
struct AlignedBuffer([u8; 65536]);

#[unsafe(link_section = ".bss")]
static mut PLUGIN_LOAD_BUFFER: AlignedBuffer = AlignedBuffer([0; 65536]);

struct LoadedPlugin {
    header: &'static PluginHeader,
    #[allow(dead_code)]
    name: &'static str,
}

pub struct PluginRuntime {
    framebuffer: FrameBuffer,
    graphics_ctx: GraphicsContext,
    system_ctx: SystemContext,
    api: PluginAPI,
    current_plugin: Option<LoadedPlugin>,
}

// Global pointer for callbacks
static mut RUNTIME_PTR: Option<*mut PluginRuntime> = None;

impl PluginRuntime {
    /// Initialize the global plugin runtime
    pub fn init() -> &'static mut Self {
        let runtime = PLUGIN_RUNTIME.init(Self {
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
                framebuffer: core::ptr::null_mut(),
                gfx: core::ptr::null(),
                sys: core::ptr::null(),
            },
            current_plugin: None,
        });

        runtime.api.framebuffer = &mut runtime.framebuffer as *mut _;
        runtime.api.gfx = &runtime.graphics_ctx as *const _;
        runtime.api.sys = &runtime.system_ctx as *const _;

        unsafe {
            RUNTIME_PTR = Some(runtime as *mut _);
        }

        runtime
    }

    pub fn load_plugin(&mut self, plugin_bytes: &'static [u8]) -> Result<(), &'static str> {
        if plugin_bytes.len() < size_of::<PluginHeader>() {
            return Err("Plugin binary too small");
        }

        const BUFFER_SIZE: usize = 65536;
        if plugin_bytes.len() > BUFFER_SIZE {
            return Err("Plugin too large for load buffer");
        }

        // Copy from flash to RAM and relocate (plugins are linked at 0x00000000)
        unsafe {
            let buffer_ptr = addr_of_mut!(PLUGIN_LOAD_BUFFER.0).cast::<u8>();

            core::ptr::copy_nonoverlapping(plugin_bytes.as_ptr(), buffer_ptr, plugin_bytes.len());

            // Zero remaining buffer space for .bss section (uninitialized data)
            // This ensures all static/global variables are properly zeroed regardless of actual BSS size
            let bss_start = plugin_bytes.len();
            let remaining_size = BUFFER_SIZE - bss_start;
            core::ptr::write_bytes(buffer_ptr.add(bss_start), 0, remaining_size);

            let header = &*(addr_of!(PLUGIN_LOAD_BUFFER.0).cast::<PluginHeader>());

            if header.magic != PLUGIN_MAGIC {
                return Err("Invalid plugin magic number");
            }

            if header.api_version != PLUGIN_API_VERSION {
                return Err("Plugin API version mismatch");
            }

            // Relocate function pointers from 0x00000000 to buffer address
            let base_addr = addr_of!(PLUGIN_LOAD_BUFFER.0).cast::<u8>() as usize;

            // ARM Thumb bit (bit 0) must be preserved during relocation
            let init_offset = header.init as usize;
            let update_offset = header.update as usize;
            let cleanup_offset = header.cleanup as usize;

            #[cfg(feature = "defmt")]
            {
                defmt::debug!("Plugin relocation:");
                defmt::debug!("  Base address: {:#x}", base_addr);
                defmt::debug!(
                    "  Init offset: {:#x} -> {:#x}",
                    init_offset,
                    base_addr + init_offset
                );
                defmt::debug!(
                    "  Update offset: {:#x} -> {:#x}",
                    update_offset,
                    base_addr + update_offset
                );
                defmt::debug!(
                    "  Cleanup offset: {:#x} -> {:#x}",
                    cleanup_offset,
                    base_addr + cleanup_offset
                );
            }

            let relocated_header = PluginHeader {
                magic: header.magic,
                api_version: header.api_version,
                name: header.name,
                init: core::mem::transmute::<usize, unsafe extern "C" fn(*const PluginAPI) -> i32>(
                    base_addr + init_offset,
                ),
                update: core::mem::transmute::<usize, unsafe extern "C" fn(*const PluginAPI, u32)>(
                    base_addr + update_offset,
                ),
                cleanup: core::mem::transmute::<usize, unsafe extern "C" fn()>(
                    base_addr + cleanup_offset,
                ),
            };

            core::ptr::write(
                addr_of_mut!(PLUGIN_LOAD_BUFFER.0).cast::<PluginHeader>(),
                relocated_header,
            );

            // Sync caches for executable code
            #[cfg(target_arch = "arm")]
            {
                core::arch::asm!("dsb");
                core::arch::asm!("isb");
            }

            let final_header = &*(addr_of!(PLUGIN_LOAD_BUFFER.0).cast::<PluginHeader>());

            #[cfg(feature = "defmt")]
            defmt::debug!("Calling plugin init at {:#x}", final_header.init as usize);

            let result = (final_header.init)(&self.api as *const _);

            #[cfg(feature = "defmt")]
            defmt::debug!("Plugin init returned: {}", result);

            if result != 0 {
                return Err("Plugin initialization failed");
            }

            let name = {
                let mut len = 0;
                while len < 32 && final_header.name[len] != 0 {
                    len += 1;
                }
                core::str::from_utf8(&final_header.name[..len]).unwrap_or("invalid string")
            };

            self.current_plugin = Some(LoadedPlugin {
                header: final_header,
                name,
            });
        }

        Ok(())
    }

    pub fn update(&mut self, inputs: u32) {
        if let Some(plugin) = &self.current_plugin {
            unsafe {
                (plugin.header.update)(&self.api as *const _, inputs);
            }
            self.framebuffer.frame_counter = self.framebuffer.frame_counter.wrapping_add(1);
        }
    }

    pub fn framebuffer(&self) -> &FrameBuffer {
        &self.framebuffer
    }

    pub fn unload_plugin(&mut self) {
        if let Some(plugin) = self.current_plugin.take() {
            unsafe {
                (plugin.header.cleanup)();
            }
        }
    }
}

// Graphics functions with bounds checking
fn set_pixel(runtime: &mut PluginRuntime, x: i32, y: i32, color: u16) {
    if x >= 0 && x < DISPLAY_WIDTH as i32 && y >= 0 && y < DISPLAY_HEIGHT as i32 {
        let idx = (y as usize) * DISPLAY_WIDTH + (x as usize);
        runtime.framebuffer.pixels[idx] = color;
    } else {
        #[cfg(feature = "defmt")]
        defmt::trace!("set_pixel out of bounds: ({}, {})", x, y);
    }
}

fn get_pixel(runtime: &PluginRuntime, x: i32, y: i32) -> u16 {
    if x >= 0 && x < DISPLAY_WIDTH as i32 && y >= 0 && y < DISPLAY_HEIGHT as i32 {
        let idx = (y as usize) * DISPLAY_WIDTH + (x as usize);
        runtime.framebuffer.pixels[idx]
    } else {
        #[cfg(feature = "defmt")]
        defmt::trace!("get_pixel out of bounds: ({}, {})", x, y);
        0
    }
}

fn clear(runtime: &mut PluginRuntime, color: u16) {
    runtime.framebuffer.pixels.fill(color);
}

fn fill_rect(runtime: &mut PluginRuntime, x: i32, y: i32, w: i32, h: i32, color: u16) {
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

fn draw_line(runtime: &mut PluginRuntime, x0: i32, y0: i32, x1: i32, y1: i32, color: u16) {
    let mut x = x0;
    let mut y = y0;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        set_pixel(runtime, x, y, color);

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

fn draw_circle(runtime: &mut PluginRuntime, cx: i32, cy: i32, radius: i32, color: u16) {
    if radius < 0 {
        #[cfg(feature = "defmt")]
        defmt::warn!("draw_circle: negative radius {}", radius);
        return;
    }

    let mut x = radius;
    let mut y = 0;
    let mut decision = 1 - radius;

    while x >= y {
        set_pixel(runtime, cx + x, cy + y, color);
        set_pixel(runtime, cx - x, cy + y, color);
        set_pixel(runtime, cx + x, cy - y, color);
        set_pixel(runtime, cx - x, cy - y, color);
        set_pixel(runtime, cx + y, cy + x, color);
        set_pixel(runtime, cx - y, cy + x, color);
        set_pixel(runtime, cx + y, cy - x, color);
        set_pixel(runtime, cx - y, cy - x, color);

        y += 1;

        if decision <= 0 {
            decision += 2 * y + 1;
        } else {
            x -= 1;
            decision += 2 * (y - x) + 1;
        }
    }
}

fn blit(runtime: &mut PluginRuntime, x: i32, y: i32, w: i32, h: i32, data: *const u16) -> bool {
    if data.is_null() {
        #[cfg(feature = "defmt")]
        defmt::warn!("blit: null data pointer");
        return false;
    }

    if w <= 0 || h <= 0 || w > 1024 || h > 1024 {
        #[cfg(feature = "defmt")]
        defmt::warn!("blit: invalid dimensions {}x{}", w, h);
        return false;
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

    true
}

// C API wrappers
unsafe extern "C" fn gfx_set_pixel(x: i32, y: i32, color: u16) {
    unsafe {
        if let Some(runtime) = RUNTIME_PTR {
            set_pixel(&mut *runtime, x, y, color);
        }
    }
}

unsafe extern "C" fn gfx_get_pixel(x: i32, y: i32) -> u16 {
    unsafe { RUNTIME_PTR.map_or(0, |runtime| get_pixel(&*runtime, x, y)) }
}

unsafe extern "C" fn gfx_clear(color: u16) {
    unsafe {
        if let Some(runtime) = RUNTIME_PTR {
            clear(&mut *runtime, color);
        }
    }
}

unsafe extern "C" fn gfx_fill_rect(x: i32, y: i32, w: i32, h: i32, color: u16) {
    unsafe {
        if let Some(runtime) = RUNTIME_PTR {
            fill_rect(&mut *runtime, x, y, w, h, color);
        }
    }
}

unsafe extern "C" fn gfx_draw_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u16) {
    unsafe {
        if let Some(runtime) = RUNTIME_PTR {
            draw_line(&mut *runtime, x0, y0, x1, y1, color);
        }
    }
}

unsafe extern "C" fn gfx_draw_circle(cx: i32, cy: i32, radius: i32, color: u16) {
    unsafe {
        if let Some(runtime) = RUNTIME_PTR {
            draw_circle(&mut *runtime, cx, cy, radius, color);
        }
    }
}

unsafe extern "C" fn gfx_blit(x: i32, y: i32, w: i32, h: i32, data: *const u16) {
    unsafe {
        if let Some(runtime) = RUNTIME_PTR {
            blit(&mut *runtime, x, y, w, h, data);
        }
    }
}

// System utilities
unsafe extern "C" fn sys_random() -> u32 {
    static mut SEED: u32 = 0xDEADBEEF;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        SEED
    }
}

unsafe extern "C" fn sys_millis() -> u32 {
    unsafe {
        RUNTIME_PTR.map_or(0, |runtime| {
            (*runtime).framebuffer.frame_counter.saturating_mul(16)
        })
    }
}

unsafe extern "C" fn sys_rgb(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | ((b as u16 & 0xF8) >> 3)
}
