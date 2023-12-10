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
				if state.entities.players.get(&id)?.position().y() < KILL_PLANE {
					suicide(state, id, "fell off the world")?;
				}
			})
		})();
	}
}
