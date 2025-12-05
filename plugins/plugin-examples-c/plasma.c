#include <stdint.h>
#include "plugin_api.h"
#include "plugin_helpers.h"

// Simple sine table for performance (one full period: 0 to 2π)
// Generated as: value[i] = 128 + 112 * sin(2π * i / 64)
static const uint8_t sine_table[64] = {
    128, 139, 150, 161, 171, 181, 190, 199,
    207, 215, 221, 227, 231, 235, 238, 239,
    240, 239, 238, 235, 231, 227, 221, 215,
    207, 199, 190, 181, 171, 161, 150, 139,
    128, 117, 106, 95,  85,  75,  66,  57,
    49,  41,  35,  29,  25,  21,  18,  17,
    16,  17,  18,  21,  25,  29,  35,  41,
    49,  57,  66,  75,  85,  95,  106, 117
};

static const PluginAPI* api;
static uint32_t time_offset = 0;

// Fast sine approximation
static uint8_t fast_sin(uint8_t angle) {
    return sine_table[angle & 0x3F]; // Wrap to 64 entries
}

int32_t plasma_init(const PluginAPI* plugin_api) {
    api = plugin_api;
    time_offset = 0;
    return 0; // Success
}

void plasma_update(const PluginAPI* plugin_api, uint32_t inputs) {
    FrameBuffer* fb = api->framebuffer;

    // DIRECT BUFFER ACCESS for maximum performance
    for (int y = 0; y < 128; y++) {
        for (int x = 0; x < 128; x++) {
            // Calculate plasma value
            uint8_t v1 = fast_sin((x >> 1) + time_offset);
            uint8_t v2 = fast_sin((y >> 1) + (time_offset << 1));
            uint8_t v3 = fast_sin(((x + y) >> 2) + (time_offset * 3));
            uint8_t v = (v1 + v2 + v3) / 3;

            // Convert to RGB565
            fb->pixels[y * 128 + x] = RGB565(v, v >> 1, 255 - v);
        }
    }

    time_offset++;
}

void plasma_cleanup(void) {
    // Nothing to clean up
}

// Export the plugin header
__attribute__((section(".plugin_header")))
const PluginHeader PLUGIN_HEADER = {
    .magic = PLUGIN_MAGIC,
    .api_version = PLUGIN_API_VERSION,
    .name = "Plasma Effect",
    .init = plasma_init,
    .update = plasma_update,
    .cleanup = plasma_cleanup,
};