use super::internal::*;

pub(crate) fn tick_hud_text(state: &mut ServerState) {
	state.hud_timer.tick(state.dt);
	if state.hud_timer.just_finished() {
		for id in state.entities.player_ids() {
			state.diffs.push(
				UpdateHUD(HUDUpdate {
					pos: HUDPos::BottomLeft,
					text: hud_health_text(state, id),
					ttl_sec: 3.0,
				})
				.to_just(id),
			)
		}
	}
}

fn hud_health_text(state: &ServerState, id: ID) -> String {
	// no spawned
	if !state.entities.players.get(&id).map(|p| p.spawned).unwrap_or_default() {
		return String::new(); // 👈
	}

	if let Some(h) = state.health.get(&id) {
		let invuln = select(h.is_invulnerable(), "*", " ");
		let adv = select(h.advantage, "+", " ");
		format!("{adv}{FONT_SHIELD}{:03.0}\n{invuln}{FONT_HEART}{:03.0}", h.shield, h.health)
	} else {
		String::new()
	}
}

/// Announce message to a player's HUD.
/// E.g. "You fragged Foo"
pub(crate) fn hud_announce(state: &mut ServerState, to: Addressee, msg: impl Into<String>) {
	state.diffs.push(
		UpdateHUD(HUDUpdate {
			pos: HUDPos::TopCenter,
			text: msg.into(),
			ttl_sec: ANNOUNCE_TTL,
		})
		.to(to),
	);
}

/// Announce message to a player's HUD, line2.
pub(crate) fn hud_announce2(state: &mut ServerState, to: Addressee, msg: impl Into<String>) {
	state.diffs.push(
		UpdateHUD(HUDUpdate {
			pos: HUDPos::TopCenter2,
			text: msg.into(),
			ttl_sec: ANNOUNCE_TTL,
		})
		.to(to),
	);
}
