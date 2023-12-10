use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PickupPoint {
	pub pos: vec3,
	pub item: Handle,
	pub timer: Timer,
}
