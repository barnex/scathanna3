use super::internal::*;

/// Definition of a point light, parsed from GLTF KHR_lights_punctual.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointLightDef{
	pub pos: vec3,
	pub color: vec3,
	pub range: f32,
}