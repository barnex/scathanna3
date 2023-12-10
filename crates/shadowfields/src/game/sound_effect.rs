use super::internal::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SoundEffect {
	pub clip_name: Handle,
	pub volume: f32,
	pub spatial: Option<Spatial>,
}

/// Information needed to create spatial sound.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Spatial {
	/// Where the sound originates.
	pub location: vec3,
	// Distance at which the sound has unit volume.
	//pub unit_distance: f32,
}

impl SoundEffect {
	/// Construct a SoundEffect message without spatial audio.
	/// Used e.g. for the announcer's voice.
	pub fn raw(clip_name: Handle) -> Self {
		Self {
			clip_name,
			volume: 1.0,
			spatial: None,
		}
	}

	pub fn spatial(clip_name: Handle, location: vec3, volume: f32) -> Self {
		Self {
			clip_name,
			volume,
			spatial: Some(Spatial { location }),
		}
	}
}
