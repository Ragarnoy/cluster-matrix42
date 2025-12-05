// plugins/plugin-examples-c/quadrant.c
// Quadrant plugin - displays 4 colored panels (red, green, blue, yellow)
#include <stdint.h>
#include "plugin_api.h"
#include "plugin_helpers.h"

static const PluginAPI* api;

// Static colors for each quadrant (RGB565 values)
static const uint16_t COLOR_RED = RGB565(255, 0, 0);
static const uint16_t COLOR_GREEN = RGB565(0, 255, 0);
static const uint16_t COLOR_BLUE = RGB565(0, 0, 255);
static const uint16_t COLOR_YELLOW = RGB565(255, 255, 0);

// Fill a rectangle with a color
static void fill_rect(FrameBuffer* fb, int x, int y, int width, int height, uint16_t color) {
    for (int py = y; py < y + height; py++) {
        for (int px = x; px < x + width; px++) {
            if (px >= 0 && px < 128 && py >= 0 && py < 128) {
                fb->pixels[py * 128 + px] = color;
            }
        }
    }
}

int32_t quadrant_init(const PluginAPI* plugin_api) {
    api = plugin_api;
    return 0; // Success
}

void quadrant_update(const PluginAPI* plugin_api, uint32_t inputs) {
    FrameBuffer* fb = api->framebuffer;

    // Clear to black
    for (int i = 0; i < FRAMEBUFFER_SIZE; i++) {
        fb->pixels[i] = RGB565(0, 0, 0);
    }

    // Panel layout (natural 128x128 rendering, scaled by display driver):
    // Each panel is 64x64 in the plugin's 128x128 framebuffer
    // Top-left: Red, Top-right: Green
    // Bottom-left: Blue, Bottom-right: Yellow

    // Draw each panel with its static color
    fill_rect(fb, 0, 0, 64, 64, COLOR_RED);      // Panel 1: Top-left - Red
    fill_rect(fb, 64, 0, 64, 64, COLOR_GREEN);   // Panel 2: Top-right - Green
    fill_rect(fb, 0, 64, 64, 64, COLOR_BLUE);    // Panel 3: Bottom-left - Blue
    fill_rect(fb, 64, 64, 64, 64, COLOR_YELLOW); // Panel 4: Bottom-right - Yellow

    // Draw white border lines between panels
    uint16_t white = RGB565(255, 255, 255);

    // Vertical center line (2 pixels wide)
    fill_rect(fb, 63, 0, 2, 128, white);

    // Horizontal center line (2 pixels wide)
    fill_rect(fb, 0, 63, 128, 2, white);
}

void quadrant_cleanup(void) {
    // Nothing to clean up
}

// Export the plugin header
__attribute__((section(".plugin_header")))
const PluginHeader PLUGIN_HEADER = {
    .magic = PLUGIN_MAGIC,
    .api_version = PLUGIN_API_VERSION,
    .name = "Quadrant Test",
    .init = quadrant_init,
    .update = quadrant_update,
    .cleanup = quadrant_cleanup,
};
