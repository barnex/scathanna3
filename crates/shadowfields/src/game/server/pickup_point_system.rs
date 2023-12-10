///!
///! Pickup point system
///!
///! Re-populates pickup points (see Metadata, CustomProperties.pickup) from time to time.
///!
use super::internal::*;

pub(crate) fn tick_pickup_points(state: &mut ServerState) {
	let dt = state.dt;

	let mut props = SmallVec::<[Prop; 1]>::new();
	for (i, pickup_point) in state.map.pickup_points.iter_mut().enumerate() {
		pickup_point.timer.tick(dt);
		if pickup_point.timer.just_finished() {
			log::trace!("pickup point {i}: timer finished");
			match Prop::from_handle(pickup_point.item){
				Ok(prop) => props.push(
					prop.with(|p| p.transform.translation = pickup_point.pos)
					.with(|p| p.pickup_point_id = Some(i)),
				),
				Err(e) => debug_panic(e),
			};
		}
	}
	for prop in props {
		state.spawn_game_object(prop);
	}
}
