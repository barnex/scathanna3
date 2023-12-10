use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct SpawnPoint {
	pub position: vec3,
	pub yaw: f32,
}

impl SpawnPoint {
	pub fn orientation(&self) -> Orientation {
		Orientation { yaw: self.yaw, pitch: 0.0 }
	}
}
