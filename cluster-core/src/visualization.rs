//! Cluster visualization system

pub mod cluster;
pub mod display;
pub mod layouts;
pub mod renderer;
pub mod seats;

// Re-export commonly used types for convenience
pub use cluster::{Cluster, ClusterLayout, SeatPosition, ZoneInfo};
pub use display::{DEFAULT_LAYOUT, DisplayLayout};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
pub use layouts::{CustomLayout, GridLayout};
pub use renderer::ClusterRenderer;

// Re-export layout presets
use crate::parsing::Layout;
pub use layouts::presets;
pub use seats::{Seat, SeatState, SeatType};

/// Draw a cluster visualization frame
pub fn draw_cluster_frame<D>(display: &mut D, cluster: &Layout, frame: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let renderer = ClusterRenderer::new();
    renderer.render_frame::<D>(display, &cluster.f0, frame)
}
