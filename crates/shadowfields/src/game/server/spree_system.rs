//!
//! System for detecting killing sprees.
//!

use super::internal::*;

/// Need to score at least this fast to count towards killing sprees.
const SPREE_TIME: f32 = 3.0;

// TODO: spree system should not depend on auto_switch. Need global time + timers.
pub(crate) fn record_spree(state: &mut ServerState, player: ID) {
	//dbg!(&self.sprees);

     if let std::collections::hash_map::Entry::Vacant(e) = state.sprees.entry(player) {
         e.insert((state.autoswitch.time_playing, 1));
         return;
     }


	if let Some(&entry) = state.sprees.get(&player) {
		if (state.autoswitch.time_playing - entry.0) / (entry.1 as f32) < SPREE_TIME {
			let entry = entry.with(|e| e.1 += 1);
			let n = entry.1;
			state.sprees.insert(player, entry);
			hud_announce2(
				state,
				Just(player),
				match n {
					0 | 1 => "".into(),
					2 => "Double frag!".into(),
					3 => "Multi frag!".into(),
					4 => "Incredible frag!".into(),
					5 => "Unstoppable!!".into(),
					n => format!("{n} frags in a row!!!"),
				},
			);
			match n {
				0 | 1 => (), // unreachable!(),
				2 => sound_announce(state, Just(player), handle("ann_double_frag")),
				3 => sound_announce(state, Just(player), handle("ann_multi_frag")),
				4 => sound_announce(state, Just(player), handle("ann_incredible")),
				_ => sound_announce(state, Just(player), handle("ann_unstoppable")),
			}
		} else {
			state.sprees.remove(&player);
		}
	}
}
