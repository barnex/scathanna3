use super::internal::*;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct WeaponState {
	pub cooldown_timer: f32,
	pub cooldown_secs: f32,
	pub armed: bool,
	pub automatic: bool,
}

impl Default for WeaponState {
	fn default() -> Self {
		Self {
			cooldown_timer: default(),
			cooldown_secs: 0.7, /* like instagib */
			armed: true,
			automatic: false,
		}
	}
}

pub(crate) fn control_shooting(state: &mut Client) {
	let mut clone = state.local_player().clone();
	control_shooting_(state, &mut clone);
	*state.local_player_mut() = clone;
}

// TODO: shoot_system + movement_system
pub(crate) fn control_shooting_(state: &mut Client, player: &mut Player) {
	if !player.spawned {
		return;
	}

	// Semi-auto shooting
	// Pressing too early will shoot as soon as ready,
	// but next shot requires a trigger release first.
	let dt = state.dt();
	let weapon_state = &mut state.weapon_state;
	weapon_state.cooldown_timer -= dt;
	if weapon_state.automatic || state._win.inputs.just_pressed(Button::Mouse1) {
		weapon_state.armed = true;
	}
	if weapon_state.armed && weapon_state.cooldown_timer <= 0.0 && state._win.inputs.was_pressed(Button::Mouse1) {
		weapon_state.cooldown_timer = weapon_state.cooldown_secs;
		weapon_state.armed = false;
		shoot(state, player);
	}
}

fn shoot(state: &mut Client, player: &Player) {
	let line_of_fire = line_of_fire(state, player);
	let start = line_of_fire.start.to_f32();
	let end = crosshair_target(state, player);
	let delta = end - start;
	let len = delta.len();
	let dir = delta.safe_normalized();
	let orientation = Orientation::from_look_dir(dir);

	let texture = match player.team {
		Team::Red => handle("star_red"),
		Team::Blue => handle("star_blue"),
		Team::Green => handle("star_green"),
	};
	state.pending_diffs.push(ClientMsg::AddEffect(Effect::particle_beam(start, orientation, len, texture)));

	state.pending_diffs.push(ClientMsg::PlaySound(SoundEffect::spatial(
		must_pick_random(&[handle("bang1"), handle("bang2"), handle("bang3"), handle("bang4")]),
		player._center(),
		30.0,
	)));
	state.pending_diffs.push(ClientMsg::PlaySound(SoundEffect::spatial(
		must_pick_random(&[handle("ricochet1"), handle("ricochet2"), handle("ricochet3"), handle("ricochet4")]),
		end,
		1.0,
	)));

	if let Some((_, Some(victim_id))) = state.intersect_except_player(player.id, &line_of_fire) {
		// Hit a player
		//state.pending_diffs.push(ClientMsg::PlaySound(SoundEffect::spatial(handle("kill"), end, 1.0))); << must be in server
		state.pending_diffs.push(HitPlayer(victim_id));
	} else {
		let hr = state.map.intersect(&line_of_fire.convert());
		if let Some((normal, _, material)) = hr.attrib {
			// Hit a wall: debris effect
			let pos = line_of_fire.convert::<f32>().at(hr.t);
			let texture = state.map.material_palette.get(&material).map(|mat| mat.base_color).unwrap_or(handle("#666666"));
			state.pending_diffs.push(ClientMsg::AddEffect(Effect::debris(pos, normal, texture)));
		}
	}
}

/// Ray from the player's gun nozzle to where the player is looking.
/// I.e., the trajectory a bullet would follow.
fn line_of_fire(state: &Client, player: &Player) -> Ray64 {
	let start = gun_center(player);
	let look_at = look_at(state, player);
	let shoot_from_gun = Ray64::new(start.into(), (look_at - start).normalized().into());

	// Because of parallax between the nozzle and camera position,
	// an object can sometimes be in front of the gun but not in front of the camera.
	// This can lead to seemingly inexplicably missed shots.
	//
	// Many games have this behavior when shooting from the hip.
	// However, here it is particularly severe as the gun is quite far from the camera.
	// Therefore, when an object blocks the line of fire but not the line of sight,
	// shoot from the "eye" rather than from the hip so that the shot is not missed.
	let shoot_at = state.intersect_except_player(player.id, &shoot_from_gun).map(|(t, _)| shoot_from_gun.at(t).to_f32());
	if let Some(shoot_at) = shoot_at {
		if (shoot_at - look_at).len() > 2.0 {
			return player.line_of_sight();
		}
	}

	shoot_from_gun
}

/// (Absolute) position of the player's gun.
/// Note: this is not necessarily the position where a bullet starts,
/// use `line_of_fire().start` for that.
fn gun_center(player: &Player) -> vec3 {
	let gun_internal = gun_pos_internal(player);
	player.position() + player.skeleton.orientation.look_right() * gun_internal.x() + gun_internal.y() * vec3::EY
}

/// Position the user is looking at.
/// If looking at the (infinitely far) sky,
/// this returns a far-away point in the looking direction.
fn look_at(state: &Client, player: &Player) -> vec3 {
	let line_of_sight = player.line_of_sight();
	state
		.intersect_except_player(player.id, &line_of_sight)
		.map(|(t, _)| line_of_sight.at(t))
		.unwrap_or(player.camera().position.to_f64() + 10000.0 * player.orientation().look_dir().to_f64())
		.into()
}

/// (Absolute) position where the player's gun would hit if it fired.
fn crosshair_target(state: &Client, player: &Player) -> vec3 {
	let line_of_fire = line_of_fire(state, player);
	line_of_fire
		.at(state.intersect_except_player(player.id, &line_of_fire).map(|(t, _)| t + 0.01).unwrap_or(10000.0))
		.into()
}

pub(crate) fn gun_pos_internal(player: &Player) -> vec3 {
	vec3(player.torso_size.x(), 0.5 * player.torso_size.y(), 0.0)
}

/// Intersect a ray (e.g. a line of sight) with the map and players except `player_id`
/// (to avoid shooting yourself right where the line of fire exits your hitbox).
/// Returns intersection distance along the ray
/// and  the ID of the nearest hit player, if any.
pub(crate) fn _intersect_except_player((map, entities): (&Map, &Entities), player_id: ID, ray: &Ray64) -> Option<(f64, Option<ID>)> {
	let intersect_map = map.intersect_t_64(ray);
	let mut nearest = intersect_map.map(|t| (t, None));
	for (&id, player) in entities.players.iter() {
		if let Some(t) = player.intersect(ray) {
			if t < nearest.map(|(t, _)| t).unwrap_or(f64::INFINITY) && id != player_id {
				nearest = Some((t, Some(id)));
			}
		}
	}
	nearest
}
