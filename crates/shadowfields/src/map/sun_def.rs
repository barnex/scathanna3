use super::internal::*;

/// Definition of a sun light, parsed from GLTF KHR_lights_punctual.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SunDef{
	pub dir: vec3,
	pub color: vec3,
}