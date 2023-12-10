//!
//! Lava system kills players who walk on lava.
//!
use super::internal::*;

// lava system kills players who are on lava
pub(crate) fn tick_lava(state: &mut ServerState) {
	// for id in spawned_player_ids(&state.entities) {
	// 	(|| {
	// 		Some({
	// 			// Super hack to determine if we are on lava.
	// 			// TODO: reserved material IDs, or mark properties (lava, water, translucent,...) in palette.
	// 			// TODO: why does the player hover 1 unit above the ground (round to int physics??).
	// 			let start = state.entities.players.get(&id)?.position();
	// 			let dir = -vec3::EY;
	// 			let ray = Ray::new(start, dir);
	// 			let hit = state.map.intersect(&ray);
	// 			if hit.t < 2.0 {
	// 				if let Some((_, _, mat_id)) = hit.attrib {
	// 					if state.map.data().palette.material_name_for(mat_id).unwrap_or_default().starts_with("Lava") {
	// 						suicide(state, id, "fell in lava");
	// 					}
	// 				}
	// 			}
	// 		})
	// 	})();
	// }
}
