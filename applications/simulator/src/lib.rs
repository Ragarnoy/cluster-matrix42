use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

pub mod native_plugin;
pub mod plugin_host;

pub use native_plugin::NativePlugin;
pub use plugin_host::{Plugin, SimulatorPluginRuntime};

pub type AnimationFn =
    fn(&mut SimulatorDisplay<Rgb565>, u32) -> Result<(), core::convert::Infallible>;

#[derive(Debug, Clone)]
pub struct SimulatorConfig {
    pub size: Size,
    pub scale: u32,
    pub pixel_spacing: u32,
    pub title: String,
    pub target_fps: Option<u32>,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            size: Size::new(128, 128),
            scale: 8,
            pixel_spacing: 1,
            title: "Hub75 Matrix Simulator".to_string(),
            target_fps: Some(60),
        }
    }
}

pub struct Simulator {
    display: SimulatorDisplay<Rgb565>,
    window: Window,
    config: SimulatorConfig,
}

impl Simulator {
    pub fn new(config: SimulatorConfig) -> Result<Self, String> {
        let display = SimulatorDisplay::<Rgb565>::new(config.size);

        let output_settings = OutputSettingsBuilder::new()
            .scale(config.scale)
            .pixel_spacing(config.pixel_spacing)
            .build();

        let window = Window::new(&config.title, &output_settings);

        Ok(Self {
            display,
            window,
            config,
        })
    }

    pub fn run_animation(
        &mut self,
        animation_fn: AnimationFn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut frame: u32 = 0;
        let frame_duration = self
            .config
            .target_fps
            .map(|fps| std::time::Duration::from_millis(1000 / fps as u64));

        'running: loop {
            let frame_start = std::time::Instant::now();

            // Draw the animation frame
            animation_fn(&mut self.display, frame)?;

            // Update the window
            self.window.update(&self.display);

            // Handle events
            for event in self.window.events() {
                if event == SimulatorEvent::Quit {
                    break 'running;
                }
            }

            // Control frame rate if specified
            if let Some(duration) = frame_duration {
                let elapsed = frame_start.elapsed();
                if elapsed < duration {
                    std::thread::sleep(duration - elapsed);
                }
            }

            frame = frame.wrapping_add(1);
        }

        Ok(())
    }

    pub fn run_with_callback<F>(
        &mut self,
        mut callback: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(&mut SimulatorDisplay<Rgb565>, u32) -> Result<(), core::convert::Infallible>,
    {
        let mut frame: u32 = 0;
        let frame_duration = self
            .config
            .target_fps
            .map(|fps| std::time::Duration::from_millis(1000 / fps as u64));

        'running: loop {
            let frame_start = std::time::Instant::now();

            // Run the callback
            callback(&mut self.display, frame)?;

            // Update the window
            self.window.update(&self.display);

            // Handle events
            for event in self.window.events() {
                if event == SimulatorEvent::Quit {
                    break 'running;
                }
            }

            // Control frame rate if specified
            if let Some(duration) = frame_duration {
                let elapsed = frame_start.elapsed();
                if elapsed < duration {
                    std::thread::sleep(duration - elapsed);
                }
            }

            frame = frame.wrapping_add(1);
        }

        Ok(())
    }

    pub const fn display_mut(&mut self) -> &mut SimulatorDisplay<Rgb565> {
        &mut self.display
    }

    pub const fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}

// Convenience functions for common configurations
pub fn create_hub75_simulator(size: Size) -> Result<Simulator, String> {
    let config = SimulatorConfig {
        size,
        title: format!("Hub75 Matrix Simulator ({}x{})", size.width, size.height),
        scale: 6,
        ..Default::default()
    };
    Simulator::new(config)
}

pub fn create_64x64_simulator() -> Result<Simulator, String> {
    create_hub75_simulator(Size::new(64, 64))
}

pub fn create_128x128_simulator() -> Result<Simulator, String> {
    create_hub75_simulator(Size::new(128, 128))
}
