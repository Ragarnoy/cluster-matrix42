use common::animations;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use std::{thread, time::Duration};

fn main() -> Result<(), std::convert::Infallible> {
    // Create a new simulator display that matches our panel size (128x128)
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(128, 128));

    // Configure the simulator window with a pixel scale of 8 for better visibility
    let output_settings = OutputSettingsBuilder::new()
        .scale(8)
        .pixel_spacing(1)
        .build();

    // Create a new window with the output settings
    let mut window = Window::new("Cluster Matrix Simulator", &output_settings);

    // Animation frame counter
    let mut frame: u32 = 0;

    'running: loop {
        // Draw the current frame of the animation
        // animations::stars::draw_animation_frame(&mut display, frame)?;
        animations::fortytwo::draw_animation_frame(&mut display, frame).unwrap();

        // Update the window with the contents of the display
        window.update(&display);

        // Check for events
        for event in window.events() {
            if event == SimulatorEvent::Quit {
                break 'running;
            }
        }

        // Increment frame counter
        frame = frame.wrapping_add(1);

        // Control animation speed (roughly 80 FPS)
        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
