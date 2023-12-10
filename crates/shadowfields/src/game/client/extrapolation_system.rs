use super::internal::*;

/// Extrapolate other player's positions based on their last know velocity.
/// This greatly reduces positional stutter in the face of network latency.
pub(crate) fn extrapolate_other_players(state: &mut Client) {
	let dt = state.dt();
	for (&id, player) in state.entities.players.iter_mut() {
		if id != state.local_player_id {
			player.skeleton.position += dt * player.skeleton.velocity;
		}
	}
}
