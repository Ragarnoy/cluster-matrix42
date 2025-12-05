#ifndef PLUGIN_HELPERS_H
#define PLUGIN_HELPERS_H

#include "plugin_api.h"

// RGB565 color conversion
#define RGB565(r, g, b) \
    ((((r) & 0xF8) << 8) | (((g) & 0xFC) << 3) | (((b) & 0xF8) >> 3))

// Direct pixel access
#define PIXEL(fb, x, y) \
    ((fb)->pixels[(y) * DISPLAY_WIDTH + (x)])

// Input button bits
#define INPUT_UP    (1 << 0)
#define INPUT_DOWN  (1 << 1)
#define INPUT_LEFT  (1 << 2)
#define INPUT_RIGHT (1 << 3)
#define INPUT_A     (1 << 4)
#define INPUT_B     (1 << 5)
#define INPUT_START (1 << 6)
#define INPUT_SELECT (1 << 7)

#endif // PLUGIN_HELPERS_H