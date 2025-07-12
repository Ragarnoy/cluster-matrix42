use graphics_common::animations;
use simulator::{AnimationFn, create_128x128_simulator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sim = create_128x128_simulator()?;
    let animation: AnimationFn = animations::fortytwo::draw_animation_frame;
    sim.run_animation(animation)
}
