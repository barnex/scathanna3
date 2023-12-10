use crate::prelude::*;

pub(crate) fn lightmap_size_for(opts: &BakeOpts, island: &GltfObject) -> u32 {
	let surface_area = island
		.primitives
		.iter()
		.map(|prim| prim.mesh.iter_triangle_positions())
		.flatten()
		.map(|tri| triangle_area(&tri))
		.sum::<f32>();

	let size = f32::sqrt(surface_area) * opts.lightmap_pix_per_m;
	let size = (size as u32).clamp(1, opts.max_lightmap_size);
	nearest_pow(size, 2)
}

// Triangle surface area
pub(crate) fn triangle_area(&[a, b, c]: &[vec3; 3]) -> f32 {
	(b - a).cross(c - a).len() / 2.0
}
