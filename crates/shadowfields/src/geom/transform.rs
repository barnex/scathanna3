use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Transform {
	pub translation: vec3,
	pub scale: f32,
}

impl Transform {
	pub fn translation(translation: vec3) -> Self {
		Self { translation, ..default() }
	}

	pub fn matrix(&self) -> mat4 {
		translation_matrix(self.translation) * scale_matrix(self.scale)
	}
}

impl Default for Transform {
	fn default() -> Self {
		Self { translation: default(), scale: 1.0 }
	}
}
