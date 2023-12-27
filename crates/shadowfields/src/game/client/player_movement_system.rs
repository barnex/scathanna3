use super::internal::*;

pub(crate) fn smooth_players_movement(players: &mut HashMap<ID, Player>) {
	let a = 0.5;
	for player in players.values_mut() {
		player.skeleton.pre_filtered_position = a * player.skeleton.pre_filtered_position + (1.0 - a) * player.skeleton.target_position;
		player.skeleton.filtered_position = a * player.skeleton.filtered_position + (1.0 - a) * player.skeleton.pre_filtered_position;
	}
}

/// Control a player via keyboard/mouse
pub(crate) fn control_player_movement(state: &mut Client) {
	state.jump_sound_cooldown.tick(state.dt());
	let mut clone = state.local_player().clone();
	control(state, &mut clone);
	*state.local_player_mut() = clone;
}

/// Record a diff for controlling this player with keyboard/mouse input.
/// Called on a clone of the World's player (!so need to be careful for self-interaction!).
fn control(state: &mut Client, player: &mut Player) {
	state.mouse_filter.record_event(state.inputs()._mouse_delta);
	let mouse_delta = state.mouse_filter.tick();
	let mouse_sens = 0.00001 * state.settings.controls.mouse_sensitivity;
	player.skeleton.orientation.yaw = wrap_angle(player.skeleton.orientation.yaw - mouse_delta.x() * mouse_sens);
	player.skeleton.orientation.pitch = (player.skeleton.orientation.pitch + mouse_delta.y() * mouse_sens).clamp(-89.0 * DEG, 89.0 * DEG);
	if player.spawned {
		control_movement(state, player);
		state.pending_diffs.push(MovePlayerIfSpawned(player.skeleton.filtered_frame()));
	}
}

// __________________ movement

fn control_movement(state: &mut Client, player: &mut Player) {
	if player.flying {
		tick_fly(state, player);
		tick_jump(state, player);
	} else {
		tick_walk(state, player);
		tick_jump(state, player);
		tick_gravity(state, player);
	}

	tick_move_skel(state, player);
	tick_rescue(state, player);
}

fn tick_walk(state: &mut Client, player: &mut Player) {
	let walk_speed = player.walk_speed * walk_dir(player.orientation().yaw, &state.inputs());
	try_walk(player, state.dt(), &state.map, walk_speed);
}

fn tick_fly(state: &mut Client, player: &mut Player) {
	let walk_speed = player.walk_speed * fly_dir(player.orientation().yaw, &state.inputs());
	try_walk(player, state.dt(), &state.map, walk_speed);
}

// attempt to change velocity by walking / flying
pub(crate) fn try_walk(player: &mut Player, dt: f32, map: &Map, walk_speed: vec3) {
	let max_airctl_speed: f32 = player.walk_speed;
	const AIRCTL_ACCEL: f32 = 2.0;

	if player.on_ground(map) {
		player.skeleton.velocity[X] = walk_speed[X];
		player.skeleton.velocity[Z] = walk_speed[Z];
	} else {
		// flying through the air

		// always slightly damp movement
		let damp = 0.1;
		player.skeleton.velocity *= 1.0 - damp * dt;

		// allow to control movement in the air a bit.
		if player.skeleton.velocity.remove(1).len() > max_airctl_speed {
			// flying too fast, damp aggressively
			player.skeleton.velocity *= 1.0 - 4.0 * damp * dt;
		} else {
			// flying not too fast, allow some slow control
			player.skeleton.velocity += (AIRCTL_ACCEL * dt) * walk_speed;
		}
	}
}

pub(crate) fn try_jump(player: &mut Player, world: (&Map, &Entities), jump_speed: f32) -> bool {
	if player.on_ground(world.0) {
		unconditional_jump(player, jump_speed);
		true
	} else {
		false
	}
}

pub(crate) fn unconditional_jump(player: &mut Player, jump_speed: f32) {
	player.skeleton.velocity[Y] = jump_speed
}

fn tick_jump(state: &mut Client, player: &mut Player) {
	if state.inputs().is_down(Button::Jump) {
		if try_jump(player, state.world(), player.jump_speed) {
			if state.jump_sound_cooldown.is_idle() {
				state.pending_diffs.push(ClientMsg::PlaySound(SoundEffect::spatial(handle("jump"), player.position(), 0.3)));
				state.jump_sound_cooldown.reset();
			}
		}
	}
}

fn tick_gravity(state: &Client, player: &mut Player) {
	let dt = state.dt();
	let g = state.map.gravity;
	player.skeleton.velocity[Y] -= g * dt;
	let damp = 0.05;
	player.skeleton.velocity *= 1.0 - damp * dt;
}

fn tick_move_skel(state: &Client, player: &mut Player) {
	// 👇 Hack because velocity can become NaN.
	if !player.skeleton.velocity.is_finite() {
		player.skeleton.velocity = vec3::ZERO;
	}
	debug_assert!(player.skeleton.velocity.is_finite());

	let dt = state.dt();
	debug_assert!(dt != 0.0);

	let map = &state.map;

	const NUM_SUBSTEPS: usize = 16;

	let h_dir = player.skeleton.velocity.with(|v| v[Y] = 0.0).safe_normalized();

	player.bump = Vector3::new(false, false, false);

	for _i in 0..NUM_SUBSTEPS {
		let delta_pos = player.skeleton.velocity * dt;
		let sub_delta_pos = delta_pos / (NUM_SUBSTEPS as f32);
		let dx = vec3(sub_delta_pos.x(), 0.0, 0.0);
		let dy = vec3(0.0, sub_delta_pos.y(), 0.0);
		let dz = vec3(0.0, 0.0, sub_delta_pos.z());

		if player.pos_ok(map, player.skeleton.target_position + dx) {
			player.skeleton.target_position += dx;
		} else {
			player.bump[X] = true;
		}

		if player.pos_ok(map, player.skeleton.target_position + dz) {
			player.skeleton.target_position += dz;
		} else {
			player.bump[Z] = true;
		}

		if player.pos_ok(map, player.skeleton.target_position + dy) {
			player.skeleton.target_position += dy;
		} else {
			player.bump[Y] = true;
		}

		if player.bump.x() || player.bump.z() {
			let climb_speed = player.walk_speed;
			const STAIR_W: f32 = 0.5;
			const STAIR_H: f32 = 0.6;
			let step_up = STAIR_W * h_dir + STAIR_H * vec3::EY;
			let tiny = climb_speed * dt / (NUM_SUBSTEPS as f32) * vec3::EY;
			if player.pos_ok(map, player.skeleton.target_position + step_up) {
				if player.pos_ok(map, player.skeleton.target_position + tiny) {
					player.skeleton.target_position += tiny
				}
			}
		}
	}

	if player.on_ground(map) {
		player.skeleton.velocity[Y] = 0.0; // <<<<<<<<<<<<<<<<<<<<< ????
	}
}

fn tick_rescue(state: &Client, player: &mut Player) {
	let stairclimb_speed = player.walk_speed * 1.2; // TODO
	if !player.pos_ok(&state.map, player.skeleton.target_position) {
		player.skeleton.target_position[Y] += stairclimb_speed * state.dt();
	}
}

/// Direction an entity wants to move in,
/// based on the currently pressed keys and look direction.
fn walk_dir(yaw: f32, inputs: &Inputs) -> vec3 {
	let mut dir = vec3::ZERO;
	if inputs.is_down(Button::Left) {
		dir[X] -= 1.0;
	}
	if inputs.is_down(Button::Right) {
		dir[X] += 1.0;
	}
	if inputs.is_down(Button::Forward) {
		dir[Z] -= 1.0;
	}
	if inputs.is_down(Button::Backward) {
		dir[Z] += 1.0;
	}
	if dir == vec3::ZERO {
		return vec3::ZERO;
	}
	let dir = -yaw_matrix(-yaw).transform_point_ignore_w(dir);
	dir.safe_normalized()
}

/// Direction an entity wants to fly in,
/// based on the currently pressed keys and look direction.
fn fly_dir(yaw: f32, inputs: &Inputs) -> vec3 {
	let mut fly_dir = walk_dir(yaw, inputs);
	if inputs.is_down(Button::Jump) {
		fly_dir[Y] += 1.0;
	}
	if inputs.is_down(Button::Crouch) {
		fly_dir[Y] -= 1.0;
	}
	fly_dir.safe_normalized()
}
