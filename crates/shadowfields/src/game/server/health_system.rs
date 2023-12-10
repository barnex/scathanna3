use super::internal::*;

// Per-player health, powerups, ...
pub(crate) struct Health {
	pub health: f32,
	pub shield: f32,

	pub target_shield: f32,
	pub shield_recharge_rate: f32,
	pub shield_discharge_rate: f32,
	pub invulnerability_ttl: Option<f32>,

	pub advantage: bool,
	// max shield, recharge rate, ...
}

impl Health {
	pub fn reset(&mut self) {
		self.health = 100.0;
		self.shield = 100.0;
		self.invulnerability_ttl = Some(1.0);
	}

	pub fn advantaged() -> Self {
		Self {
			target_shield: 100.0,
			shield_discharge_rate: 3.0,
			..Self::default()
		}
	}

	pub fn is_invulnerable(&self) -> bool {
		self.invulnerability_ttl.map(|ttl| ttl > 0.0).unwrap_or_default()
	}
}

impl Default for Health {
	fn default() -> Self {
		Self {
			health: 100.0,
			shield: 0.0,
			target_shield: 0.0,
			shield_recharge_rate: 10.0,
			shield_discharge_rate: 5.0,
			invulnerability_ttl: None,
			advantage: false,
		}
		.with(|s| s.reset())
	}
}

pub(crate) fn tick_health(state: &mut ServerState) {
	let dt = state.dt;
	for (_id, health) in &mut state.health {
		match &mut health.invulnerability_ttl {
			None => (),
			Some(ttl) if *ttl >= 0.0 => *ttl -= state.dt,
			Some(_) => health.invulnerability_ttl = None,
		}

		if health.shield < health.target_shield {
			health.shield += dt * health.shield_recharge_rate
		}

		if health.shield > health.target_shield {
			health.shield -= dt * health.shield_discharge_rate
		}

		if health.shield > 500.0{
			health.shield = 500.0; // max
		}
	}
}

// Handle a client saying they just shot a player.
// We trust clients not to lie about this.
//
// Hitting players is computed client-side for latency reasons:
// a client always sees other players at a location that lags slightly behind.
// If a client hits a player where they see them on their screen, then it should
// count as a hit regardless of latency.
// Otherwise players with more than about 30ms latency would be at a noticeable disadvantage.
pub(crate) fn handle_hit_player(state: &mut ServerState, actor: ID, victim: ID) {
	shoot_player(state, actor, victim, 100.0);
}

pub(crate) fn shoot_player(state: &mut ServerState, actor: ID, victim: ID, damage: f32) -> Option<()> {
	trace!("{actor} hit {victim}");

	let h = state.health.entry(victim).or_default();

	let mut damage = select(h.is_invulnerable(), 0.0, damage);

	if h.shield > 0.0 {
		let shield_damage = f32::min(damage, h.shield);
		h.shield -= shield_damage;
		damage -= shield_damage;
		if h.shield <= 0.0 {
			// shield break effect
		}
	}

	let health_damage = f32::min(h.health, damage);
	h.health -= health_damage;

	match h.health <= 0.1 {
		true => active_kill(state, actor, victim),
		false => apply_impulse(state, actor, victim),
	};

	Some(())
}

// actor shot victim without killing them. Push them instead.
fn apply_impulse(state: &mut ServerState, actor: ID, victim: ID) -> Option<()> {
	let pos1 = state.entities.players.get(&actor)?.position();
	let pos2 = state.entities.players.get(&victim)?.position();
	let dir = (pos2 - pos1).safe_normalized();
	// impulse is roughly in the shooting direction, but always points a bit up
	// else victims standing on the ground will immediately stop moving again.
	let impulse = (5.0 * dir).with(|v| v[1] = 1.0);
	state.diffs.push(ServerMsg::ApplyImpulse(impulse).to_just(victim));
	Some(())
}

pub(crate) fn pickup_shield(state: &mut ServerState, player_id: ID, collider_id: ID) -> Option<()> {
	let collider = state.entities.props.get(&collider_id)?;
	let location = collider.transform.translation;

	sound_effect(state, handle("xmas_hat"), location, 1.0);
	state.remove_game_object(collider_id);
	let powerups = state.health.entry(player_id).or_default();
	powerups.shield += 100.0;
	let shield = powerups.shield;
	let msg = format!("{:03.0}% shield", shield);
	hud_announce(state, Just(player_id), msg);

	Some(())
}

pub(crate) fn pickup_gift_box(state: &mut ServerState, player_id: ID, collider_id: ID) -> Option<()> {
	let collider = state.entities.props.get(&collider_id)?;
	let location = collider.transform.translation;

	sound_effect(state, handle("xmas_hat"), location, 1.0);
	state.remove_game_object(collider_id);
	//let powerups = state.health.entry(player_id).or_default();
	//powerups.shield += 100.0;
	//let msg = format!("{}% shield", powerups.shield);
	//hud_announce(state, Just(player_id), msg);

	Some(())
}
