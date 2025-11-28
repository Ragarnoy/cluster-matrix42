# Plugin System

A lightweight plugin system for the 128x128 cluster matrix display. Plugins can be written in C or Rust and run on both embedded hardware (ARM Cortex-M33) and the desktop simulator.

## Architecture

```
plugins/
├── plugin-api/          # Core API definitions (Rust + C header)
├── plugin-host/         # Embedded runtime (loads .bin files)
├── plugin-examples-c/   # C plugin examples
└── plugin-examples-rust/# Rust plugin examples
```

## Plugin API

Plugins receive a `PluginAPI` struct with three contexts:

| Context       | Purpose                                                                 |
|---------------|-------------------------------------------------------------------------|
| `framebuffer` | Direct pixel buffer access (128x128 RGB565)                             |
| `gfx`         | Drawing primitives (set_pixel, fill_rect, draw_line, draw_circle, blit) |
| `sys`         | Utilities (random, millis, rgb) and color constants                     |

### Lifecycle

```
init(api)    → Called once when plugin loads (return 0 for success)
update(api, inputs) → Called every frame (~60fps)
cleanup()    → Called when plugin unloads
```

### Input Flags

```
INPUT_UP, INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT
INPUT_A, INPUT_B, INPUT_START, INPUT_SELECT
```

## Writing a Rust Plugin

1. Create a new directory in `plugin-examples-rust/`
2. Add `Cargo.toml`:

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib", "cdylib"]

[[bin]]
name = "my_plugin"
path = "src/main.rs"

[dependencies]
plugin-api = { path = "../../plugin-api" }

[features]
default = []
simulator = ["plugin-api/std"]
```

3. Create `src/lib.rs`:

```rust
#![cfg_attr(not(feature = "simulator"), no_std)]

use plugin_api::prelude::*;

pub struct MyPlugin {
    // your state here
}

plugin_main!(MyPlugin, "my_plugin");

impl PluginImpl for MyPlugin {
    fn new() -> Self {
        Self { }
    }

    fn init(&mut self, _api: &mut PluginAPI) -> i32 {
        0 // success
    }

    fn update(&mut self, api: &mut PluginAPI, inputs: Inputs) {
        let gfx = api.gfx();
        let sys = api.sys();

        gfx.clear(sys.black());
        gfx.fill_rect(10, 10, 50, 50, sys.red());
    }

    fn cleanup(&mut self) { }
}
```

4. Create `src/main.rs` (for embedded target):

```rust
#![cfg_attr(not(feature = "simulator"), no_std)]
#![cfg_attr(not(feature = "simulator"), no_main)]

pub use my_plugin::*;

#[cfg(not(feature = "simulator"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

#[cfg(feature = "simulator")]
fn main() { }
```

5. Build - the plugin is auto-discovered!

## Writing a C Plugin

1. Create `my_plugin.c` in `plugin-examples-c/`:

```c
#include "plugin_api.h"
#include "plugin_helpers.h"

static const PluginAPI* api;

int32_t my_plugin_init(const PluginAPI* plugin_api) {
    api = plugin_api;
    return 0;
}

void my_plugin_update(const PluginAPI* plugin_api, uint32_t inputs) {
    FrameBuffer* fb = api->framebuffer;

    // Direct pixel access for performance
    for (int y = 0; y < 128; y++) {
        for (int x = 0; x < 128; x++) {
            fb->pixels[y * 128 + x] = RGB565(x, y, 128);
        }
    }
}

void my_plugin_cleanup(void) { }

__attribute__((section(".plugin_header")))
const PluginHeader PLUGIN_HEADER = {
    .magic = PLUGIN_MAGIC,
    .api_version = PLUGIN_API_VERSION,
    .name = "My Plugin",
    .init = my_plugin_init,
    .update = my_plugin_update,
    .cleanup = my_plugin_cleanup,
};
```

## Examples

| Plugin          | Language | Description                                              |
|-----------------|----------|----------------------------------------------------------|
| `plasma`        | C        | Animated plasma effect using direct framebuffer access   |
| `quadrant`      | C        | Static four-color quadrant test pattern                  |
| `bouncing_ball` | Rust     | Bouncing ball with trail effect, responds to A/B buttons |
| `quadrant_rust` | Rust     | Same as quadrant, demonstrates Rust plugin structure     |

## Building

Plugins are automatically compiled when building the host:

```bash
# Embedded (produces .bin files)
cargo build -p plugin-host --target thumbv8m.main-none-eabihf --release

# Simulator (produces shared libraries)
cargo build -p simulator --release
cargo run -p simulator --bin plugin_sim --release
```

## Requirements

- Rust stable toolchain
- `thumbv8m.main-none-eabihf` target (`rustup target add thumbv8m.main-none-eabihf`)
- `arm-none-eabi-gcc` for C plugins (embedded only)
- SDL2 for simulator

## Future Features

- **Memory Protection (MPU)** - Enable ARM MPU to prevent plugins from writing outside their allocated memory space
- **Panic Detection** - Detect and handle Rust panics in plugins without crashing the host
- **Fault Handling** - Recover from HardFaults and other exceptions caused by misbehaving plugins
- **Plugin Supervision** - Monitor `update()` execution time and disable plugins that take too long (watchdog)
- **Dynamic Loading** - Load plugins over the network (Ethernet/WiFi) at runtime
