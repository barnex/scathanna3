use super::internal::*;

// Load an animation from `assets/obj/body.*.*.obj`.
//
// Walking has 6 keyframes. E.g. for body "torso1":
// 	`torso1.walk.0.obj`, `torso1.walk.1.obj`,... `torso1.walk.5.obj`
//
pub(crate) fn load_anim_vaos(body: &str) -> Result<AnimVAOs> {
	const WALK: &str = "walk"; // TODO: s/walk/kf/g
	const WALK_CYCLE: usize = 6;

	let meshbuffers = load_poses(&format!("{body}_{WALK}"), WALK_CYCLE)?;
	let meshbuffers = rescale_poses(meshbuffers);
	let walk = to_keyframe_vertices(&meshbuffers)?;

	Ok(AnimVAOs::new(walk))
}

// Load `n` keyframes of an animation cycle.
// E.g.: cycle_name: `torso1.walk`, n: 6
fn load_poses(cycle_name: &str, n: usize) -> Result<Vec<MeshBuffer>> {
	let poses = (0..n)
		.map(|i| format!("{cycle_name}_{i}"))
		.map(|name| load_wavefront_merged(&name))
		.collect::<Result<Vec<_>>>()?;

	//check_indices_per_frame(&poses)?;

	Ok(poses)
}

fn rescale_poses(walk: Vec<MeshBuffer>) -> Vec<MeshBuffer> {
	let bounds = BoundingBox::from_points(walk.iter().flat_map(|mesh| mesh.vertices.iter().map(|v| v.position))) //
		.unwrap_or(BoundingBox::new(default(), default()));
	let offset = bounds.min.y() * vec3::EY;
	let scale = 1.0 / bounds.size().y();
	walk.into_iter().map(|mesh| mesh.map_positions(|p| (p - offset) * scale)).collect::<Vec<_>>()
}

fn to_keyframe_vertices(poses: &[MeshBuffer]) -> Result<Vec<Arc<VAO>>> {
	let indices = poses[0].indices();
	check_indices_per_frame(poses)?;

	poses
		.iter()
		.enumerate()
		.map(|(i, pose)| -> Result<Arc<VAO>> {
			let next = &poses[wrap(i + 1, poses.len())];
			let host_vertices = to_keyframe_vertices_(pose, next)?;
			Ok(Arc::new(ctx().upload_vao(&host_vertices, indices)))
		})
		.collect::<Result<Vec<_>>>()
}

fn to_keyframe_vertices_(pose1: &MeshBuffer, pose2: &MeshBuffer) -> Result<Vec<VertexKF>> {
	Ok(pose1
		.vertices()
		.iter()
		.zip(pose2.vertices())
		.map(|(v1, v2)| VertexKF {
			texcoords: v1.texcoords,
			position1: v1.position,
			position2: v2.position,
			normal1: v1.normal,
			normal2: v2.normal,
		})
		.collect::<Vec<_>>())
}

fn check_indices_per_frame(poses: &[MeshBuffer]) -> Result<()> {
	for (i, pose) in poses.iter().enumerate().skip(1) {
		if pose.vertices.len() != poses[0].vertices.len() {
			return Err(anyhow!(
				"keyframe {i} has a different number of vertices: {:?}",
				(poses.iter().map(|p| p.vertices().len())).collect::<Vec<_>>()
			));
		}
		if pose.indices != poses[0].indices {
			return Err(anyhow!("keyframes have different indices"));
		}
	}
	Ok(())
}

fn wrap(i: usize, len: usize) -> usize {
	if i == len {
		0
	} else {
		i
	}
}

fn rescale_head(head: MeshBuffer) -> MeshBuffer {
	let bounds = BoundingBox::from_points(head.vertices.iter().map(|v| v.position)) //
		.unwrap_or(BoundingBox::new(default(), default()));
	let offset = bounds.min.y() * vec3::EY;
	let scale = 1.0 / bounds.size().y();
	head.map_positions(|p| (p - offset) * scale)
}
