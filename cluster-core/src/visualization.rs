//! Cluster visualization system

pub mod display;
pub mod renderer;

// Re-export commonly used types for convenience
use crate::models::Layout;
pub use display::{DEFAULT_LAYOUT, DisplayLayout};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
pub use renderer::ClusterRenderer;

/// Draw a cluster visualization frame
pub fn draw_cluster_frame<D>(display: &mut D, cluster: &Layout, frame: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let renderer = ClusterRenderer::new();
    renderer.render_frame::<D>(display, &cluster.f0, frame)
}
