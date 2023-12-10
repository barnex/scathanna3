use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JumpPad {
	pub bounds: BoundingBox32,
	pub jump_height: f32,
}
