//!
//! Handle console commands by players.
//!

use super::internal::*;

pub(crate) fn server_command(state: &mut ServerState, client_id: ID, cmd: String) {
	info!("command from {client_id} ({}): '{cmd}'", must_name(state, client_id));
	match server_command_with_result(state, client_id, cmd) {
		Ok(()) => info!("command ok"),
		Err(e) => {
			info!("command error: {e:#}");
			state.diffs.push(Log(format!("error: {e:#}")).to_just(client_id));
		}
	}
}

fn server_command_with_result(state: &mut ServerState, client_id: ID, cmd: String) -> Result<()> {
	Ok(match &cmd.split_ascii_whitespace().collect::<Vec<_>>()[..] {
		["say", ..] => say(state, client_id, cmd["say".len()..].trim_start()),
		["switch"] => switch_next_map(state),
		["switch", map_name] => switch_map(state, map_name),
		["kill", victim_name] => kill_cmd(state, client_id, victim_name)?,
		["adv", player_name] => adv_cmd(state, player_name)?,
		["summon", handle] => summon_cmd(state, client_id, handle)?,
		["time_limit", seconds] => state.autoswitch.time_limit = seconds.parse()?,
		["frag_limit", frags] => state.autoswitch.frag_limit = frags.parse()?,
		_ => return Err(anyhow!("unknown command: {}", cmd)),
	})
}

fn summon_cmd(state: &mut ServerState, client_id: ID, handle: &str) -> Result<()> {
	const DIST_FROM_PLAYER: f32 = 2.0;
	let player = player(state, client_id).ok_or_else(noexist)?;
	let position = player.position() + DIST_FROM_PLAYER * player.skeleton.filtered_frame().orientation.look_dir_h();
	let mut gobj = Prop::from_str(handle)?;
	gobj.transform.translation = position;
	state.spawn_game_object(gobj);
	Ok(())
}

fn kill_cmd(state: &mut ServerState, player_id: ID, victim_name: &str) -> Result<()> {
	let victim_id = player_by_name(state, victim_name)?;
	active_kill(state, player_id, victim_id).ok_or(anyhow!("failed (maybe monad bailed out)"))
}

fn adv_cmd(state: &mut ServerState, player_name: &str) -> Result<()> {
	let player_id = player_by_name(state, player_name)?;
	log::info!("adv {player_name}");
	let health = state.health.entry(player_id).or_default();
	health.advantage = true;
	health.target_shield = 200.0;
	health.shield_discharge_rate = 3.0;
	state.diffs.push(ServerMsg::Command("adv".to_string()).to_just(player_id));
	Ok(())
}

fn say(state: &mut ServerState, player_id: ID, msg: &str) {
	let msg = format!("{}: {}", must_name(state, player_id), msg);
	state.diffs.push(Log(msg).to_all());
}

fn noexist() -> Error {
	anyhow!("entity ID does not exist")
}

pub(crate) fn player_by_name(state: &ServerState, player_name: &str) -> Result<ID> {
	state
		.entities
		.players
		.iter()
		.find(|(_, player)| player.name.eq_ignore_ascii_case(player_name))
		.map(|(id, _)| *id)
		.ok_or(anyhow!("no such player"))
}
