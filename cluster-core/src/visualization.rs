//! Cluster visualization system

pub mod cluster;
pub mod display;
pub mod layouts;
pub mod renderer;
pub mod seats;

// Re-export commonly used types for convenience
pub use cluster::{Cluster, ClusterLayout, SeatPosition, ZoneInfo};
pub use display::{DisplayLayout, DEFAULT_LAYOUT};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
pub use layouts::{CustomLayout, GridLayout};
pub use renderer::ClusterRenderer;

// Re-export layout presets
pub use layouts::presets;
pub use seats::{Seat, SeatState, SeatType};

/// Draw a cluster visualization frame
pub fn draw_cluster_frame<D, L>(
    display: &mut D,
    cluster: &Cluster<L>,
    motd: &str,
    frame: u32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
    L: ClusterLayout,
{
    let renderer = ClusterRenderer::new();
    renderer.render_frame(display, cluster, motd, frame)
}
