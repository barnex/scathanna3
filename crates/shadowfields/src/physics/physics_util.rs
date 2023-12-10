use super::internal::*;

pub const STD_GRAVITY:f32 = 9.8;

/// How fast does one need to jump to reach a height?
/// https://en.wikipedia.org/wiki/Torricelli%27s_equation.
pub fn jump_velocity_for_height(g: f32, height: f32) -> f32 {
	f32::sqrt(2.0 * g * height)
}
