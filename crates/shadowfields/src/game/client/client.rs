use super::internal::*;

/// The `Client` updates its state based on local input and network messages,
/// then sends updates to the network and renders its state in the local window.
///
///                                 +----------+
///		 conn.messages ---------->  |          |   -- messages --> conn
///		                            |  Client  |   
///		 win.inputs --> keymap -->  |          |   
///		                            +----------+   -- draw   -->  win
///
pub(crate) struct Client {
	pub settings: Settings,
	pub win: WinitWindow,
	pub res: Resources,
	pub sound_pack: SoundPack,
	pub debug: DebugOpts,
	pub hud: HUD,
	pub zones: Vec<Object>,
	pub effects: Vec<Effect>,

	pub conn: NetPipe<ClientMsg, ServerMsg>,
	pub pending_diffs: Vec<ClientMsg>,

	pub map: Map,
	pub entities: Entities,

	// local player resources
	pub local_player_id: ID,
	pub advantage: bool,
	pub weapon_state: WeaponState,
}

impl Client {
	pub async fn gameloop(settings: Settings, win: WinitWindow) -> Result<()> {
		let mut client = Self::connect(settings, win).await?;

		match client.settings.bot.enabled {
			false => Self::run_human_gameloop(&mut client).await,
			true => run_bot_gameloop(&mut client).await,
		}
	}

	async fn connect(settings: Settings, win: WinitWindow) -> Result<Client> {
		let mut win = win;

		let server = settings.network.servers.first().ok_or(anyhow!("no servers in settings"))?.clone();
		let join_req = JoinRequest {
			name: settings.player.name.clone(),
			avatar_id: settings.player.avatar,
			team: settings.player.team.parse()?,
		};
		let (conn, acc) = with_loading_screen(&mut win, move || connect(server.as_str(), join_req)).await?;

		let (res, map, zones, entities) = with_loading_screen(&mut win, || -> Result<_> {
			let mut res = Resources::new()?;
			let (map, zones, entities) = load_state(&mut res, acc.map_switch)?;
			Ok((res, map, zones, entities))
		})
		.await?;

		let sound_settings = settings.sound.clone();
		let sound_pack = with_loading_screen(&mut win, move || SoundPack::new(&sound_settings)).await?;

		let hud = HUD::new();

		let mut client = Client {
			settings,
			win,
			hud,
			conn,
			res,
			local_player_id: acc.player_id,
			map,
			entities,
			effects: default(),
			debug: default(),
			pending_diffs: default(),
			zones,
			advantage: false,
			weapon_state: default(),
			sound_pack,
		};

		if client.settings.player.advantage {
			client.pending_diffs.push(ClientMsg::Command(format!("adv {}", client.local_player().name)))
		}

		Ok(client)
	}

	async fn run_human_gameloop(client: &mut Client) -> Result<()> {
		loop {
			// 💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀
			// 💀 Systems order is critical                        💀
			// 💀 Getting it wrong results in subtle added latency 💀
			// 💀 which is hard to spot but deadly                 💀
			// 💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀💀

			// Loop is cyclical so it does not matter where we start
			// Arbitrarily start at drawing

			let mut sg = SceneGraph::new(client.win.viewport_size);
			draw_gamestate(&mut sg, client);
			draw_debug_overlay(&mut sg, client);
			client.win.present_and_wait(sg).await;
			//         👆 this collects the inputs that came in while awaiting the frame

			// 💀 shoot based on what you saw during the last frame
			// 💀 do not move the player or look direction yet,
			// 💀 because the shot needs to correspond to what's currently on screen
			control_shooting(client);

			// What follows can be in pretty much any order

			// Apply server messages first as they are delayed by network latency,
			// but this only rarely matters, e.g., in case of a collision with the local player.
			apply_all_server_messages(client)?;
			extrapolate_other_players(client); // 👈 TODO: this is very inaccurate. should take into account time since last message

			// Now update the local player's position, hud, etc.
			control_player_movement(client);
			jump_pad_system(client);
			control_respawn(client);
			console_system(client).await?;
			animate_footsteps(client);
			tick_effects(client);
			client.hud.tick(client.dt());

			client.apply_and_send_diffs()?;
		}
	}

	pub(crate) async fn bot_pre_tick(client: &mut Client, bot: &Bot) -> Result<()> {
		console_system(client).await?;

		apply_all_server_messages(client)?;
		extrapolate_other_players(client);

		control_respawn(client);
		control_player_movement(client);
		control_shooting(client);

		jump_pad_system(client);
		animate_footsteps(client);

		tick_effects(client);
		client.hud.tick(client.dt());

		client.apply_and_send_diffs()?;

		let mut sg = SceneGraph::new(client.win.viewport_size);
		draw_gamestate(&mut sg, client);
		draw_debug_overlay(&mut sg, client);
		draw_bot_overlay(&mut sg, client, bot);

		client.win.present_and_wait(sg).await;

		Ok(())
	}

	pub fn dt(&self) -> f32 {
		self.win.inputs.tick_time.as_secs_f32() * self.settings.debug.time_passage
	}

	pub fn inputs(&self) -> &Inputs {
		&self.win.inputs
	}

	pub fn inputs_mut(&mut self) -> &mut Inputs {
		&mut self.win.inputs
	}

	fn apply_and_send_diffs(&mut self) -> Result<()> {
		let diffs = mem::take(&mut self.pending_diffs);
		self.apply_self_msgs(&diffs);
		for diff in diffs {
			self.conn.send(diff)?;
		}
		Ok(())
	}

	/// Apply a message by the local client, without round-tripping to the server.
	/// This only applies:
	///
	///   * updates to the local player, so that position/orientation don't lag by one round-trip-time.
	///   * visual effects, because these don't otherwise interact with the game state.
	///
	/// Other messages are not applied locally, but go to the server
	/// and eventually mutate the local GameState via `apply_server_msg`.
	pub fn apply_self_msgs(&mut self, msgs: &ClientMsgs) {
		use ClientMsg::*;
		for msg in msgs {
			match msg {
				MovePlayerIfSpawned { .. } => (/*already applied locally by control*/),
				AddEffect(effect) => handle_add_effect(self, effect.clone()),
				PlaySound(sound) => play_sound(self, sound),
				HitPlayer { .. } => (/* handled by server*/),
				ReadyToSpawn => (/*handled by server*/),
				Command(_) => (/*handled by server*/),
			}
		}
	}

	/// The player controlled by this client.
	pub fn local_player(&self) -> &Player {
		&self.entities.players[&self.local_player_id]
	}

	/// See `local_player`.
	pub fn local_player_mut(&mut self) -> &mut Player {
		self.entities.players.get_mut(&self.local_player_id).unwrap()
	}

	pub fn world(&self) -> (&Map, &Entities) {
		(&self.map, &self.entities)
	}

	pub fn intersect_except_player(&self, player_id: ID, ray: &Ray64) -> Option<(f64, Option<ID>)> {
		_intersect_except_player((&self.map, &self.entities), player_id, ray)
	}
}
