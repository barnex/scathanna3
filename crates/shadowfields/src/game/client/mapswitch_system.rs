use super::internal::*;

pub(crate) fn switch_map(state: &mut Client, map_switch: MapSwitch) -> Result<()> {
	let (map, zones, entities) = load_state(&mut state.res, map_switch)?;

	state.map = map;
	state.zones = zones;
	state.entities = entities;
	state.effects.clear();
	state.pending_diffs.clear();

	// carry over advantages, e.g. to weapon system
	if state.advantage {
		advantage(state)
	}

	Ok(())
}

//pub(crate) fn load_state_with_splash(win: &mut Shell, res: &mut Resources, map_switch: MapSwitch) -> Result<(Map, Vec<Object>, Entities)> {
//	ui::loading_screen(win, move |_client| load_state(&mut res.lock().unwrap(), map_switch))
//}

pub(crate) fn load_state(res: &mut Resources, map_switch: MapSwitch) -> Result<(Map, Vec<Object>, Entities)> {
	let map_name = &map_switch.map_name;
	let map_dir = assets_dir().find_map_dir(map_name);
	let map = Map::load(&map_name)?;

	let zones = upload_scene_mesh(&map_dir, res)?;

	Ok((map, zones, map_switch.entities))
}
