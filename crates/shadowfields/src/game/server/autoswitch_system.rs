//!
//! Automatically switch maps when a frag or time limit is reached.
//!

use super::internal::*;

pub(crate) struct AutoSwitch {
	pub frag_limit: i32,
	pub time_playing: f32,
	pub time_limit: f32,
}

impl AutoSwitch {
	pub fn from_settings(settings: &ServerOpts) -> Self {
		Self {
			frag_limit: settings.frag_limit as i32,
			time_limit: settings.time_limit as f32,
			time_playing: 0.0,
		}
	}

	pub fn time_remaining(&self) -> f32 {
		self.time_limit - self.time_playing
	}
}

pub(crate) fn tick_autoswitch(state: &mut ServerState) {
	tick_time_remaining(state);
	tick_next_game(state);
}

fn tick_time_remaining(state: &mut ServerState) {
	let dt = state.dt;
	state.autoswitch.time_playing += dt;

	if (state.autoswitch.time_playing + dt) as i32 != state.autoswitch.time_playing as i32 {
		// broadcast every second so the timer ticks down
		broadcast_scores(state);
	}
	// 1 minute warning
	if (state.autoswitch.time_remaining() + dt) > 60.00 && state.autoswitch.time_remaining() <= 60.0 {
		trace!("1 minute warning");
		sound_announce(state, All, handle("ann_1_min_warn"));
	}
}

fn tick_next_game(state: &mut ServerState) {
	if state.autoswitch.time_playing > state.autoswitch.time_limit {
		switch_next_map(state);
	}

	if state.scores.max() >= state.autoswitch.frag_limit {
		switch_next_map(state);
	}
}

pub(crate) fn announce_remaining_frags(state: &mut ServerState) {
	let top_score = state.scores.max();
	if top_score == (state.autoswitch.frag_limit) - 1 {}

	let remaining = state.autoswitch.frag_limit - top_score;
	info!("{remaining} frag(s) remaining");
	match remaining {
		1 => sound_announce(state, All, handle("ann_1_frag")),
		2 => sound_announce(state, All, handle("ann_2_frags")),
		3 => sound_announce(state, All, handle("ann_3_frags")),
		_ => (),
	}
}
