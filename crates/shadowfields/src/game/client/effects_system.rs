use super::internal::*;

/// Advance visual effects in time.
/// This is done locally (after creation,
/// visual effects do not need to synchronize over the network).
pub(crate) fn tick_effects(state: &mut Client) {
	let dt = state.dt();
	let effects = &mut state.effects;
	let mut i = 0;
	while i < effects.len() {
		effects[i].ttl -= dt;
		if effects[i].ttl <= 0.0 {
			effects.swap_remove(i);
		} else {
			i += 1;
		}
	}
}
