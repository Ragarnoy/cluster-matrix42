//! Quadrant test plugin
//!
//! Displays four colored quadrants to test the display.

#![cfg_attr(not(feature = "simulator"), no_std)]

use plugin_api::prelude::*;

pub struct QuadrantPlugin;

// Generate C ABI functions for the plugin
plugin_main!(QuadrantPlugin, "quadrant_rust");

impl PluginImpl for QuadrantPlugin {
    fn new() -> Self {
        Self
    }

    fn init(&mut self, _api: &mut PluginAPI) -> i32 {
        0 // Success
    }

    fn update(&mut self, api: &mut PluginAPI, _inputs: Inputs) {
        let gfx = api.gfx();
        let sys = api.sys();

        // Top-left: Red
        gfx.fill_rect(0, 0, 64, 64, sys.red());

        // Top-right: Green
        gfx.fill_rect(64, 0, 64, 64, sys.green());

        // Bottom-left: Blue
        gfx.fill_rect(0, 64, 64, 64, sys.blue());

        // Bottom-right: Yellow
        gfx.fill_rect(64, 64, 64, 64, sys.yellow());

        // Draw white borders
        gfx.draw_line(63, 0, 63, 127, sys.white()); // Vertical middle
        gfx.draw_line(0, 63, 127, 63, sys.white()); // Horizontal middle
    }

    fn cleanup(&mut self) {
        // Nothing to clean up
    }
}

impl Default for QuadrantPlugin {
    fn default() -> Self {
        Self::new()
    }
}
