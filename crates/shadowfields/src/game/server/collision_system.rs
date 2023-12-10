///!
///! Collision system.
///!
///! Tests which players collide with which objects and execute `on_collide` scripts.
///! TODO: generate collision events, let other systems handle pickups etc.
///!
use super::internal::*;

pub(crate) fn tick_collisions(state: &mut ServerState) {
	let player_ids = state.entities.spawned_player_ids();
	let collider_ids = state.entities.props.iter().filter_map(|(id, obj)| obj.on_collide.map(|_| id)).copied().collect::<SmallVec<[_; 32]>>();

	for player_id in player_ids {
		let bb = state.entities.players.get(&player_id).map(|p| p.skeleton.bounds()).unwrap_or(BoundingBox::new(default(), default()));
		for &collider_id in &collider_ids {
			if let Some(collider_bb) = state.entities.props.get(&collider_id).map(|c| c.bounds()) {
				if bb.overlaps(&collider_bb) {
					collide_player(state, player_id, collider_id);
				}
			}
		}
	}
}

fn collide_player(state: &mut ServerState, player_id: ID, collider_id: ID) -> Option<()> {
	let collider = state.entities.props.get(&collider_id)?.clone();

	let handle = collider.on_collide?;

	// TODO: `handle!` macro so we can match on the numerical representation.
	match handle.as_str() {
		"shield" => pickup_shield(state, player_id, collider_id),
		"gift_box" => pickup_gift_box(state, player_id, collider_id),
		_ => Some(error!("on_collide: no such handle: {handle}")),
	};

	if let Some(pickup_point_id) = collider.pickup_point_id {
		state.map.pickup_points.get_mut(pickup_point_id)?.timer.reset();
	}

	Some(())
}
