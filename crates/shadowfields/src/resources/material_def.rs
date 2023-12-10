use super::internal::*;

/// Material definition: handles to textures for color, normals, ...
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialDef {
	pub base_color: Handle,
	pub emissive: Option<EmissiveDef>,
	pub normal_map: Option<Handle>,
	//...
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmissiveDef {
	pub emissive_texture: Handle,
	pub emissive_strength: f32,
}

impl Default for MaterialDef {
	fn default() -> Self {
		Self {
			base_color: handle("#aaaaaa"), // TODO
			emissive: None,
			normal_map: None,
		}
	}
}
