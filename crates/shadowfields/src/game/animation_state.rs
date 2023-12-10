use super::internal::*;

/// Controlled by the local client, never overwritten by the server.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct AnimationState {
	pub feet_phase: f32, // used for avatar animation (-PI..PI)
	pub feet_pitch: f32,
}

