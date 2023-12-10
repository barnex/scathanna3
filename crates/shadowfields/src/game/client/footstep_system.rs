use super::internal::*;

const FEET_ANIM_SPEED: f32 = 12.0;
const FEET_ANIM_DAMP: f32 = 6.0;

/// Animate the players feet if they are moving.
/// This is done locally by each client (do not send feet position over the network all the time).
/// Also generate footstep, jump,... sounds locally (do not send these sound effects over the network).
pub(crate) fn animate_footsteps(state: &mut Client) {
	for id in state.entities.spawned_player_ids() {
		animate_footsteps_1(state, id);
	}

	/*
		*/
	//let curr = &self.entities.players[&player_id].local;
	//self.make_footstep_sounds(eng, player_id, prev, curr);
}

fn animate_footsteps_1(state: &mut Client, id: ID) -> Option<()> {
	let dt = state.dt();
	let anim = state.entities.animation_state.entry(id).or_default();
	let player = state.entities.players.get(&id)?;

	let walk_speed = player.skeleton.velocity;
	//let vspeed = player.skeleton.velocity.y();
	if walk_speed != vec3::ZERO {
		// move feet in semicircles while moving
		anim.feet_phase += dt * FEET_ANIM_SPEED;

		if anim.feet_phase > PI {
			anim.feet_phase = -PI;
		}
	} else {
		// gradually relax feet to resting position while still
		anim.feet_phase *= 1.0 - (FEET_ANIM_DAMP * dt);
		anim.feet_phase = anim.feet_phase.clamp(-PI, PI);
	}

	// let target_pitch = if vspeed > 0.0 {
	// 	-30.0 * DEG
	// } else if vspeed < 0.0 {
	// 	30.0 * DEG
	// } else {
	// 	0.0
	// };
	//let damp = FEET_ANIM_DAMP * dt;
	//anim.feet_phase = (1.0 - damp) * anim.feet_phase;

	Some(())
}

pub(crate) fn make_footstep_sounds(state: &mut Client, player_id: ID, prev: &AnimationState, curr: &AnimationState) {
	let speed = state.entities.players[&player_id].skeleton.velocity;
	let vspeed = speed.y();
	let walking = { vspeed.abs() < 0.1 && speed != vec3::ZERO };

	if walking {
		if prev.feet_phase.signum() != curr.feet_phase.signum() {
			// make one's own footsteps less loud
			// (quite distracting otherwise)
			let volume = if player_id == state.local_player_id { 0.01 } else { 0.3 };
			play_sound_spatial(
				state,
				random_footstep_clip(),
				volume,
				&Spatial {
					location: state.entities.players[&player_id].position(),
				},
			)
		}
	}
}

fn random_footstep_clip() -> Handle {
	must_pick_random(&[
		handle("footstep01"), //
		handle("footstep02"),
		handle("footstep03"),
		handle("footstep04"),
		handle("footstep05"),
		handle("footstep06"),
		handle("footstep07"),
		handle("footstep08"),
	])
}
