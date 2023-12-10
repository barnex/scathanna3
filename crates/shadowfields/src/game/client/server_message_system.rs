use super::internal::*;

pub(crate) fn apply_all_server_messages(state: &mut Client) -> Result<()> {
	while let Some(msg) = state.conn.try_recv() {
		apply_server_msg(state, msg?)?;
	}
	Ok(())
}

/// Apply a diff to the game state.
fn apply_server_msg(state: &mut Client, msg: ServerMsg) -> Result<()> {
	use ServerMsg::*;
	match msg {
		AddPlayer(player) => handle_add_player(state, player),
		DropPlayer(player_id) => handle_drop_player(state, player_id),
		MovePlayer(player_id, frame) => handle_move_player(state, player_id, frame),
		ForceMovePlayer(position) => handle_force_move_player(state, position),
		UpdatePlayerPartial(player) => handle_update_player_partial(state, player),
		UpdatePlayerFull(player) => handle_update_player_full(state, player),
		UpdateHUD(update) => handle_update_hud(state, update),
		AddEffect(effect) => handle_add_effect(state, effect),
		PlaySound(sound_effect) => play_sound(state, &sound_effect),
		SwitchMap(map_switch) => switch_map(state, map_switch)?,
		InsertEntity(id, gameobj) => handle_insert_object(state, id, gameobj),
		UpdateEntity(id, diff) => handle_update_object(state, id, diff),
		RemoveObject(id) => handle_remove_object(state, id),
		Log(msg) => LOG.write(msg),
		Command(cmd) => exec_server_command(state, &cmd),
		ApplyImpulse(delta_v) => handle_apply_impulse(state, delta_v),
	};
	Ok(())
}

// todo: all "handle" methods here

fn handle_add_player(state: &mut Client, player: Player) {
	state.entities.players.insert(player.id, player);
}

fn handle_drop_player(state: &mut Client, player_id: ID) {
	LOG.write(format!("dropping player {player_id}"));
	state.entities.players.remove(&player_id);
}

fn handle_move_player(state: &mut Client, player_id: ID, frame: Frame) {
	if let Some(p) = state.entities.players.get_mut(&player_id) {
		p.skeleton.set_frame(frame)
	}
}

fn handle_apply_impulse(state: &mut Client, delta_v: vec3) {
	state.local_player_mut().skeleton.velocity += delta_v;
	state.local_player_mut().skeleton.position[1] += 0.1; // 👈 tiny jump to get you airborne
}

fn handle_force_move_player(state: &mut Client, position: vec3) {
	state.local_player_mut().skeleton.position = position;
}

// Update part of player state controlled by server: everything except frame.
// Server sends frame anyway (for simplicity), but this is ignored here.
fn handle_update_player_partial(state: &mut Client, new: Player) {
	if let Some(old) = state.entities.players.get_mut(&new.id) {
		let mut new = new;
		new.skeleton.set_frame(old.skeleton.frame());
		*old = new;
	}
}

// Update the entire player (including frame).
// Server will only ask this when de-spawned.
fn handle_update_player_full(state: &mut Client, new: Player) {
	if let Some(player_mut) = state.entities.players.get_mut(&new.id) {
		//debug_assert!(!player_mut.spawned);
		*player_mut = new
	}
}

fn handle_insert_object(state: &mut Client, id: ID, gobj: Prop) {
	state.entities.insert_prop(id, gobj);
}

fn handle_update_object(state: &mut Client, id: ID, diff: PropDiff) {
	state.entities.apply_to_prop(id, diff);
}

fn handle_remove_object(state: &mut Client, id: ID) {
	state.entities.remove_prop(id);
}

// TODO: private
pub(crate) fn handle_add_effect(state: &mut Client, effect: Effect) {
	state.effects.push(effect);
}
fn handle_update_hud(state: &mut Client, upd: HUDUpdate) {
	state.hud.apply(upd);
}
