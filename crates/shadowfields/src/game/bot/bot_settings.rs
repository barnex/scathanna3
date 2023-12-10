use crate::prelude::*;

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct BotSettings {
	pub enabled: bool,
	pub respawn_time: f32,
	pub response_time: f32,
	pub aim_speed: f32, // radians / s.
	pub jump_period: f32,
}

impl Default for BotSettings {
	fn default() -> Self {
		Self {
			enabled: false,
			respawn_time: 1.0,
			response_time: 0.5,
			aim_speed: 1.5,
			jump_period: 10.0,
		}
	}
}
