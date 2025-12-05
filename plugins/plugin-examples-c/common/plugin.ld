# plugins/plugin-examples-c/common/plugin.ld
MEMORY {
    PLUGIN : ORIGIN = 0x00000000, LENGTH = 64K
}

SECTIONS {
    .plugin_header : {
        KEEP(*(.plugin_header))
    } > PLUGIN

    .text : {
        *(.text*)
        *(.rodata*)
    } > PLUGIN

    .data : {
        *(.data*)
    } > PLUGIN

    .bss : {
        *(.bss*)
    } > PLUGIN
}