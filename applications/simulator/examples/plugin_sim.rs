//! Plugin simulator binary
//!
//! Runs plugins in the embedded-graphics simulator window.
//! Supports both C and Rust plugins compiled as shared libraries.
//!
//! Controls:
//! - Arrow keys: D-pad input
//! - Z: A button
//! - X: B button
//! - Enter: Start
//! - Backspace: Select
//! - Tab: Switch to next plugin
//! - Escape: Quit

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use plugin_api::{
    INPUT_A, INPUT_B, INPUT_DOWN, INPUT_LEFT, INPUT_RIGHT, INPUT_SELECT, INPUT_START, INPUT_UP,
};
use simulator::{NativePlugin, Plugin, SimulatorPluginRuntime};
use std::time::{Duration, Instant};

/// Plugin entry with its type info
struct PluginEntry {
    name: &'static str,
    is_c: bool, // true = C plugin, false = Rust plugin
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Plugin Simulator");
    println!("================");
    println!("Controls:");
    println!("  Arrow keys: D-pad");
    println!("  Z: A button");
    println!("  X: B button");
    println!("  Enter: Start");
    println!("  Backspace: Select");
    println!("  Tab: Switch plugin");
    println!("  Escape: Quit");
    println!();

    // Get available plugins
    let available_plugins: Vec<PluginEntry> = NativePlugin::all_available_plugins()
        .into_iter()
        .map(|(name, is_c)| PluginEntry { name, is_c })
        .collect();

    println!("Available plugins:");
    for entry in &available_plugins {
        let kind = if entry.is_c { "C" } else { "Rust" };
        println!("  - {} ({})", entry.name, kind);
    }
    println!();

    if available_plugins.is_empty() {
        eprintln!("No plugins available!");
        return Ok(());
    }

    // Create display
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(128, 128));

    let output_settings = OutputSettingsBuilder::new()
        .scale(6)
        .pixel_spacing(1)
        .build();

    let mut window = Window::new("Plugin Simulator", &output_settings);

    // Create plugin runtime
    let mut runtime = SimulatorPluginRuntime::new();

    // Load first plugin
    let mut current_plugin_idx = 0;
    let entry = &available_plugins[current_plugin_idx];
    let mut current_plugin = load_plugin(entry)?;

    println!(
        "Loading plugin: {} ({})",
        entry.name,
        if entry.is_c { "C" } else { "Rust" }
    );
    runtime.init_plugin(&mut current_plugin);

    // Input state
    let mut inputs: u32 = 0;

    // Frame timing
    let target_frame_duration = Duration::from_millis(16); // ~60 FPS
    let mut frame_count: u64 = 0;
    let mut fps_timer = Instant::now();

    // Initial window update required before calling events()
    window.update(&display);

    'running: loop {
        let frame_start = Instant::now();

        // Handle events
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Up => inputs |= INPUT_UP,
                    Keycode::Down => inputs |= INPUT_DOWN,
                    Keycode::Left => inputs |= INPUT_LEFT,
                    Keycode::Right => inputs |= INPUT_RIGHT,
                    Keycode::Z => inputs |= INPUT_A,
                    Keycode::X => inputs |= INPUT_B,
                    Keycode::Return => inputs |= INPUT_START,
                    Keycode::Backspace => inputs |= INPUT_SELECT,
                    Keycode::Tab => {
                        // Cleanup current plugin
                        current_plugin.cleanup();

                        // Switch to next plugin
                        current_plugin_idx = (current_plugin_idx + 1) % available_plugins.len();
                        let entry = &available_plugins[current_plugin_idx];

                        println!(
                            "Switching to plugin: {} ({})",
                            entry.name,
                            if entry.is_c { "C" } else { "Rust" }
                        );

                        // Reinitialize runtime and load new plugin
                        runtime = SimulatorPluginRuntime::new();
                        current_plugin = load_plugin(entry).expect("Failed to load plugin");
                        runtime.init_plugin(&mut current_plugin);
                    }
                    Keycode::Escape => break 'running,
                    _ => {}
                },
                SimulatorEvent::KeyUp { keycode, .. } => match keycode {
                    Keycode::Up => inputs &= !INPUT_UP,
                    Keycode::Down => inputs &= !INPUT_DOWN,
                    Keycode::Left => inputs &= !INPUT_LEFT,
                    Keycode::Right => inputs &= !INPUT_RIGHT,
                    Keycode::Z => inputs &= !INPUT_A,
                    Keycode::X => inputs &= !INPUT_B,
                    Keycode::Return => inputs &= !INPUT_START,
                    Keycode::Backspace => inputs &= !INPUT_SELECT,
                    _ => {}
                },
                _ => {}
            }
        }

        // Update current plugin
        runtime.update(&mut current_plugin, inputs);

        // Render to display
        runtime.render_to_display(&mut display);

        // Update window
        window.update(&display);

        // Frame timing
        frame_count += 1;
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            println!("FPS: {} ({})", frame_count, current_plugin.name());
            frame_count = 0;
            fps_timer = Instant::now();
        }

        // Control frame rate
        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_duration {
            std::thread::sleep(target_frame_duration - elapsed);
        }
    }

    // Cleanup
    current_plugin.cleanup();

    println!("Simulator closed");
    Ok(())
}

fn load_plugin(entry: &PluginEntry) -> Result<NativePlugin, String> {
    if entry.is_c {
        NativePlugin::load_c_plugin(entry.name)
    } else {
        NativePlugin::load_rust_plugin(entry.name)
    }
}
