//! Bouncing ball plugin
//!
//! A simple bouncing ball demo that responds to input.

#![cfg_attr(not(feature = "simulator"), no_std)]

use plugin_api::prelude::*;

pub struct BouncingBallPlugin {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
    radius: i32,
}

// Generate C ABI functions for the plugin
plugin_main!(BouncingBallPlugin, "bouncing_ball");

impl PluginImpl for BouncingBallPlugin {
    fn new() -> Self {
        Self {
            x: 64,
            y: 64,
            vx: 2,
            vy: 3,
            radius: 8,
        }
    }

    fn init(&mut self, _api: &mut PluginAPI) -> i32 {
        0
    }

    fn update(&mut self, api: &mut PluginAPI, inputs: Inputs) {
        let gfx = api.gfx();
        let sys = api.sys();

        // Handle input to change ball size
        if inputs.a() && self.radius < 32 {
            self.radius += 1;
        }
        if inputs.b() && self.radius > 2 {
            self.radius -= 1;
        }

        // Clear screen
        gfx.clear(sys.black());

        // Update position
        self.x += self.vx;
        self.y += self.vy;

        // Bounce off walls
        if self.x - self.radius <= 0 || self.x + self.radius >= DISPLAY_WIDTH as i32 {
            self.vx = -self.vx;
            self.x = self
                .x
                .clamp(self.radius, DISPLAY_WIDTH as i32 - self.radius);
        }
        if self.y - self.radius <= 0 || self.y + self.radius >= DISPLAY_HEIGHT as i32 {
            self.vy = -self.vy;
            self.y = self
                .y
                .clamp(self.radius, DISPLAY_HEIGHT as i32 - self.radius);
        }

        // Color based on velocity
        let speed = (self.vx.abs() + self.vy.abs()) as u8;
        let color = sys.rgb(
            speed.saturating_mul(30),
            200u8.saturating_sub(speed.saturating_mul(20)),
            150,
        );

        // Draw ball
        gfx.draw_circle(self.x, self.y, self.radius, color);

        // Draw trail (multiple circles with decreasing intensity)
        for i in 1..4i32 {
            let trail_x = self.x - self.vx * i;
            let trail_y = self.y - self.vy * i;
            let trail_color = sys.rgb(
                speed.saturating_mul(10) / i as u8,
                80 / i as u8,
                60 / i as u8,
            );
            gfx.draw_circle(trail_x, trail_y, self.radius / 2, trail_color);
        }
    }

    fn cleanup(&mut self) {
        // Nothing to clean up
    }
}

impl Default for BouncingBallPlugin {
    fn default() -> Self {
        Self::new()
    }
}
