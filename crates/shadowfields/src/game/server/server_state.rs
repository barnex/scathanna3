use super::internal::*;

/// A game server's mutable state and business logic.
///
/// Owned and controlled by a NetServer, who adds an RPC layer on top.
///
/// Handles NetServer events:
///   * add/drop player
///   * tick
///   * client messages
///
pub(crate) struct ServerState {
	pub dt: f32,

	// Each of these pieces of data is primarily controlled by one system.
	// (Though there are some dependencies).
	pub maplist: Vec<String>,
	pub autoswitch: AutoSwitch,
	pub scores: Scores,
	pub sprees: HashMap<ID, (f32, u32)>,
	pub health: HashMap<ID, Health>,
	pub hud_timer: Timer,

	pub map: Map,

	pub entities: Entities,

	prev_entities: Entities,

	pub diffs: Diffs,
}

impl ServerState {
	pub fn new(settings: ServerOpts) -> Result<Self> {
		let maplist = load_maplist(&settings)?;
		let autoswitch = AutoSwitch::from_settings(&settings);
		let scores = default();
		let sprees = default();
		let map = Map::load(maplist.get(0).ok_or_else(|| anyhow!("no maps"))?)?;
		let entities = default();
		let diffs = default();
		let health = default();
		let hud_timer = Timer::repeating(0.3);

		Ok(Self {
			dt: default(),
			maplist,
			autoswitch,
			map,
			sprees,
			scores,
			entities,
			prev_entities: default(),
			diffs,
			health,
			hud_timer,
		})
	}

	/// Add a new player to the game and return their unique ID.
	pub fn handle_join_new_player(&mut self, join_msg: JoinRequest) -> (ID, MapSwitch) {
		join_new_player(self, join_msg)
	}

	pub fn handle_drop_player(&mut self, client_id: ID) {
		drop_player(self, client_id);
	}

	pub fn handle_tick(&mut self, dt: f32) -> Diffs {
		self.dt = dt;

		// systems
		tick_pickup_points(self);
		tick_collisions(self);
		tick_lava(self);
		tick_health(self);
		tick_killplane(self);
		tick_hud_text(self);
		tick_autoswitch(self);

		mem::take(&mut self.diffs)
	}

	/// Respond to message sent by a player.
	pub fn handle_client_msg(&mut self, player_id: ID, msg: ClientMsg) {
		// check that the player has not been disconnected in a network race.
		// after this check, all downstream methods may safely use `self.player(id)`,
		// as we will never remove a player while handling client messages.
		if !self.entities.players.contains_key(&player_id) {
			return;
		}

		use ClientMsg::*;
		match msg {
			MovePlayerIfSpawned(frame) => self.handle_move_player_if_spawned(player_id, frame),
			ReadyToSpawn => handle_ready_to_respawn(self, player_id),
			AddEffect(effect) => self.handle_add_effect(player_id, effect),
			PlaySound(sound) => self.handle_play_sound(player_id, sound),
			HitPlayer(victim_id) => handle_hit_player(self, player_id, victim_id),
			Command(cmd) => server_command(self, player_id, cmd),
		};
	}

	fn handle_move_player_if_spawned(&mut self, player_id: ID, frame: Frame) {
		self.move_player_if_spawned(player_id, frame);
	}

	// Handle a client's AddEffect message: just broadcast to other clients.
	// There is little point in adding visual effects to the server's world.
	fn handle_add_effect(&mut self, player_id: ID, effect: Effect) {
		self.diffs.push(AddEffect(effect).to_not(player_id))
	}

	// Handle a client's PlaySound message: just broadcast to other clients.
	fn handle_play_sound(&mut self, player_id: ID, sound: SoundEffect) {
		self.diffs.push(PlaySound(sound).to_not(player_id))
	}

	pub fn spawn_game_object(&mut self, gobj: Prop) -> ID {
		let id = ID::new();
		log::trace!("spawn_game_object {id}: {gobj:?}");
		self.entities.insert_prop(id, gobj.clone());
		self.diffs.push(InsertEntity(id, gobj).to_all());
		id
	}

	pub fn remove_game_object(&mut self, id: ID) {
		self.entities.props.remove(&id);
		self.diffs.push(RemoveObject(id).to_all());
	}

	pub fn drop_player(&mut self, player_id: ID) {
		self.entities.players.remove(&player_id);
		self.diffs.push(DropPlayer(player_id).to_not(player_id));
	}

	pub fn apply_to_player_partial<F: Fn(&mut Player)>(&mut self, player_id: ID, f: F) -> Option<()> {
		self.entities.players.get_mut(&player_id).and_then(|player| {
			Some({
				f(player);
				self.diffs.push(UpdatePlayerPartial(player.clone()).to_all());
			})
		})
	}

	pub fn force_apply_to_full<F: Fn(&mut Player)>(&mut self, player_id: ID, f: F) -> Option<()> {
		self.entities.players.get_mut(&player_id).and_then(|player| {
			Some({
				f(player);
				self.diffs.push(UpdatePlayerFull(player.clone()).to_all());
			})
		})
	}

	pub fn move_player_if_spawned(&mut self, player_id: ID, frame: Frame) {
		if let Some(player) = self.entities.players.get_mut(&player_id) {
			if player.spawned {
				player.skeleton.set_frame(frame);
				self.diffs.push(MovePlayer(player_id, player.skeleton.frame()).to_not(player_id));
			}
		}
	}

	pub fn join_new_player(&mut self, spawn_point: &SpawnPoint, join_msg: JoinRequest) -> (ID, MapSwitch) {
		// Join new player cannot be done via apply(msg):
		// we need to add the player before we can get the player ID.

		let player_id = self.entities.join_new_player(spawn_point, join_msg);
		let player = self.entities.players[&player_id].clone();
		self.diffs.push(AddPlayer(player).to_all());
		self.health.insert(player_id, default());

		let map_switch = MapSwitch {
			map_name: self.map.name().into(),
			entities: self.entities.clone(),
		};

		(player_id, map_switch)
	}
}

//-------------------------------------------------------------------------------- respawn

pub(crate) fn handle_ready_to_respawn(state: &mut ServerState, player_id: ID) {
	trace!("ready_to_respawn: {player_id}");
	let spawn_point = pick_spawn_point(state);
	state.force_apply_to_full(player_id, |p| {
		if !p.spawned {
			trace!("respawn {player_id}");
			// client could request spawn multiple times in a network race.
			p.spawned = true;
			p.skeleton.position = spawn_point.position;
			p.skeleton.orientation = spawn_point.orientation();
		}
	});
	state.health.entry(player_id).or_default().reset();
}

fn pick_spawn_point(state: &ServerState) -> SpawnPoint {
	pick_random(&state.map.spawn_points).cloned().unwrap_or_default()
}

//-------------------------------------------------------------------------------- join/drop/switch players

pub(crate) fn join_new_player(state: &mut ServerState, join_msg: JoinRequest) -> (ID, MapSwitch) {
	let spawn_point = pick_spawn_point(state);

	let (player_id, map_switch) = state.join_new_player(&spawn_point, join_msg);

	state.scores.join_new_player(player_id);
	// self.broadcast_scores(world);
	log(state, format!("{} joined", must_name(state, player_id)));
	let map_name = state.map.name().to_string();
	hud_announce(state, Just(player_id), map_name);
	state.diffs.push(PlaySound(SoundEffect::raw(handle("ann_begin"))).to_just(player_id));
	(player_id, map_switch)
}

pub(crate) fn drop_player(state: &mut ServerState, player_id: ID) {
	log(state, format!("{} left", must_name(state, player_id)));
	state.drop_player(player_id);
}

/// Player name.
pub(crate) fn player_name(state: &ServerState, id: ID) -> Option<&str> {
	state.entities.players.get(&id).map(|p| p.name.as_str())
}

/// Player name or "???".
pub(crate) fn must_name(state: &ServerState, id: ID) -> &str {
	player_name(state, id).unwrap_or("???")
}

/// Log a message to all player's consoles. E.g. "Bob joined the game".
pub(crate) fn log(state: &mut ServerState, msg: impl Into<String>) {
	state.diffs.push(Log(msg.into()).to_all());
}

/// Seconds to show HUD announcements like "You fragged Foo".
pub const ANNOUNCE_TTL: f32 = 5.0;
pub const ANN_VOLUME: f32 = 1.0;

pub(crate) fn sound_announce(state: &mut ServerState, to: Addressee, clip_name: Handle) {
	state.diffs.push(
		PlaySound(SoundEffect {
			clip_name,
			volume: ANN_VOLUME,
			spatial: None,
		})
		.to(to),
	)
}

pub(crate) fn sound_effect(state: &mut ServerState, clip_name: Handle, location: vec3, volume: f32) {
	state.diffs.push(
		PlaySound(SoundEffect {
			clip_name: clip_name.into(),
			volume,
			spatial: Some(Spatial { location }),
		})
		.to_all(),
	)
}

pub(crate) fn despawn(state: &mut ServerState, id: ID) -> Option<()> {
	trace!("despawn {id}");
	state.apply_to_player_partial(id, |p| p.spawned = false)
}

pub(crate) fn player(state: &ServerState, id: ID) -> Option<&Player> {
	state.entities.players.get(&id)
}

/// Spawn an effect (for all players).
pub(crate) fn add_effect(state: &mut ServerState, effect: Effect) {
	state.diffs.push(AddEffect(effect).to_all())
}
