use super::internal::*;

pub(crate) fn play_sound(state: &mut Client, sound: &SoundEffect) {
	match &sound.spatial {
		None => state.sound_pack.play_raw_volume(sound.clip_name, sound.volume),
		Some(spatial) => play_sound_spatial(state, sound.clip_name, sound.volume, &spatial),
	}
}

pub(crate) fn play_sound_spatial(state: &mut Client, clip_name: Handle, volume: f32, spatial: &Spatial) {
	// Sounds closer than this distance do not become any louder.
	// Otherwise very nearby sounds could become infinitely loud.

	const UNIT_DIST: f32 = 40.0;

	let player = state.local_player();
	let ear_pos = state.local_player().camera().position;
	let sound_pos = spatial.location;
	if (ear_pos - sound_pos).len() < 8.0 {
		// spatial audio does not work / is pointless when sound location is at or very near player location
		state.sound_pack.play_raw_volume(clip_name, volume.clamp(0.0, 1.0))
	} else {
		let azimuth = azimuth(&player.skeleton.filtered_frame(), sound_pos);
		let distance2 = (ear_pos - sound_pos).len2();
		let falloff_volume = (volume * (UNIT_DIST * UNIT_DIST) / distance2).clamp(0.0, 1.0);
		// muffle sound when obstructed by a wall
		let obstructed_volume = if is_obstructed(&state.map, ear_pos, sound_pos) { 0.3 * falloff_volume } else { falloff_volume };
		state.sound_pack.play_spatial(clip_name, azimuth, obstructed_volume)
	}
}


fn azimuth(frame: &Frame, sound_pos: vec3) -> f32 {
	let sound_dir = (sound_pos - frame.position).with(|v| v[Y] = 0.0).normalized();
	let look_dir = frame.orientation.look_dir().with(|v| v[Y] = 0.0).normalized();
	let sin_theta = look_dir.cross(sound_dir).y();
	let cos_theta = look_dir.dot(sound_dir);
	let azimuth = f32::atan2(sin_theta, cos_theta);
	if azimuth.is_nan() {
		0.0
	} else {
		azimuth
	}
}
