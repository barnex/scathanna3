//!
//! Draw debug info on the screen. E.g. bounding boxes, draw call stats.
//!

use super::internal::*;
use crate::graphics::wgpu_util::COUNTERS;

pub(crate) fn draw_debug_overlay(sg: &mut SceneGraph, state: &mut Client) {
	if state.debug.dbg_overlay {
		state.hud.set_text(HUDPos::Debug, &fmt_dbg_overlay(&state), 1.0);
	}
	if state.debug.fps_overlay {
		state.hud.set_text(HUDPos::TopRight, &COUNTERS.format_and_reset(), 1.0);
	}
	if state.debug.ecs_overlay {
		state.hud.set_text(HUDPos::Debug, fmt_ecs_overlay(state), 1.0);
	}
	if state.debug.bb_overlay {
		draw_bb_overlay(sg, &state.entities);
	}
}

fn fmt_ecs_overlay(state: &Client) -> String {
	fmt_overlay(&state.entities.animation_state)
}

fn fmt_overlay<V>(component: &HashMap<ID, V>) -> String
where
	V: Serialize,
{
	let mut buf = String::new();
	use std::fmt::Write;

	let keys = component.keys().collect::<Vec<_>>().with(|keys| keys.sort_unstable());
	for id in keys {
		let entity = component.get(&id).unwrap();
		let pretty = ron::to_string(entity).unwrap().replace("\n", "");
		writeln!(&mut buf, "{id}: {pretty}").unwrap();
	}

	buf
}

// debug overlay
pub(crate) fn fmt_dbg_overlay(state: &Client) -> String {
	let player = state.local_player();
	let spawned = player.spawned;
	let target_position = player.skeleton.target_position;
	let filtered_position = player.skeleton.filtered_position;
	let look_dir = player.skeleton.orientation.look_dir();
	let on_ground = player.on_ground(&state.map);
	let velocity = player.skeleton.velocity.0.map(|v| format!("{:+.5}", v));
	let bump = player.bump;

	let mut extra = String::new();
	if player.flying {
		extra.push_str("flying\n");
	}

	format!(
		r#"
spawned: {spawned}
filtered_position: {filtered_position}
targert_position: {target_position}
velocity: {velocity:?}
look_dir: {look_dir}
bump: {bump}
on_ground: {on_ground}
{extra}
"#,
	)
}

// draw bounding boxes
fn draw_bb_overlay(sg: &mut SceneGraph, ecs: &Entities) {
	// for (_,player) in &ecs.players {
	// 	let bb = player.skeleton.bounds();
	// 	let buf = buf.map_positions(|v| v * bb.size() + bb.min);
	// 	let vao = Arc::new(sg.ctx.upload_meshbuffer(&buf));
	// 	let tex = Arc::new(uniform_texture(&sg.ctx, vec4(0.0, 1.0, 0.0, 0.5)));
	// 	sg.push(Object::new(&vao, sg.ctx.shader_pack.entity(&tex, mat4::UNIT)));
	// }
	//for obj in ecs.objects.values() {
	//	let mut buf = MeshBuffer::new();
	//	for face in unit_cube_faces(Handle::default()) {
	//		buf.append(&face_meshbuffer(&face, default(), default(), default()))
	//	}
	//	let bb = obj.bounds();
	//	let buf = buf.map_positions(|v| v * bb.size() + bb.min);
	//	let vao = Arc::new(sg.ctx.upload_meshbuffer(&buf));
	//	let tex = Arc::new(uniform_texture(&sg.ctx, vec4(0.0, 1.0, 0.0, 0.5)));
	//	sg.push(Object::new(&vao, sg.ctx.shader_pack.entity(&tex, mat4::UNIT)));
	//	//sg.push(Object::new(&vao, sg.ctx.shader_pack.highlight(&tex))) // WTF
	//}
}

fn draw_bounding_box(sg: &mut SceneGraph, bb: &BoundingBox32) {
	let mut buf = MeshBuffer::new();

	let (x1, y1, z1) = bb.min.into();
	let (x2, y2, z2) = bb.max.into();

	buf.push_line(dbg!(vec3(x1, y1, z1)), dbg!(vec3(x2, y1, z1)));
	buf.push_line(dbg!(vec3(x2, y1, z1)), dbg!(vec3(x2, y2, z1)));
	buf.push_line(dbg!(vec3(x2, y2, z1)), dbg!(vec3(x1, y2, z1)));
	buf.push_line(dbg!(vec3(x1, y2, z1)), dbg!(vec3(x1, y1, z1)));
	buf.push_line(dbg!(vec3(x1, y1, z2)), dbg!(vec3(x2, y1, z2)));
	buf.push_line(dbg!(vec3(x2, y1, z2)), dbg!(vec3(x2, y2, z2)));
	buf.push_line(dbg!(vec3(x2, y2, z2)), dbg!(vec3(x1, y2, z2)));
	buf.push_line(dbg!(vec3(x1, y2, z2)), dbg!(vec3(x1, y1, z2)));
	buf.push_line(dbg!(vec3(x1, y1, z1)), dbg!(vec3(x1, y1, z2)));
	buf.push_line(dbg!(vec3(x2, y1, z1)), dbg!(vec3(x2, y1, z2)));
	buf.push_line(dbg!(vec3(x2, y2, z1)), dbg!(vec3(x2, y2, z2)));
	buf.push_line(dbg!(vec3(x1, y2, z1)), dbg!(vec3(x1, y2, z2)));

	let ctx = ctx();
	let vao = Arc::new(ctx.upload_meshbuffer(&buf));
	sg.push(Object::new(vao, ctx.shader_pack.lines(&ctx.fallback_texture)))
}
