use super::internal::*;

/// List all currently spawned player IDs (does not borrow).
pub(crate) fn _spawned_player_ids(entities: &Entities) -> impl Iterator<Item = ID> {
	entities //
		.players
		.iter()
		.filter(|(_, p)| p.spawned)
		.map(|(id, _)| id)
		.copied()
		.collect::<SmallVec<[_; 16]>>()
		.into_iter()
}

pub(crate) fn _player_ids(entities: &Entities) -> impl Iterator<Item = ID> {
	entities //
		.players
		.keys()
		.copied()
		.collect::<SmallVec<[_; 16]>>()
		.into_iter()
}
