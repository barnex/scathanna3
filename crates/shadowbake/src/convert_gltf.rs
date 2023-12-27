use crate::prelude::*;

pub fn convert_only(map_name: &str, opts: &BakeOpts) -> Result<ParsedGltf> {
	let map_dir = assets_dir().find_map_dir(map_name);

	println!("\n⚠️ NOT cleaning map dir, converting only, keeping lightmaps");

	// 📄 parse
	println!("\nParsing GLTF:");
	let mut parsed_gltf = parse_gltf(&map_dir)?;

	// ✅ validate
	validate_gltf(&parsed_gltf)?;

	// 💾 metadata
	parsed_gltf.metadata.sky_color = opts.sky_color;
	parsed_gltf.metadata.sky_box = opts.sky_box.clone();
	print_metadata(&parsed_gltf);
	parsed_gltf.metadata.save(&map_dir)?;

	// 🔺 save meshes
	print_scene_objects(&parsed_gltf);
	let face_tree = convert_collider_mesh(&parsed_gltf.objects);
	save_collider_mesh(&map_dir, &face_tree)?;

	let parsed_gltf = ParsedGltf {
		objects: parsed_gltf.objects, // TODO: repack lightmap?
		metadata: parsed_gltf.metadata,
	};
	save_scene_mesh(&map_dir, &parsed_gltf.objects)?;

	Ok(parsed_gltf)
}

pub fn convert_and_bake(map_name: &str, opts: &BakeOpts, iterative: bool) -> Result<()> {
	let map_dir = assets_dir().find_map_dir(map_name);

	// 🧹 clean
	println!("\nCleaning {map_dir:?}");
	clean_map_dir(&map_dir)?;

	let parsed_gltf = convert_only(map_name, opts)?;

	// 💡 bake + save
	println!("bake options: {:#?}", &opts);
	let mut state = BakeState::new(opts.clone(), parsed_gltf)?;
	if false {
		// << disabling debug
		save_validity(&map_dir, &state.validity)?;
		save_occupancy(&map_dir, &state.occupancy)?;
	}

	if iterative {
		while !state.done() {
			state.advance();
			save_lightmaps(&map_dir, &state)?;
		}
	} else {
		state.bake_full();
		save_lightmaps(&map_dir, &state)?;
	}

	Ok(())
}

fn save_lightmaps(map_dir: &MapDir, baked: &BakeState) -> Result<()> {
	println!("\nSaving lightmaps");

	for (lm_name, img) in baked.area_sphz.iter() {
		save_lightmap_with_debug(map_dir, AREA_SPHZ, lm_name, img)?;
	}

	for (lm_name, img) in baked.sun_mask.iter() {
		save_lightmap_with_debug(map_dir, SUN_MASK, lm_name, img)?;
	}

	for (lm_name, sphx) in baked.area_sphx.iter() {
		let sphy = &baked.area_sphy[lm_name];
		save_sph(map_dir, AREA_SPHX, lm_name, sphx)?;
		save_sph(map_dir, AREA_SPHY, lm_name, sphy)?;
	}

	Ok(())
}

fn save_validity(map_dir: &MapDir, validity: &HashMap<Handle, Img<Accum>>) -> Result<()> {
	for (lm_name, lightmap) in validity.iter() {
		save_lightmap_with_debug(map_dir, VALIDITY, lm_name, lightmap)?; // 👈 for debug
	}
	Ok(())
}

fn save_occupancy(map_dir: &MapDir, occupancy: &HashMap<Handle, Img<Accum>>) -> Result<()> {
	for (lm_name, lightmap) in occupancy.iter() {
		let max_occupancy = lightmap.pixels().iter().map(|v| v.num_samples()).max().unwrap_or(1) as f32;
		let occupancy = lightmap.map_to_srgb(|v| vec3::repeat(v.num_samples() as f32 / max_occupancy));
		save_image(&map_dir.lm_file(OCCUPANCY, lm_name.as_str()), &occupancy)?;
	}
	Ok(())
}

fn save_lightmap_with_debug(map_dir: &MapDir, infix: &str, lm_name: &Handle, lightmap: &Img<Accum>) -> Result<()> {
	// 🪲 TODO: Don't save stderr for validity, occupancy

	save_image(&map_dir.lm_file(infix, lm_name.as_str()), &unweigh(&lightmap).to_srgb())?;

	if false {
		// <<<< disbling debug output
		let max = lightmap.pixels().iter().map(|pix| pix.num_samples()).max().unwrap_or(1) as f32;
		let scaled = lightmap.map_to_srgb(|pix| vec3::repeat(pix.num_samples() as f32) / max);
		save_image(&map_dir.lm_file(&(infix.to_string() + "_samples"), lm_name.as_str()), &scaled)?;
	}

	if false {
		// <<<< disbling debug output
		let max = lightmap.pixels().iter().filter_map(|pix| pix.error()).max_by(|a, b| partial_max(a, b)).unwrap_or(1.0) as f32;
		let scaled = lightmap.map_to_srgb(|pix| vec3::repeat(pix.error().unwrap_or_default()) / max);
		save_image(&map_dir.lm_file(&(infix.to_string() + "_error"), lm_name.as_str()), &scaled)?;
	}

	Ok(())
}

// Save spherical harmonics (X and Y), in linear color space like normal maps (0u8 => -1.0, 255u8 => 1.0).
fn save_sph(map_dir: &MapDir, infix: &str, lm_name: &Handle, lightmap: &Img<Accum>) -> Result<()> {
	let file = map_dir.lm_file(infix, lm_name.as_str());
	//println!("  💾 {:?}: {}x{}", fname.file_name().unwrap_or_default(), img.width(), img.height());
	save_image(
		&file,
		&lightmap
			.map(|accum| {
				accum //
					.avg()
					.unwrap_or_default()
					.map(|v| 0.5 * v + 0.5)
					.map(|v| (v * 255.0).clamp(0.0, 255.0) as u8)
			})
			.to_rgb_image(),
	)
}

fn save_image(fname: &Path, img: &RgbImage) -> Result<()> {
	let mut f = create(&fname)?;
	println!("  💾 {:?}: {}x{}", fname.file_name().unwrap_or_default(), img.width(), img.height());
	img.write_to(&mut f, image::ImageOutputFormat::Png).with_context(|| format!("write {fname:?}"))?;
	Ok(())
}

fn print_metadata(parsed_gltf: &ParsedGltf) {
	println!("\nMetadata: {:#?}", &parsed_gltf.metadata);
}

fn print_scene_objects(parsed_gltf: &ParsedGltf) {
	println!("\nScene objects ({} total):", &parsed_gltf.objects.len());
	for obj in &parsed_gltf.objects {
		println!("    🔲 {}", obj.name);
	}
}

// Place GLTF triangles in a BVH tree for efficient collision detection.
fn convert_collider_mesh(gltf_objects: &[GltfObject]) -> Node<ColliderFace> {
	println!("\nConverting collider mesh");
	let mut collider_faces = vec![];
	for GltfObject { primitives: meshes, .. } in gltf_objects {
		for Primitive { material, mesh } in meshes {
			let material = *material;
			for indices in mesh.iter_triangle_indices() {
				let vert = indices.map(|i| mesh.positions[i as usize]);
				collider_faces.push(ColliderFace { vert, material });
			}
		}
	}
	Node::build_tree(collider_faces)
}

fn save_collider_mesh(map_dir: &MapDir, tree: &Node<ColliderFace>) -> Result<()> {
	save_bincode_gz(tree, &map_dir.collider_mesh_file())
}

fn clean_map_dir(map_dir: &MapDir) -> Result<()> {
	let paths = [map_dir.lightmap_dir(), map_dir.collider_mesh_file(), map_dir.metadata_file(), map_dir.scene_mesh_file()];

	for path in &paths {
		match force_remove(path) {
			Ok(()) => println!("  ✅ {path:?}"),
			Err(e) => {
				println!("  ❌ rm {path:?}: {e}");
				bail!("{e}");
			}
		}
	}
	Ok(())
}

// divide colors by number of samples. zero samples => red (for debug).
fn _unweigh_or_red(img: &Img<Accum>) -> Img<vec3> {
	img.map(|weighted| weighted.avg().unwrap_or(RED))
}

// divide colors by number of samples. zero samples => black.
pub(crate) fn _unweigh_or_black(img: &Img<Accum>) -> Img<vec3> {
	img.map(|weighted| weighted.avg().unwrap_or_default())
}


// divide colors by number of samples. zero samples => black.
pub(crate) fn unweigh(img: &Img<Accum>) -> Img<vec3> {
	leak_filter(&img.map(|weighted| weighted.avg())).map(|v|v.unwrap_or(vec3::ZERO))
}

fn leak_filter(img: &Img<Option<vec3>>) -> Img<Option<vec3>> {
	let mut result = Img::new(img.size());

	let (w, h) = img.size().into();
	for dst_idx in cross(0..w, 0..h) {
		let dst_idx = uvec2::from(dst_idx);
		let (w, h) = (w as i32, h as i32);
		result.set(dst_idx, img.at(dst_idx));
		if img.at(dst_idx).is_none() {
			let mut n = 0;
			let mut sum = vec3::ZERO;
			for delta in cross(-1..=1, -1..=1) {
				let delta = ivec2::from(delta);
				let src_idx = dst_idx.convert() + delta;
				if src_idx.x() >= 0 && src_idx.x() < w && src_idx.y() >= 0 && src_idx.y() < h {
					if let Some(c) = img.at(src_idx.map(|v| v as u32)) {
						n += 1;
						sum += c;
					}
				}
			}
			if n != 0 {
				result.set(dst_idx, Some(sum / (n as f32)));
			}
		}
	}

	result
}
