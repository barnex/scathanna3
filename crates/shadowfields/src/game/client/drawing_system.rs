use super::internal::*;
use crate::resources::effect_pack::*;

pub(crate) fn draw_gamestate(sg: &mut SceneGraph, state: &Client) {
	sg.camera = state.local_player().camera().clone();
	sg.bg_color = state.map.sky_color;
	sg.sun_dir = state.map.sun_dir;
	sg.sun_color = state.map.sun_color;

	draw_skybox(sg, state);
	sg.objects.extend_from_slice(&state.zones);

	draw_players(sg, state);
	draw_effects(sg, state);
	draw_props(sg, state);

	if state.debug.axes {
		draw_axes(sg, &state.res);
	}

	draw_hud(sg, state);
}

fn draw_skybox(sg: &mut SceneGraph, state: &Client) -> Option<()> {
	let ctx = ctx();
	let tex = state.map.sky_box?;
	let tex = state.res.textures.load_sync_with(
		|handle| {
			let img = load_image(handle)?;
			Ok(ctx.upload_image_nomip(&img, &TextureOpts::DEFAULT))
		},
		tex,
	)?;

	let vao = state.res.vaos.load_sync_with(
		|handle| {
			let meshbuf = load_wavefront_merged(handle)?;
			let meshbuf = meshbuf.map_positions(|p| p * 512.0);
			Ok(ctx.upload_meshbuffer(&meshbuf))
		},
		handle("skyball"),
	)?;

	let shader = ctx.shader_pack.flat(&tex);
	let obj = Object::new(vao, shader);
	sg.push(obj);
	Some(())
}

fn draw_hud(sg: &mut SceneGraph, state: &Client) {
	state.hud.draw_on(sg);
}

fn draw_axes(sg: &mut SceneGraph, res: &Resources) -> Option<()> {
	let ctx = ctx();
	let axes = res.vaos.load_sync(handle("axes"))?;
	let tex = res.textures.load_sync(handle("rainbow"))?;
	const SIZE: f32 = 16.0;
	sg.push(Object::new(
		axes,
		ctx.shader_pack.entity(&tex, scale_matrix(SIZE), &BoundingBox { min: -vec3::ONES, max: vec3::ONES }, &LightBox::WHITE),
	));
	Some(())
}

fn draw_props(sg: &mut SceneGraph, state: &Client) {
	for prop in state.entities.props.values() {
		draw_prop(sg, state, prop);
	}
}

pub(crate) fn draw_prop(sg: &mut SceneGraph, state: &Client, prop: &Prop) -> Option<()> {
	let ctx = ctx();
	let vao = state.res.vao(prop.mesh);

	// release build: don't draw object if mesh does not exist.
	// else: abort.
	debug_assert!(vao.is_some());

	let tex = state.res.textures.load_sync(prop.texture).unwrap_or(ctx.fallback_texture.clone());

	let vao = vao?;
	sg.push(Object::new(
		vao,
		ctx.shader_pack.entity(
			&tex,
			prop.transform.matrix(),
			&prop.bounds(),
			&state.map.volumetric_light_cache.lightbox_for(&state.map, &prop.bounds()),
		),
	));
	Some(())
}

fn draw_effects(sg: &mut SceneGraph, state: &Client) {
	for effect in &state.effects {
		draw_effect(sg, state, effect)
	}
}

fn draw_effect(sg: &mut SceneGraph, state: &Client, effect: &Effect) {
	match effect.typ {
		EffectType::ParticleExplosion { pos, texture } => draw_particle_explosion(sg, &state, pos, texture, effect.ttl),
		EffectType::Debris { pos, normal, texture } => draw_debris(sg, &state, pos, normal, texture, effect.ttl),
		EffectType::ParticleBeam { start, orientation, len, texture } => draw_particle_beam(sg, &state, start, orientation, len, texture, effect.ttl),
	};
}

fn draw_particle_explosion(sg: &mut SceneGraph, state: &Client, pos: vec3, texture: Handle, ttl: f32) {
	let ctx = ctx();

	const TTL: f32 = PARTICLE_EXPLOSION_TTL;
	let time = TTL - ttl; // 0 ... PARTICLE_EXPLOSION_TTL (seconds)
	let phase = time / TTL; // 0 ... 1

	// decrease number of particles over time
	let num_indices = (((3 * PARTICLE_EXPLOSION_N) as f32) * (1.0 - phase)) as u32; // 💀 not multiple of 3
	let num_indices = num_indices.clamp(3, 3 * PARTICLE_EXPLOSION_N);

	let transf = translation_matrix(pos);
	let vao = state.res.effects.particle_explosion.clone();
	let texture = state.res.textures.load_or_default(texture);
	let obj = Object::new(vao, ctx.shader_pack.particles(&texture, transf, time)).with(|o| o.index_range = Some(0..num_indices));

	sg.push(obj);
}

fn draw_debris(sg: &mut SceneGraph, state: &Client, pos: vec3, normal: vec3, texture: Handle, ttl: f32) {
	let ctx = ctx();

	const TTL: f32 = DEBRIS_TTL;
	const N: u32 = DEBRIS_N;
	let time = TTL - ttl; // 0 ... PARTICLE_EXPLOSION_TTL (seconds)
	let phase = time / TTL; // 0 ... 1

	// decrease number of particles over time
	let num_indices = (((3 * N) as f32) * (1.0 - phase)) as u32; // 💀 not multiple of 3
	let num_indices = num_indices.clamp(3, 3 * N);

	let pitch = normal.y().acos();
	let yaw = -f32::atan2(normal.x(), normal.z());
	let transf = translation_matrix(pos) * yaw_matrix(yaw) * pitch_matrix(pitch);

	let vao = state.res.effects.debris.clone();
	let texture = state.res.textures.load_or_default(texture);
	let obj = Object::new(vao, ctx.shader_pack.debris(&texture, transf, time)).with(|o| o.index_range = Some(0..num_indices));

	sg.push(obj);
}

fn draw_particle_beam(sg: &mut SceneGraph, state: &Client, start: vec3, orientation: Orientation, len: f32, texture: Handle, ttl: f32) {
	let ctx = ctx();

	let time = PARTICLE_BEAM_TTL - ttl;

	let pitch_mat = pitch_matrix(-90.0 * DEG - orientation.pitch);
	let yaw_mat = yaw_matrix(180.0 * DEG - orientation.yaw);
	let location_mat = translation_matrix(start);
	let transf = location_mat * yaw_mat * pitch_mat;

	// pick the number of triangles to match the desired beam length.
	// number of vertices = 3*number of triangles.
	let vao = state.res.effects.particle_beam.clone();

	// how fast the head of the particle beam moves forward
	let bullet_speed = 500.0; // m/s

	let max_len = time * bullet_speed;
	let len = f32::min(len, max_len);

	let n = 3 * (len as u32 + 1) * PARTICLE_BEAM_DENSITY;
	let n = n.clamp(3, vao.num_indices); // TODO!

	let texture = state.res.textures.load_or_default(texture);
	let obj = Object::new(vao, ctx.shader_pack.particles(&texture, transf, time)).with(|o| o.index_range = Some(0..n));

	sg.push(obj);
}

pub(crate) fn draw_line(sg: &mut SceneGraph, start: vec3, end: vec3) {
	let ctx = ctx();
	let buf = MeshBuffer::line(start, end); // TODO: don't upload a new vao each frame
	let vao = Arc::new(ctx.upload_meshbuffer(&buf));
	sg.push(Object::new(vao, ctx.shader_pack.lines(&ctx.fallback_texture)))
}

fn draw_players(sg: &mut SceneGraph, state: &Client) {
	for (_, player) in state.entities.players.iter().filter(|(_, p)| p.spawned) {
		if player.id == state.local_player_id {
			//draw_player_1st_person(rs, sg, world, player);
		} else {
			//if camera.can_see(player.position()) {
			draw_player_3d_person(sg, state, player);
			//}
		}
	}
}

fn draw_player_1st_person(rs: &Resources, sg: &mut SceneGraph, player: &Player) {
	//rs.model_pack.get(0 /*TODO*/).draw_1st_person(sg, rs, player);
	//let line_of_fire = player.line_of_fire(world);
	//let shoot_at = player.shoot_at(world);
	//self.draw_line(sg, rs, line_of_fire.start.to_f32(), shoot_at);
}

fn draw_player_3d_person(sg: &mut SceneGraph, state: &Client, player: &Player) -> Option<()> {
	match has_morph_model(player.avatar_id) {
		true => draw_player_3d_person_morphed(sg, state, player),
		false => draw_player_3d_person_parts(sg, state, player),
	}
}

/// Draw an avatar made up from morph targets (like "robot_frame.obj").
fn draw_player_3d_person_morphed(sg: &mut SceneGraph, state: &Client, player: &Player) -> Option<()> {
	let entities = &state.entities;
	let res = &state.res;
	let ctx = ctx();
	let bounds = player.skeleton.bounds();
	let lightbox = state.map.volumetric_light_cache.lightbox_for(&state.map, &bounds);
	let avatar = player.avatar_id;

	// Body
	{
		let matrix = translation_matrix(player.position()) //.
			* yaw_matrix(180.8*DEG /*BLENDER HACK*/-player.skeleton.frame().orientation.yaw) //.
			* scale_matrix(player.torso_size.y()); // <<<< TODO
		let feet_phase = entities.animation_state.get(&player.id).cloned().unwrap_or_default().feet_phase; // TODO: phase = 0..1.
		let t = 0.5 * (feet_phase / PI) + 0.5;
		let anim_vaos = &res.animations.load_sync(animation_for_avatar_id(avatar))?;
		let tex = res.textures.load_sync(texture_for_avatar(avatar, player.team))?;
		sg.push(anim_vaos.draw_animated(&tex, matrix, t, &bounds, &lightbox));
	}

	// Head
	{
		let head = res.vaos.load_sync(head_for_avatar_id(avatar))?;
		let matrix = translation_matrix(player.position() + vec3::EY * (0.98 * player.torso_size.y() /*NECK HACK*/)) //.
				* yaw_matrix(180.0*DEG /*BLENDER HACK*/-player.skeleton.frame().orientation.yaw) //.
				* pitch_matrix(-0.4 * player.orientation().pitch)
				* scale_matrix(player.head_size.y());
		let tex = res.textures.load_sync(texture_for_avatar(avatar, player.team))?;
		sg.push(Object::new(head, ctx.shader_pack.entity(&tex, matrix, &player.skeleton.bounds(), &lightbox)));
	}
	Some(())
}

/// Draw an avatar made up from loose parts (separate head & feet meshes, like "frog.obj", "frog_foot.obj").
fn draw_player_3d_person_parts(sg: &mut SceneGraph, state: &Client, player: &Player) -> Option<()> {
	let ctx = ctx();
	let bounds = player.skeleton.bounds();
	let lightbox = state.map.volumetric_light_cache.lightbox_for(&state.map, &bounds);

	let tex = state.res.textures.load_sync(texture_for_avatar(player.avatar_id, player.team))?;

	let yaw = yaw_matrix(180.0*DEG /*BLENDER HACK*/-player.skeleton.frame().orientation.yaw);

	// Head
	{
		let head = state.res.vaos.load_sync(head_for_avatar_id(player.avatar_id))?;

		let matrix = translation_matrix(player.position() + vec3::EY * player.torso_size.y()) //.
				* &yaw //.
				* pitch_matrix(-0.4 * player.orientation().pitch)
				* scale_matrix(player.head_size.y());

		sg.push(Object::new(head, ctx.shader_pack.entity(&tex, matrix, &bounds, &lightbox)));
	}

	// Feet

	{
		let feet_phase = state.entities.animation_state.get(&player.id).cloned().unwrap_or_default().feet_phase;

		const FOOT_SCALE: f32 = 0.15;
		let foot = state.res.vaos.load_sync(foot_for_avatar_id(player.avatar_id))?;

		let [l_pos, r_pos] = feet_pos_internal(player, feet_phase);
		let l_matrix = translation_matrix(player.position()) //.
				* &yaw
				*translation_matrix(l_pos)
				* scale_matrix(FOOT_SCALE);

		let r_matrix = translation_matrix(player.position()) //.
				* &yaw
				*translation_matrix(r_pos)
				* scale_matrix(FOOT_SCALE);

		sg.push(Object::new(foot.clone(), ctx.shader_pack.entity(&tex, l_matrix, &bounds, &lightbox)));
		sg.push(Object::new(foot, ctx.shader_pack.entity(&tex, r_matrix, &bounds, &lightbox)));
	}

	Some(())
}

fn feet_pos_internal(player: &Player, phase: f32) -> [vec3; 2] {
	const SEP: f32 = 0.15;

	let r = 0.2;
	let c = phase.cos();
	let s = phase.sin();

	[
		vec3(-SEP, r * s.max(0.0), r * c), //
		vec3(SEP, r * (-s).max(0.0), -r * c),
	]
}

/// Is the an avatar model made up from moving loose parts,
/// or from morph targets?
fn has_morph_model(avatar: u8) -> bool {
	match avatar {
		0..=9 => false, // little animals
		10.. => true,   // witch etc
	}
}

fn texture_for_avatar(avatar: u8, team: Team) -> Handle {
	use Team::*;
	match (avatar, team) {
		(0, _) => handle("checkboard4"),
		(1, _) => handle("bunny"),
		(2, _) => handle("chicken"),
		(3, _) => handle("frog"),
		(4, _) => handle("hamster"),
		(5, _) => handle("panda"),
		(6, _) => handle("pig"),
		(7, _) => handle("turkey"),
		(10, _) => handle("chicken"),
		(11 | 12, Red) => handle("skin_red"),
		(11 | 12, Green) => handle("skin_green"),
		(11 | 12, Blue) => handle("skin_blue"),
		_ => handle("checkboard4"),
	}
}

fn head_for_avatar_id(avatar: u8) -> Handle {
	match avatar {
		0 => handle("box"),
		1 => handle("bunny_head"),
		2 => handle("chicken_head"),
		3 => handle("frog_head"),
		4 => handle("hamster_head"),
		5 => handle("panda_head"),
		6 => handle("pig_head"),
		7 => handle("turkey_head"),
		10 => handle("chicken_head"),
		11 => handle("witch_head"),
		12 => handle("wizard_head"),
		_ => handle("box"),
	}
}

fn foot_for_avatar_id(avatar: u8) -> Handle {
	match avatar {
		0 => handle("simple_foot"),
		1 => handle("frog_foot"),
		2 => handle("chicken_leg"),
		3 => handle("frog_foot"),
		4 => handle("hamster_foot"),
		5 => handle("panda_foot"),
		6 => handle("frog_foot"),
		7 => handle("chicken_leg"),
		_ => handle("simple_foot"),
	}
}

fn animation_for_avatar_id(avatar: u8) -> Handle {
	match avatar {
		10 => handle("chicken"),
		11 => handle("witch"),
		12 => handle("witch"),
		_ => handle("chicken"),
	}
}

// fn _draw_player_3d_person_morphed(res: &mut Resources, sg: &mut SceneGraph, map: &Map, vl: &mut VolumetricLight, entities: &Entities, player: &Player) -> Option<()> {
// 	let ctx = ctx();
// 	let lightbox = vl.lightbox_for(map, &player.skeleton.bounds());
//
// 	// TORSO
// 	{
// 		let matrix = translation_matrix(player.position()) * yaw_matrix(-player.skeleton.frame().orientation.yaw) * scale_matrix(player.torso_size.y());
// 		let feet_phase = entities.animation_state.get(&player.id).cloned().unwrap_or_default().feet_phase;
// 		let t = 0.5 * (feet_phase / PI) + 0.5;
// 		let bob = &res.animations.load_sync(handle("chicken"))?;
// 		let tex = res.textures.load_sync(handle("skin1"))?;
// 		sg.push(bob.draw_animated(&tex, matrix, t, &player.skeleton.bounds(), &lightbox));
// 	}
//
// 	// Head
// 	{
// 		let head = res.vaos.load_sync(handle("chicken_head"))?;
// 		let matrix = translation_matrix(player.position() + vec3::EY * (0.95 * player.torso_size.y() /*NECK HACK*/)) //.
// 				* yaw_matrix(180.0*DEG /*BLENDER HACK*/-player.skeleton.frame().orientation.yaw) //.
// 				* pitch_matrix(-0.4 * player.orientation().pitch)
// 				* scale_matrix(player.head_size.y());
// 		let tex = res.textures.load_sync(handle("chicken"))?;
// 		sg.push(Object::new(head, ctx.shader_pack.entity(&tex, matrix, &player.skeleton.bounds(), &lightbox)));
// 	}
//
// 	// {
// 	// 	let gun = res.vaos.load_sync(&res.ctx, handle("bubblegun"))?;
// 	// 	let matrix = translation_matrix(player.position() + gun_pos_internal(player)) //.
// 	// 			* yaw_matrix(180.0*DEG /*BLENDER HACK*/-player.skeleton.frame().orientation.yaw) //.
// 	// 			* pitch_matrix(-player.orientation().pitch)
// 	// 			* scale_matrix(Player::HEAD_HEIGHT);
// 	// 	let tex = res.textures.load_sync(&res.ctx, handle("party_hat"))?;
// 	// 	sg.push(Object::new(gun, res.ctx.shader_pack.entity(tex, matrix)));
// 	// }
//
// 	Some(())
// }
