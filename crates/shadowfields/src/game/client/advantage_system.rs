use crate::prelude::*;

pub(crate) fn advantage(state: &mut Client) {
	state.weapon_state.cooldown_secs = 0.35;
	state.weapon_state.automatic = true;
}
