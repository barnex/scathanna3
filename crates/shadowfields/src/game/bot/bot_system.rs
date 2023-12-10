use crate::prelude::*;
use ClientMsg::*;

// Serialize is for debug overlay only.
#[derive(Serialize)]
pub(crate) struct Bot {
	#[serde(skip)]
	settings: BotSettings,

	#[serde(skip)]
	rng: Xoshiro256PlusPlus,

	// ✅ Respawn after some time
	#[serde(skip)]
	respawn_timer: Timer,

	// Strategic
	engaging_enemy_id: Option<ID>, // ✅
	waypoint: Option<vec3>,        // ✅ TODO: enemy leaves breadcrumbs on ground
	waypoint_info: &'static str,   // for debug only
	waypoint_ttl: Timer,
	aimpoint: Option<vec3>,
	aimpoint_info: &'static str, // for debug only
	randomize_timer: Timer,

	// Executive
	target_orientation: Option<Orientation>,
	delta_orientation: Option<Orientation>,
	victim_in_crosshairs: bool,
	shoot_timer: Timer,
	jump_timer: Timer,
}

pub(crate) async fn run_bot_gameloop(state: &mut Client) -> Result<()> {
	let mut bot = Bot::new(&state.settings.bot);
	loop {
		Client::bot_pre_tick(state, &bot).await?;

		bot.tick(state).await;

		if state.settings.debug.bot_overlay {
			state.hud.set_text(HUDPos::Debug, &fmt_bot_overlay(&bot, state), 1.0);
		}
	}
}

impl Bot {
	pub fn new(settings: &BotSettings) -> Self {
		Self {
			settings: settings.clone(),
			rng: Xoshiro256PlusPlus::seed_from_u64(1),
			respawn_timer: Timer::one_off(settings.respawn_time),
			shoot_timer: Timer::unarmed(),
			victim_in_crosshairs: false,
			engaging_enemy_id: None,
			target_orientation: None,
			delta_orientation: None,
			waypoint: None,
			waypoint_info: "",
			waypoint_ttl: Timer::unarmed(),
			aimpoint: None,
			aimpoint_info: "",
			jump_timer: Timer::one_off(1.0),
			randomize_timer: Timer::one_off(1.0),
		}
	}

	pub async fn tick(&mut self, state: &mut Client) {
		self.respawn(state);
		if state.local_player().spawned {
			// cleanup to start from consistent state
			self.clear_dead_waypoint(state); // ✅
			self.clear_reached_waypoint(state); // ✅
			self.clear_expired_waypoint(state); //

			// strategic: what to do
			self.find_enemy(state); // ✅
			self.find_waypoint(state); // ❌ TODO: searching
			self.find_aimpoint(state);
			self.find_random_waypoint(state);

			//cannot see waypoint: move sideways, then forget if not reachable
			//runs in circles

			// executive: try to do it
			self.execute_aim(state);
			self.execute_shooting(state);
			self.execute_moving(state);
			self.random_jump(state);
		}
	}

	// -----------------------------  cleanup

	/// Remove waypoint if enemy has just died.
	/// Hack to avoid chasing ghosts.
	fn clear_dead_waypoint(&mut self, state: &Client) -> Option<()> {
		if state.entities.players.get(&self.engaging_enemy_id?)?.spawned == false {
			self.waypoint = None;
			self.waypoint_info = "dead enemy";
			self.engaging_enemy_id = None;
		}
		Some(())
	}

	/// Clear current waypoint if reached (approximately),
	/// so that we can move on to the next.
	fn clear_reached_waypoint(&mut self, state: &Client) -> Option<()> {
		let waypoint = self.waypoint?;
		let my_pos = state.local_player().position();
		// 💀 Reach waypoint ignoring vertical distance.
		// High-up waypoints (left by jumping target enemy) may be hard to reach.
		// BUG: cannot pickup high up shields!
		const WAYPOINT_TOLERANCE: f32 = 1.0; // m
		if my_pos.xz().distance_to(waypoint.xz()) < WAYPOINT_TOLERANCE {
			self.waypoint = None;
			self.waypoint_info = "reached";
		}
		Some(())
	}

	fn clear_expired_waypoint(&mut self, state: &Client) -> Option<()> {
		const WAYPOINT_TTL: f32 = 3.0; //s
		match can_see(&state.map, state.local_player(), self.waypoint?) {
			true => self.waypoint_ttl = Timer::one_off(WAYPOINT_TTL),
			false => {
				if self.waypoint_ttl.tick(state.dt()).just_finished() {
					self.waypoint = None;
					self.waypoint_info = "expired";
				}
			}
		}

		Some(())
	}

	// -----------------------------  strategic

	/// Find an enemy to engage, set enemy_id.
	fn find_enemy(&mut self, state: &Client) {
		// Pick closest, visible, spawned enemy.
		let me = state.local_player();
		self.engaging_enemy_id = state
			.entities
			.players
			.values()
			.filter(|p| p.spawned && p.id != me.id && p.team != me.team) // 👈 spawned enemy
			.filter(|p| can_see(&state.map, me, p.center())) // 👈 visible
			.min_by_key(|p| me.center().distance_to(p.center()) as i32) // 👈 closest
			.map(|p| p.id);
	}

	fn find_waypoint(&mut self, state: &Client) {
		// Remove current waypoint if reached
		self.clear_reached_waypoint(state);

		// Priority 1: set waypoint to where an enemy was last seen
		// ! even if there's already a waypoint: keep chasing the moving player
		self.find_enemy_waypoint(state);

		// Priority 2: set waypoint to a nearby pickup
		if self.waypoint.is_none() {
			self.find_pickup_waypoint(state);
		}
	}

	fn find_random_waypoint(&mut self, state: &mut Client) {
		if self.waypoint.is_none() {
			self.randomize_waypoint(state);
		}
		if self.randomize_timer.tick(state.dt()).just_finished() {
			self.randomize_timer.set(10.0);
			self.randomize_waypoint(state);
		}
	}

	fn randomize_waypoint(&mut self, state: &mut Client) -> Option<()> {
		let me = state.local_player();

		let mut candidates = SmallVec::<[_; 5]>::new();

		for _i in 0..5 {
			let dir = me.skeleton.orientation.with(|o| o.pitch = 0.0).with(|o| o.yaw += self.rng.gen_range(-PI..PI)).look_dir();
			let ray = Ray32::new(me.camera().position, dir);
			let t = state.map.intersect_t(&ray)?;
			if t > 2.0 {
				let dst = ray.at(t / 2.0 - 0.5);
				candidates.push((t, dst));
			}
		}
		let best = candidates.into_iter().max_by_key(|(t, _dst)| *t as i32);
		self.waypoint = best.map(|(_, dst)| dst);
		self.waypoint_info = "random";
		Some(())
	}

	fn random_jump(&mut self, state: &mut Client) {
		if self.jump_timer.tick(state.dt()).just_finished() {
			state.inputs_mut().press_button(Button::Jump);
			state.inputs_mut().release_button(Button::Jump);
			self.jump_timer.set(self.settings.jump_period * 2.0 * self.rng.gen::<f32>());
		}
	}

	fn find_enemy_waypoint(&mut self, state: &Client) -> Option<()> {
		// TODO: clear if target player killed
		let dst = state.entities.players.get(&self.engaging_enemy_id?)?.center();
		const PERSONAL_SPACE: f32 = 2.0;
		if state.local_player().center().distance_to(dst) > PERSONAL_SPACE {
			self.waypoint = Some(dst);
			self.waypoint_info = "target player";
		}
		Some(())
	}

	fn find_pickup_waypoint(&mut self, state: &Client) -> Option<()> {
		let me = state.local_player();
		self.waypoint = state
			.entities
			.props
			.values()
			.filter(|prop| can_see(&state.map, me, prop.center()))
			.min_by_key(|prop| me.center().distance_to(prop.center()) as i32)
			.map(|prop| prop.center());
		if self.waypoint.is_some() {
			self.waypoint_info = "pickup";
		}
		Some(())
	}

	fn find_aimpoint(&mut self, state: &Client) {
		self.aimpoint = None;
		self.aimpoint_info = "";
		self.find_enemy_aimpoint(state);
		if self.aimpoint.is_none() {
			self.find_waypoint_aimpoint(state);
		}
	}

	fn find_enemy_aimpoint(&mut self, state: &Client) {
		if let Some(id) = self.engaging_enemy_id {
			if let Some(enemy) = state.entities.players.get(&id) {
				let enemy_pos = enemy.center();
				if can_see(&state.map, state.local_player(), enemy_pos) {
					self.aimpoint = Some(enemy_pos);
					self.aimpoint_info = "enemy";
				}
			}
		}
	}

	fn find_waypoint_aimpoint(&mut self, state: &Client) {
		// 💀 there should always be an aim point, even if we can't see it
		// TODO: if we can't see it: move sideways or jump
		self.aimpoint = self.waypoint;
	}

	fn execute_aim(&mut self, state: &mut Client) -> Option<()> {
		self.aim_at(state, self.aimpoint?);
		Some(())
	}

	fn aim_at(&mut self, state: &mut Client, target_pos: vec3) {
		self.target_orientation = None;
		self.delta_orientation = None;

		let me = state.local_player();
		let my_camera = me.camera();
		let my_orientation = my_camera.orientation;

		let target_dir = (target_pos - my_camera.position).safe_normalized();
		let target_orientation = Orientation::from_look_dir(target_dir);
		self.target_orientation = Some(target_orientation);
		let mut delta_yaw = target_orientation.yaw - my_orientation.yaw;
		if delta_yaw > PI {
			delta_yaw -= 2.0 * PI;
		} else if delta_yaw < -PI {
			delta_yaw += 2.0 * PI;
		}
		let delta_pitch = target_orientation.pitch - my_orientation.pitch;
		self.delta_orientation = Some(Orientation { yaw: delta_yaw, pitch: delta_pitch });

		let dt = state.dt();
		let aim_speed = self.settings.aim_speed;

		let me = state.local_player_mut();
		//                                     👇 TODO: proper aiming function with inertia
		me.skeleton.orientation.pitch += dt * (4.0 * aim_speed * delta_pitch).clamp(-aim_speed, aim_speed);
		me.skeleton.orientation.yaw += dt * (4.0 * aim_speed * delta_yaw).clamp(-aim_speed, aim_speed);
	}

	fn execute_shooting(&mut self, state: &mut Client) {
		self.victim_in_crosshairs = self.has_victim_in_crosshairs(state);

		if self.victim_in_crosshairs {
			self.shoot_timer.set_if_idle(self.rng.gen_range(0.0..2.0 * self.settings.response_time));
		}

		if self.shoot_timer.tick(state.dt()).just_finished() {
			state.inputs_mut().press_button(Button::Mouse1)
		}
	}

	fn execute_moving(&mut self, state: &mut Client) -> Option<()> {
		if let Some(aimpoint) = self.aimpoint {
			if self.delta_orientation?.yaw < 30.0 * DEG {
				state.inputs_mut().press_button(Button::Forward);
			} else {
				state.inputs_mut().release_button(Button::Forward);
			}

			// move sideways if waypoint risks disappearing behind a corner.
			// E.g.: waypoint becomes invisible from right eye => strafe left.
			//
			//          * waypoint
			//         / \
			//        /   \
			//       /    ======WALL====
			//      /       \
			//     o         o
			//   left       right
			//   eye        eye
			{
				let cam_pos = state.local_player().camera().position;
				let orientation = state.local_player().orientation();
				{
					let left_eye = cam_pos - 0.5 * orientation.look_right();
					if is_obstructed(&state.map, left_eye, aimpoint) {
						// TODO: f32 left/right neurons, added to
						state.inputs_mut().press_button(Button::Right);
						state.inputs_mut().release_button(Button::Forward);
					} else {
						state.inputs_mut().release_button(Button::Right) // TODO: always release at beginning of tick?
					}
				}

				{
					let right_eye = cam_pos + 0.5 * orientation.look_right();
					if is_obstructed(&state.map, right_eye, aimpoint) {
						state.inputs_mut().press_button(Button::Left);
						state.inputs_mut().release_button(Button::Forward);
					} else {
						state.inputs_mut().release_button(Button::Left) // TODO: always release at beginning of tick?
					}
				}
			}
		} else {
			state.inputs_mut().release_button(Button::Forward);
		}

		Some(())
	}

	fn has_victim_in_crosshairs(&mut self, state: &Client) -> bool {
		let ray = state.local_player().line_of_sight();
		match state.intersect_except_player(state.local_player_id, &ray) {
			Some((_, Some(id))) => match state.entities.players.get(&id) {
				Some(player) => player.team != state.local_player().team,
				None => false,
			},
			_ => return false,
		}
	}

	fn respawn(&mut self, state: &mut Client) {
		if !state.local_player().spawned {
			if self.respawn_timer.ensure_running().tick(state.dt()).just_finished() {
				state.pending_diffs.push(ReadyToSpawn)
			}
		}
	}
}

fn can_see(map: &Map, player: &Player, target: vec3) -> bool {
	let camera = player.camera();
	let to_target = (target - camera.position).normalized();
	camera.orientation.look_dir().dot(to_target) > 0.0 && !is_obstructed(map, camera.position, target)
}

pub(crate) fn draw_bot_overlay(sg: &mut SceneGraph, state: &Client, bot: &Bot) {
	draw_waypoint(sg, state, bot);
	draw_aimpoint(sg, state, bot);
}

fn draw_waypoint(sg: &mut SceneGraph, state: &Client, bot: &Bot) -> Option<()> {
	draw_prop(
		sg,
		state,
		&Prop {
			mesh: handle("shield"), // TODO
			texture: handle("#FF3333"),
			aabb_size: vec3::ONES, // <<<<<
			on_collide: None,
			transform: Transform {
				translation: bot.waypoint?,
				scale: 1.2,
			},
			pickup_point_id: None,
		},
	)
}

fn draw_aimpoint(sg: &mut SceneGraph, state: &Client, bot: &Bot) -> Option<()> {
	draw_prop(
		sg,
		state,
		&Prop {
			mesh: handle("shield"), // TODO
			texture: handle("#3333FF"),
			aabb_size: vec3::ONES, // <<<<<
			on_collide: None,
			transform: Transform {
				translation: bot.aimpoint?,
				scale: 1.3,
			},
			pickup_point_id: None,
		},
	)
}
