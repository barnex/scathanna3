use crate::prelude::*;

// Baking:
//
// Scene
//  -> Islands (share lightmap)
//    -> Meshes
//      ->  Triangles
//        -> Texels
// 		    -> Points
//

pub(crate) struct BakeState {
	pub scene: Scene,
	max_samples: u32,

	pub lightmap_sizes: HashMap<Handle, uvec2>,
	pub occupancy: HashMap<Handle, Img<Accum>>, // 👈 TODO: Img<f32>?, Img<vec3>?
	pub validity: HashMap<Handle, Img<Accum>>,

	// Intensity of the area lights (sky and interreflections),
	// weighted by cosine of angle between source and smoothed surface normal,
	// and integrated over the hemisphere defined by the smoothed surface normal:
	//
	//   1/π ∬ I cos θ_(smoothed normal)
	//
	// I.e., in a spherical harmonics decomposition of the (θ,φ)-dependent intensity,
	// this is the coefficient corresponding to Y_(1,0) (a.k.a, 'up' in the fragment-local frame).
	//
	// This is the lowest relevant SPH coefficient.
	// Y_(0,0) is never used (integrals always use a cosine kernel along some direction,
	// which all have zero weight for Y_(0,0)).
	//
	// This intensity is sufficient to compute the exact illumination of a perfectly diffuse surface
	// with smooth or flat normals, but without normal mapping.
	//
	pub area_sphz: HashMap<Handle, Img<Accum>>,

	pub area_sphx: HashMap<Handle, Img<Accum>>,
	pub area_sphy: HashMap<Handle, Img<Accum>>,

	pub sun_mask: HashMap<Handle, Img<Accum>>,
}

const SAMPLES_STEP: u32 = SUPERBLK * SUPERBLK;

impl BakeState {
	pub fn new(opts: BakeOpts, gltf: ParsedGltf) -> Result<Self> {
		let lightmap_sizes = lightmap_sizes(&opts, &gltf.objects);

		let black_lightmaps = lightmap_sizes.iter().map(|(&name, &size)| (name, Img::new(size))).collect();

		let scene = Scene::new(opts, gltf, black_lightmaps)?;

		// 🪲 TODO: use minimal samples!
		println!("⧄ bake occupancy");
		let occupancy = integrate(&scene, sample_occupancy);

		println!("☑ bake validity");
		let validity = integrate(&scene, sample_validity);

		Ok(Self {
			scene,
			max_samples: SAMPLES_STEP,
			lightmap_sizes,
			occupancy,
			validity,
			area_sphz: default(),
			area_sphx: default(),
			area_sphy: default(),
			sun_mask: default(),
		})
	}

	pub fn bake_full(&mut self) {
		self.advance_()
	}

	// 🪲 TODO: this is way too rough
	pub fn advance(&mut self) {
		self.scene.opts.max_samples = self.max_samples;
		self.advance_();
		self.max_samples *= 2;
	}

	fn advance_(&mut self) {
		println!("💡 bake sources");
		let start = Instant::now();

		println!("☀️ bake sun mask");
		let sun_mask = integrate(&self.scene, sample_sun_mask);

		println!("☀️ bake sun light");
		let sun = integrate(&self.scene, sample_sun); // ❗️TODO(performance): derive from sun_mask

		println!("💡 bake point lights");
		let point_lights = integrate(&self.scene, sample_point_lights);

		println!("📺 bake emissive");
		let emissive = integrate(&self.scene, sample_emissive);

		let mut indirect = self.lightmap_sizes.iter().map(|(&name, &size)| (name, Img::new(size))).collect();

		const MAX_INTENS: f32 = 2.0;

		for i in 0..self.scene.opts.indirect_depth {
			self.scene.temp_lightmap = clamp_lightmaps(
				add(
					&add(&add(&unweigh_lightmaps(&sun), &unweigh_lightmaps(&indirect)), &unweigh_lightmaps(&point_lights)),
					&unweigh_lightmaps(&emissive),
				),
				MAX_INTENS,
			);
			println!("🌥️ bake indirect {i}");
			indirect = integrate(&self.scene, sample_indirect);

			if self.scene.opts.filter {
				println!("🌥️ filter indirect {i}");
				indirect = superblock_filter(&indirect);
			}
		}

		// add emissive & point lights to sphz
		for (k, indirect) in indirect.iter_mut() {
			let emissive = &emissive[k];
			let point_lights = &point_lights[k];
			// 🪲 WRONG: should never add weighted lightmaps
			indirect.pixels_mut().iter_mut().zip(emissive.pixels().iter()).for_each(|(acc, em)| {
				acc.add_other(em);
			});
			indirect.pixels_mut().iter_mut().zip(point_lights.pixels().iter()).for_each(|(acc, pt)| {
				acc.add_other(pt);
			});
		}

		let (area_sphx, area_sphy) = if self.scene.opts.spherical_harmonics {
			// ❗️TODO(performance): bake together with sphz
			println!("🔮 spherical harmonics");
			(hole_filter(&integrate(&self.scene, sample_area_sphx)), hole_filter(&integrate(&self.scene, sample_area_sphy)))
		} else {
			(default(), default())
		};

		println!("⌛️ baked in {:.2}s", start.elapsed().as_secs_f64());

		println!("🔳 filling holes");
		let sun_mask = hole_filter(&sun_mask);
		let indirect = hole_filter(&indirect);

		self.area_sphz = indirect;
		self.sun_mask = sun_mask;

		self.area_sphx = area_sphx;
		self.area_sphy = area_sphy;
	}

	pub fn done(&self) -> bool {
		self.max_samples >= 3000 // 🪲 TODO: proper criterion: e.g. error estimate
	}
}

/// Clamp intensity to `max`. Used to limit dynamic range of intermediate lightmaps.
/// E.g. a surface close to a point light might receive an enormous intensity,
/// which would cause spurious bright spots in neighboring indirectly lit surfaces.
/// I.e. clamping the dynamic range trades noise for bias.
fn clamp_lightmaps(lights: HashMap<Handle, Img<vec3>>, max: f32) -> HashMap<Handle, Img<vec3>> {
	let mut lights = lights;
	for img in lights.values_mut() {
		for pix in img.pixels_mut() {
			*pix = pix.map(|v| v.clamp(0.0, max))
		}
	}
	lights
}

// Shadow rays start at a small offset from the emanating surface
// to avoid shadow acne.
const OFFSET: f32 = 1.0 / 8192.0;

fn add(a: &HashMap<Handle, Img<Color>>, b: &HashMap<Handle, Img<Color>>) -> HashMap<Handle, Img<Color>> {
	a.iter().map(|(&handle, img)| (handle, img + &b[&handle])).collect()
}

fn unweigh_lightmaps(sources: &HashMap<Handle, Img<Accum>>) -> HashMap<Handle, Img<Color>> {
	sources.iter().map(|(&name, img)| (name, unweigh_or_black(img))).collect()
}

fn integrate<F>(scene: &Scene, sample: F) -> HashMap<Handle, Img<Accum>>
where
	F: Fn(vec2, &Scene, &Primitive, Position, Normal, Tangents) -> Option<Color> + Send + Sync,
{
	struct WorkItem<'a> {
		lm_handle: Handle,
		primitive: &'a Primitive,
		tri: [u32; 3],
	}

	let mut work = vec![];

	for object in &scene.gltf.objects {
		let lm_handle = object.name;
		for primitive in &object.primitives {
			for tri in primitive.mesh.iter_triangle_indices() {
				work.push(WorkItem { lm_handle, primitive, tri });
			}
		}
	}

	let (send, recv) = crossbeam::channel::unbounded();

	for item in work {
		send.send(item).unwrap();
	}
	drop(send); // 👈 close sender so workers can terminate

	// each thread gets their own lightmap to write to
	let mut per_thread_lightmaps = vec![];
	let num_cpu = 16; // 🪲 TODO: use real number of CPUs!
	for _i in 0..num_cpu {
		per_thread_lightmaps.push(scene.temp_lightmap.iter().map(|(&name, img)| (name, Img::new(img.size()))).collect::<HashMap<Handle, Img<Accum>>>());
	}

	// parallel bake
	let mut per_thread_lightmaps = per_thread_lightmaps
		.into_par_iter()
		.map(|mut dst| {
			for WorkItem { lm_handle, primitive, tri } in recv.iter() {
				bake_triangle_with(dst.get_mut(&lm_handle).unwrap(), scene, primitive, &tri, &sample)
			}
			dst
		})
		.collect::<Vec<HashMap<Handle, Img<Accum>>>>();

	// now sum the per-thread lightmaps
	let mut sum = per_thread_lightmaps.pop().expect("have lightmaps");
	for lm in per_thread_lightmaps {
		for (name, dst) in sum.iter_mut() {
			let src = &lm[name];
			dst.pixels_mut().iter_mut().zip(src.pixels().iter()).for_each(|(dst, src)| dst.add_other(src));
		}
	}
	sum
}

pub const SUPERBLK: u32 = 3;

fn bake_triangle_with<F>(dst: &mut Img<Accum>, scene: &Scene, primitive: &Primitive, tri: &[u32; 3], sample: F)
where
	F: Fn(vec2, &Scene, &Primitive, Position, Normal, Tangents) -> Option<Color>,
{
	debug_assert!(dst.width() == dst.height());

	let mesh = &primitive.mesh;
	let tri_uvs = mesh.triangle_lightcoords(&tri);

	for (pix, texel_uv_range) in conservative_raster(&tri_uvs, dst.width()) {
		if let Some((_uv, pos, normal, tangents)) = sample_in_triangle_center(mesh, tri, &tri_uvs, &texel_uv_range) {
			let sudoku = (pix.x() % SUPERBLK) + SUPERBLK * (pix.y() % SUPERBLK);
			assert!(sudoku < SUPERBLK * SUPERBLK);

			for halton_i in 0..(scene.opts.max_samples as u32) {
				let rand = halton25((SUPERBLK * SUPERBLK) * halton_i + sudoku).into();
				if let Some(color) = sample(rand, scene, primitive, pos, normal, tangents) {
					if color.is_finite() {
						dst.at_mut(pix).add(color);
					}
				}

				if halton_i > scene.opts.min_samples as u32 {
					if halton_i % 25 == 0 {
						if let Some(err) = dst.ref_at(pix).error() {
							if err < scene.opts.target_error {
								break;
							}
						}
					}
				}
			}
		}
	}
}

/// ℹ️ For debugging: occupancy reveals which fraction of a texel is covered by triangles.
/// 0      : fully outside of any triangle
/// (0..1) : on the edge of a triangle
///      1 : inside a triangle
///     >1 : inside more than one triangle => bad lightmap coordinates
#[inline]
fn sample_occupancy(_rand: vec2, _: &Scene, _: &Primitive, _: vec3, _: vec3, _: Tangents) -> Option<vec3> {
	Some(vec3::ONES)
}

/// ℹ️ For debugging: validity reveals if lightmaps should be calculated in a point.
/// In a non-manifold geometry, some points are underneath another object
/// (e.g. a crate standing on a floor). Light intensity in such points is zero,
/// but this value must *not* be added to the lightmap because it leaks into nearby texels
/// due to finite resolution (shots/077-test05-validity.png).
#[inline]
fn sample_validity(_rand: vec2, scene: &Scene, _: &Primitive, pos: vec3, normal: vec3, _: Tangents) -> Option<vec3> {
	/*

	Send a probe ray perpendicular to the surface.
	Point (b) is not valid because it lies inside an object,
	revealed by dot(ray_dir, normal) > 0.


							   +--------+
							   |        |
				 +--------+    +--------+
		 ^       |   ^    |      ^
		 |       |   |    |      |
	-----a-------+---b----+------c-------

	*/
	let ray = Ray::new(pos + OFFSET * normal, normal);

	let hr = scene.faces.intersection(&ray);
	if let Some((hit_normal, _texcoord, _lightcoord, _mathandle, _lmhandle)) = &hr.attrib {
		if !is_valid_sampling_point(&ray, hit_normal) {
			// (b) ray hits something from the inside: invalid
			Some(vec3::ZERO)
		} else {
			// (c) ray hits something from the outside: valid
			Some(vec3::ONES)
		}
	} else {
		// (a) ray hits nothing: valid
		Some(vec3::ONES)
	}
}

#[inline]
fn sample_emissive(_rand: vec2, scene: &Scene, prim: &Primitive, _pos: vec3, _normal: vec3, _: Tangents) -> Option<vec3> {
	// 🪲 hack for emissive materials util we have an emissive shader:
	// apply light to the base_color texture. Works well enough for lava.
	scene.gltf.metadata.materials[&prim.material].emissive.as_ref().map(
		//
		|em| vec3::repeat(em.emissive_strength),
	)
}

//#[inline]
//fn sample_sun_and_lights(rand: vec2, scene: &Scene, pos: vec3, normal: vec3) -> Option<vec3> {
//	let sun = sample_sun(rand, scene, pos, normal);
//	let lights = sample_point_lights(rand, scene, pos, normal);
//	match (sun, lights) {
//		(None, None) => None,
//		_ => Some(sun.unwrap_or_default() + lights.unwrap_or_default()),
//	}
//}

#[inline]
fn sample_point_lights(rand: vec2, scene: &Scene, _: &Primitive, pos: vec3, normal: vec3, _: Tangents) -> Option<vec3> {
	let mut accum = None;
	for light in &scene.gltf.metadata.point_lights {
		// 🪲 TODO: consider distance
		if let Some(light) = sample_point_light(rand, scene, light, pos, normal) {
			accum = Some(accum.unwrap_or_default() + light)
		}
	}
	accum
}

fn sample_point_light(_rand: vec2, scene: &Scene, light: &PointLightDef, pos: vec3, normal: vec3) -> Option<vec3> {
	let to_light_unnormalized = light.pos - pos;
	let dist2 = to_light_unnormalized.len2();
	let light_dist = dist2.sqrt();
	let to_light_normalized = to_light_unnormalized / light_dist;
	let cos_theta = normal.dot(to_light_normalized);

	if cos_theta < 0.0 {
		// our surface element points away from the light, so it is definitely not illuminated.
		// no need to intersect a shadow ray. Expected ~50% speed-up.
		return Some(vec3::ZERO); // 🪲 wrong
	}

	// Shadow ray.
	let ray = Ray::new(pos + OFFSET * normal, to_light_normalized);

	let hr = scene.faces.intersection(&ray);
	if let Some((hit_normal, _texcoord, _lightcoord, _mathandle, _lmhandle)) = &hr.attrib {
		if !is_valid_sampling_point(&ray, hit_normal) {
			None
		} else {
			if hr.t < light_dist {
				// ray intersects scene *between* light and sampling point: occluded.
				Some(vec3::ZERO)
			} else {
				// ray intersects scene *behind* light: not occluded.
				Some(light.color * (cos_theta / dist2))
			}
		}
	} else {
		Some(light.color * (cos_theta / dist2))
	}
}

#[inline]
fn sample_sun(rand: vec2, scene: &Scene, _: &Primitive, pos: vec3, normal: vec3, _: Tangents) -> Option<vec3> {
	let sun_def = scene.gltf.metadata.sun_def.as_ref()?;

	// Direction used for intensity ignores sun radius and random sampling.
	// This averages out to almost exactly the correct result,
	// but avoids unnecessary noise in 100% lit regions (i.e. outside of penumbrae).
	let to_sun = -sun_def.dir;
	let cos_theta = normal.dot(to_sun);

	if cos_theta < 0.0 {
		// our surface element points away from the sun, so it is definitely not illuminated.
		// no need to intersect a shadow ray. Expected ~50% speed-up.
		return Some(vec3::ZERO); // 🪲 WRONG !!!! must determine validity ahead of time!
	}

	// Shadow ray.
	// Random draw from the surface of a disk.
	let sun_radius = scene.opts.sun_diam_deg * DEG / 2.0;
	let to_sun_jittered = (to_sun + sun_radius * disk_with_normal(rand, to_sun)).normalized();

	let ray = Ray::new(pos + OFFSET * normal, to_sun_jittered);

	let hr = scene.faces.intersection(&ray);
	if let Some((hit_normal, _texcoord, _lightcoord, _mathandle, _lmhandle)) = &hr.attrib {
		if !is_valid_sampling_point(&ray, hit_normal) {
			None
		} else {
			Some(vec3::ZERO)
		}
	} else {
		Some(sun_def.color * cos_theta)
	}
}

/// Sun intensity, irrespective of normal vector.
/// The cos θ factor is added by the fragment shader, using normals from the normal map.
/// Thus, the sun mask only tells the shader how much of the sunlight was occluded.
/// TODO(performance): this can be a monochrome texture instead.
#[inline]
fn sample_sun_mask(rand: vec2, scene: &Scene, _: &Primitive, pos: vec3, normal: vec3, _: Tangents) -> Option<vec3> {
	let sun_def = scene.gltf.metadata.sun_def.as_ref()?;

	// Direction used for intensity ignores sun radius and random sampling.
	// This averages out to almost exactly the correct result,
	// but avoids unnecessary noise in 100% lit regions (i.e. outside of penumbrae).
	let to_sun = -sun_def.dir;
	let cos_theta = normal.dot(to_sun);

	if cos_theta < 0.0 {
		// our surface element points away from the sun, so it is definitely not illuminated.
		// no need to intersect a shadow ray. Expected ~50% speed-up.
		return Some(vec3::ZERO); // 🪲 WRONG !!!! must determine validity ahead of time!
	}

	// Shadow ray.
	// Random draw from the surface of a disk.
	let sun_radius = scene.opts.sun_diam_deg * DEG / 2.0;
	let to_sun_jittered = (to_sun + sun_radius * disk_with_normal(rand, to_sun)).normalized();

	let ray = Ray::new(pos + OFFSET * normal, to_sun_jittered);

	let hr = scene.faces.intersection(&ray);
	if let Some((hit_normal, _texcoord, _lightcoord, _mathandle, _lmhandle)) = &hr.attrib {
		if !is_valid_sampling_point(&ray, hit_normal) {
			None
		} else {
			Some(vec3::ZERO)
		}
	} else {
		Some(sun_def.color) // 👈 note: no cos θ, done in fragment shader
	}
}

#[inline]
fn sample_indirect(rand: vec2, scene: &Scene, _prim: &Primitive, pos: vec3, normal: vec3, _: Tangents) -> Option<vec3> {
	// Emissive material gets no indirect light
	// TODO: we could add an inverted ambient occlusion term,
	// to make lava glow more at the edges.
	// if let Some(emissive) = scene.gltf.metadata.materials[&prim.material].emissive.as_ref() {
	// 	return Some(scene.base_colors[emissive.emissive_texture].at_uv_nearest_clamp(*light))
	// }

	let dir = cosine_sphere(rand, normal);
	let ray = Ray::new(pos + OFFSET * normal, dir);

	let hr = scene.faces.intersection(&ray);
	if let Some((hit_normal, texcoord, lightcoord, mathandle, lmhandle)) = &hr.attrib {
		if !is_valid_sampling_point(&ray, hit_normal) {
			None
		} else {
			let light = scene.temp_lightmap[lmhandle].at_uv_nearest_clamp(*lightcoord);
			let base_color = scene.base_colors[mathandle].at_uv_nearest_wrap(*texcoord);
			Some(light * base_color * scene.opts.reflectivity_factor)
		}
	} else {
		Some(scene.gltf.metadata.sky_color)
	}
}

#[inline]
fn sample_area_sphx(rand: vec2, scene: &Scene, prim: &Primitive, pos: vec3, normal: vec3, tangents: Tangents) -> Option<vec3> {
	sample_area_sph(rand, scene, prim, pos, normal, tangents, tangents.0)
}

#[inline]
fn sample_area_sphy(rand: vec2, scene: &Scene, prim: &Primitive, pos: vec3, normal: vec3, tangents: Tangents) -> Option<vec3> {
	sample_area_sph(rand, scene, prim, pos, normal, tangents, tangents.1)
}

#[inline]
fn sample_area_sph(rand: vec2, scene: &Scene, _prim: &Primitive, pos: vec3, normal: vec3, _tangents: Tangents, tangent: vec3) -> Option<vec3> {
	// ☠️ TODO: copied from indirect
	let dir = cosine_sphere(rand, normal);
	let ray = Ray::new(pos + OFFSET * normal, dir);

	let hr = scene.faces.intersection(&ray);

	let indirect = if let Some((hit_normal, texcoord, lightcoord, mathandle, lmhandle)) = &hr.attrib {
		if !is_valid_sampling_point(&ray, hit_normal) {
			None
		} else {
			let light = scene.temp_lightmap[lmhandle].at_uv_nearest_clamp(*lightcoord);
			let base_color = scene.base_colors[mathandle].at_uv_nearest_wrap(*texcoord);
			Some(light * base_color * scene.opts.reflectivity_factor)
		}
	} else {
		Some(scene.gltf.metadata.sky_color)
	};

	indirect.map(|c| c * dir.dot(tangent))
}

/// In a non-manifold geometry, a ray emanating from one object may start from inside another object.
/// With an infinite resolution lightmap, the illumination seen by such ray (usually zero) would be hidden
/// from us (corresponding lightmap pixel lies inside another object). However, with a finite-sized
/// lightmap, the illumination can partially "leak" outside (when the corresponding lightmap pixel
/// lies only partially inside the enclosing object).
///
/// Therefore, we cull rays that start inside other objects. `valid_sampling_point` tests for insideness in other objects.
///
/// 🪲 TODO: use geometric normal, not shading normal!
#[inline]
fn is_valid_sampling_point(ray: &Ray<f32>, hit_normal: &vec3) -> bool {
	// Assumes the enclosing object is closed (or at least encloses the hemisphere around the ray's direction),
	// which is always the case in real game maps.
	hit_normal.dot(ray.dir) < 0.0
}

/// Sample a 3D point inside the given triangle AND texel range
/// (which must overlap -- see conservative rasterization).
/// Uses deterministic sampling via halton(5,7);
fn __sample_in_triangle_halton(mesh: &MeshBuffer2, tri: &[u32; 3], tri_uvs: &[vec2; 3], texel_uv_range: &Bounds2D<f32>, halton_i: u32) -> Option<(TexCoord, Position, Normal)> {
	let rs = vec2(halton(5, halton_i), halton(7, halton_i));
	let sampling_uv = texel_uv_range.min + rs * texel_uv_range.size();
	debug_assert!(texel_uv_range.contains(sampling_uv));

	let barycentric = barycentric_coordinates(&tri_uvs, sampling_uv);
	if barycentric.iter().all(|&v| v >= 0.0 && v <= 1.0) {
		let normal = barycentric_interpolation(&mesh.triangle_normals(&tri), &barycentric).normalized();
		let pos = barycentric_interpolation(&mesh.triangle_positions(&tri), &barycentric);
		return Some((sampling_uv, pos, normal));
	}
	None
}
pub type Tangents = (vec3, vec3);

fn sample_in_triangle_center(mesh: &MeshBuffer2, tri: &[u32; 3], tri_uvs: &[vec2; 3], texel_uv_range: &Bounds2D<f32>) -> Option<(TexCoord, Position, Normal, Tangents)> {
	let rs = vec2(0.499, 0.501); // 👈 tiny offset from center because it's very common for a triangle edge to cut straight through the center, causing ambiguity.
	let sampling_uv = texel_uv_range.min + rs * texel_uv_range.size();
	debug_assert!(texel_uv_range.contains(sampling_uv));

	let barycentric = barycentric_coordinates(&tri_uvs, sampling_uv);
	if barycentric.iter().all(|&v| v >= 0.0 && v <= 1.0) {
		let normal = barycentric_interpolation(&mesh.triangle_normals(&tri), &barycentric).normalized();
		let tangent = mesh
			.triangle_tangents(&tri)
			.map(|tangents| barycentric_interpolation(&tangents, &barycentric).normalized())
			.unwrap_or_default();
		let bitangent = mesh
			.triangle_bitangents(&tri)
			.map(|bitangents| barycentric_interpolation(&bitangents, &barycentric).normalized())
			.unwrap_or_default();
		let tangents = (tangent, bitangent);

		let pos = barycentric_interpolation(&mesh.triangle_positions(&tri), &barycentric);
		return Some((sampling_uv, pos, normal, tangents));
	}
	None
}

/// Iterator yields pixels (indices and corresponding UV ranges) that touch a triangle
/// (conservative rasterization).
fn conservative_raster(tri_uvs: &[vec2; 3], lm_size: u32) -> impl Iterator<Item = (uvec2, Bounds2D<f32>)> {
	let tri_uvs = tri_uvs.clone();
	let triangle_pix_bounds = triangle_conservative_bounds(&tri_uvs, lm_size);
	triangle_pix_bounds.iter_incl().filter_map(move |pix| {
		let texel_uv_range = Bounds2D::new(pix, pix + (1, 1)).map(|v| (v.clamp(0, lm_size) as f32 / lm_size as f32));
		if triangle_overlaps_texel(&tri_uvs, &texel_uv_range) {
			Some((pix, texel_uv_range))
		} else {
			None
		}
	})
}

// Conservative bounding box, in pixels, that contains the triangle (coordinates 0.0..1.0) entirely.
fn triangle_conservative_bounds(tri_uvs: &[vec2; 3], lm_size: u32) -> Bounds2D<u32> {
	let Bounds2D { min, max } = triangle_bounds(&tri_uvs);
	let min = min.map(|v| ((v * (lm_size as f32)) as i32).clamp(0, lm_size as i32 - 1) as u32);
	let max = max.map(|v| ((v * (lm_size as f32)) as i32 + 1).clamp(0, lm_size as i32 - 1) as u32);
	Bounds2D { min, max }
}

/// Conservative rasterization (volume): does a triangle contain overlap with a texel (even by a small amount)?
#[inline]
fn triangle_overlaps_texel(tri: &[vec2; 3], texel: &Bounds2D<f32>) -> bool {
	is_inside(tri, texel.center()) || edge_overlaps_texel(tri, texel)
}

/// Conservative rasterization (edges): does a triangle edge touch a texel (even by a small amount)?
#[inline]
fn edge_overlaps_texel(&[a, b, c]: &[vec2; 3], texel: &Bounds2D<f32>) -> bool {
	texel.intersects_segment(a, b) || texel.intersects_segment(b, c) || texel.intersects_segment(c, a)
}

// Bounding box around a triangle.
fn triangle_bounds(points: &[vec2; 3]) -> Bounds2D<f32> {
	let xs = points.map(|p| p.x());
	let ys = points.map(|p| p.y());

	let x_min = xs.iter().copied().reduce(f32::min).unwrap();
	let x_max = xs.iter().copied().reduce(f32::max).unwrap();
	let y_min = ys.iter().copied().reduce(f32::min).unwrap();
	let y_max = ys.iter().copied().reduce(f32::max).unwrap();

	Bounds2D::new(vec2(x_min, y_min), vec2(x_max, y_max))
}

// Map objects to corresponding lightmap size (larger objects get larger lightmaps).
pub(crate) fn lightmap_sizes(opts: &BakeOpts, gltf_objects: &[GltfObject]) -> HashMap<Handle, uvec2> {
	gltf_objects.iter().map(|obj| (obj.name, uvec2::repeat(lightmap_size_for(opts, obj)))).collect()
}
