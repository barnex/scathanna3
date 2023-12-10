use super::internal::*;

pub(crate) fn jump_pad_system(state: &mut Client) {
	//let mut clone = state.local_player().clone();
	//control(state, &mut clone);
	//*state.local_player_mut() = clone;

	let player_bounds = state.local_player().skeleton.bounds();

	for pad in &state.map.jump_pads {
		if player_bounds.overlaps(&pad.bounds) {
			state.local_player_mut().skeleton.velocity = vec3(0.0, jump_velocity_for_height(state.map.gravity, pad.jump_height), 0.0);
			break;
		}
	}
}

