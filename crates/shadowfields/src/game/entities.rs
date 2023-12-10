use super::internal::*;

#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct Entities {
	pub players: HashMap<ID, Player>,
	pub props: HashMap<ID, Prop>,
	pub animation_state: HashMap<ID, AnimationState>,
}

impl Entities {
	pub fn player_ids(&self) -> impl Iterator<Item = ID> {
		_player_ids(&self)
	}

	pub fn spawned_player_ids(&self) -> impl Iterator<Item = ID> {
		_spawned_player_ids(self)
	}

	// TODO: insert(Diff) ?
	pub fn insert_prop(&mut self, id: ID, obj: Prop) {
		let old = self.props.insert(id, obj);
		debug_assert!(old.is_none()); // insert: entity should not yet exist.
	}

	pub fn apply_to_prop(&mut self, id: ID, diff: PropDiff) {
		debug_assert!(self.props.contains_key(&id)); // apply: entity should exist
		if let Some(gobj) = self.props.get_mut(&id) {
			diff.apply_to(gobj);
		}
	}

	pub fn remove_prop(&mut self, id: ID) {
		let old = self.props.remove(&id);
		debug_assert!(old.is_some()); // remove: entity should exist
	}
}

impl Entities {
	pub fn join_new_player(&mut self, spawn_point: &SpawnPoint, req: JoinRequest) -> ID {
		let player_id = ID::new();
		let player = Player::new(player_id, spawn_point.position, spawn_point.orientation(), req.name, req.avatar_id, req.team);
		self.players.insert(player_id, player);
		player_id
	}
}

pub(crate) fn ids<V>(map: &HashMap<ID, V>) -> SmallVec<[ID; 16]> {
	map.keys().copied().collect()
}
