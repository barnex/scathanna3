///! Utilities for working with colors and SRGB color space.
mod srgb;

pub use srgb::*;

use vector::*;

pub const BSOD_BLUE: vec3 =  vec3(srgb_to_linear(98), srgb_to_linear(102), srgb_to_linear(255));
