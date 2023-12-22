use crate::prelude::*;

pub(crate) fn exec_command(state: &mut Client, cmd: &str) -> Result<()> {
	match &cmd.split_ascii_whitespace().collect::<Vec<_>>()[..] {
		["head_size", v] => set("player.head_size.y", &mut state.local_player_mut().head_size[1], v),
		["torso_size", v] => set("player.torso_size.y", &mut state.local_player_mut().torso_size[1], v),

		["ms" | "mouse_sensitivity", v] => set("mouse_sensitivity", &mut state.settings.controls.mouse_sensitivity, v),
		["stutter_filter", v] => set("stutter_filter", &mut state.mouse_filter.stutter_filter, v),
		["mouse_smoothing", v] => set("mouse_smoothing", &mut state.mouse_filter.smoothing, v),

		["timepassage", v] => Ok(state.settings.debug.time_passage = v.parse()?),
		["fly"] => flip(&mut state.local_player_mut().flying),

		["bot_overlay"] => flip(&mut state.debug.bot_overlay),
		["dbg_overlay" | "dbg"] => flip(&mut state.debug.dbg_overlay),
		["fps_overlay" | "fps"] => flip(&mut state.debug.fps_overlay),
		["ecs_overlay" | "ecs"] => flip(&mut state.debug.ecs_overlay),
		["bb_overlay" | "bb"] => flip(&mut state.debug.bb_overlay),

		["g" | "gravity", g] => set("gravity", &mut state.map.gravity, g),
		["axes"] => flip(&mut state.debug.axes),
		["adv"] => Ok(advantage(state)),
		// unknown command gets forwarded to server, who might know what to do
		_ => Ok(state.pending_diffs.push(ClientMsg::Command(cmd.into()))),
	}
}

/// `*dst = val.parse()`, with logging.
fn set<T>(label: &str, dst: &mut T, val: &str) -> Result<()>
where
	T: std::fmt::Display,
	T: FromStr,
	T: Clone,
	<T as std::str::FromStr>::Err: std::fmt::Debug,
{
	match val.parse() {
		Ok(v) => {
			let prev = dst.clone();
			*dst = v;
			LOG.write(format!("set {label}: {prev} -> {}", *dst));
			Ok(())
		}
		Err(e) => Err(anyhow!("set {label}: {val}: {e:?}")),
	}
}

pub(crate) fn exec_server_command(state: &mut Client, cmd: &str) {
	match exec_command(state, cmd) {
		Ok(()) => LOG.write(format!("server command: {cmd}: OK")),
		Err(e) => LOG.write(format!("server command: {cmd}: {e:#}")),
	}
}

fn flip(v: &mut bool) -> Result<()> {
	*v = !*v;
	Ok(())
}
