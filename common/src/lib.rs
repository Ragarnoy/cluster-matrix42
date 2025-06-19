#![no_std]

#[cfg(feature = "std")]
extern crate std;

use embedded_graphics::prelude::DrawTarget;

pub mod animations;
pub mod constants;
pub mod shared;
pub mod visualization;
