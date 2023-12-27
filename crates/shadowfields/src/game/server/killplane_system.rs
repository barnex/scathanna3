//!
//! Killplane system kills players who fall off the world.
//!
use super::internal::*;

// kill players who fell off the world
pub(crate) fn tick_killplane(state: &mut ServerState) {
	const KILL_PLANE: f32 = -64.0;
	for id in state.entities.spawned_player_ids() {
		(|| {
			Some({
				// ! must use target_position, as filtered_position lags a bit behind
				// ! causing an infinite kill loop.
				if state.entities.players.get(&id)?.skeleton.target_position.y() < KILL_PLANE {
					state.entities.players.get_mut(&id)?.skeleton.target_position[1]+=1.0; // HACK: avoid being killed again before proper respawn.
					suicide(state, id, "fell off the world")?;
				}
			})
		})();
	}
}
