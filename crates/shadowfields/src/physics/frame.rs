use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Frame {
	pub position: vec3,
	pub velocity: vec3,
	pub orientation: Orientation,
}
