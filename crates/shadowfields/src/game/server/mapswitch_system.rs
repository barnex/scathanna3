//!
//! System to switch between maps
//!

use super::internal::*;

/// Take the maplist from server settings.
/// Use all maps (except test maps) if no list specified.
/// Verify maps by loading them once, so that we get no unexpected errors later.
pub(crate) fn load_maplist(settings: &ServerOpts) -> Result<Vec<String>> {
	println!("server: maplist: {}", settings.maplist.join(", "));
	let maplist = match settings.maplist.len() {
		0 => assets_dir()
			.find_all_maps()?
			.into_iter()
			.filter(|name| !(name.starts_with("test") || name.starts_with("fail")))
			.collect_vec(),
		_ => settings.maplist.clone(),
	};
	verify_maps(&maplist)?;
	Ok(maplist)
}

/// Switch to a map by name, e.g. "deck".
pub(crate) fn switch_map(state: &mut ServerState, map_name: &str) {
	match Map::load(map_name) {
		Err(e) => broadcast_error(state, e),
		Ok(new_map) => {
			trace!("map_switch {}", new_map.name());
			// TODO: throw away entire state, start fresh.
			//for id in state.entities.players.keys().copied().collect::<SmallVec<[_; 8]>>() {
			for id in ids(&state.entities.players) {
				despawn(state, id);
			}
			// announce_winner(state);
			//state.world.switch_map(&mut state.diffs, new_map);

			let map_name = new_map.name().to_string();
			let mut tmp = new_map;
			mem::swap(&mut state.map, &mut tmp);
			drop(tmp /*now the old map*/);

			// Keep players, remove other entities

			state.entities.props.clear();

			state.diffs.push(
				SwitchMap(MapSwitch {
					map_name,
					entities: state.entities.clone(),
				})
				.to_all(),
			);

			state.autoswitch.time_playing = 0.0;
			state.scores.reset(state.entities.players.keys().copied());
			broadcast_scores(state);
			hud_announce(state, All, state.map.name().to_owned());
		}
	}
}

/// Switch to the next map in state.maplist.
pub(crate) fn switch_next_map(state: &mut ServerState) {
	info!("switching to next map");

	let curr = state.maplist.iter().position(|map| map == state.map.name()).unwrap_or(0);

	let next = match (curr + 1) == state.maplist.len() {
		true => 0,
		false => curr + 1,
	};

	if let Some(next_name) = state.maplist.get(next) {
		switch_map(state, &next_name.clone())
	}
}

pub(crate) fn broadcast_error(state: &mut ServerState, err: Error) {
	error!("{err}");
	state.diffs.push(Log(format!("{err}")).to_all());
}

fn verify_maps(maplist: &[String]) -> Result<()> {
	if maplist.is_empty() {
		return Err(anyhow!("server: maplist: need at least one map"));
	}
	for map_name in maplist {
		if let Err(e) = Map::load(map_name) {
			return Err(anyhow!("map {} failed verification: {:#}", map_name, e));
		}
	}
	Ok(())
}
