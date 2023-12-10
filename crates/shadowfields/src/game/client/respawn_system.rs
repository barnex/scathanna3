use super::internal::*;

pub(crate) fn control_respawn(state: &mut Client) {
	if state.inputs().just_pressed(Button::Mouse1) && !state.local_player().spawned {
		state.pending_diffs.push(ReadyToSpawn);
	}
}
