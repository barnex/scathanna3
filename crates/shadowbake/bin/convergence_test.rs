/*
use crate::*;
use std::iter;

pub(crate) fn convergence_test(flags: Flags) -> Result<()> {
	for map_name in &flags.map_names {
		test_one(map_name)?;
	}
	Ok(())
}

fn test_one(map_name: &str) -> Result<()> {
	let map_dir = assets_dir().find_map_dir(&map_name);

	// 👉 Bake options as specified by the map.
	// Used only to get the sky color. Other options are overridden for testing.
	let map_bake_opts = load_bake_opts(&map_dir)?;

	// 👉 duplicated from shadowbake::convert_and_bake, without saving output:

	// 📄 parse
	println!("\nParsing GLTF:");
	let mut parsed_gltf = parse_gltf(&map_dir)?;

	// ✅ validate
	validate_gltf(&parsed_gltf)?;

	// 💾 metadata
	parsed_gltf.metadata.sky_color = map_bake_opts.sky_color; // 👈 TODO: clean way to specify skybox texture.
	parsed_gltf.metadata.save(&map_dir)?;

	// 🔺 scene mesh
	let parsed_gltf = ParsedGltf {
		objects: parsed_gltf.objects, // TODO: repack lightmap?
		metadata: parsed_gltf.metadata,
	};

	// 💡 bake gold

	let gold_samples = 100_001;
	let gold_bake_opts = BakeOpts {
		max_lightmap_size: 128,
		lightmap_pix_per_m: 2.0,
		fake_ambient: vec3::ZERO,
		min_samples: gold_samples,
		max_samples: gold_samples,
		target_error: 0.0,
		sun_diam_deg: 0.0,
		reflectivity_factor: 0.5,
		indirect_depth: 1,
		sky_color: map_bake_opts.sky_color,
		filter: false,
	};
	let gold_baked = bake(gold_bake_opts.clone(), parsed_gltf.clone())?;

	let mut result = vec![];

	//for log_samples in 1..14 {
	//let max_samples = 1 << log_samples;
	for max_samples in (1..30).step_by(1).chain((30..300).step_by(10)).chain((300..3000).step_by(100)) {
		let test_bake_opts = BakeOpts {
			max_samples,
			..gold_bake_opts.clone()
		};
		let test_baked = bake(test_bake_opts, parsed_gltf.clone())?;
		let num_rays = num_rays(&test_baked);

		let diff = diff(&gold_baked, &test_baked);
		println!("{num_rays} => {diff}");
		result.push((num_rays, diff));
	}

	let file = PathBuf::from(format!("convergence_{map_name}.txt"));
	create(&file)?.write_all(
		iter::once("# num_samples rms_diff".to_string())
			.chain(result.iter().map(|(n, diff)| format!("{n} {diff}\n")))
			.join("\n")
			.as_bytes(),
	)?;

	Ok(())
}

fn num_rays(lms: &Lightmaps) -> u64 {
	let mut sum = 0;

	for lm in lms.values() {
		sum += lm.sun_sph00.pixels().iter().map(|v| v.num_samples() as u64).sum::<u64>();
		sum += lm.point_lights.pixels().iter().map(|v| v.num_samples() as u64).sum::<u64>();
		sum += lm.indirect.pixels().iter().map(|v| v.num_samples() as u64).sum::<u64>();
	}

	sum
}

fn diff(la: &Lightmaps, b: &Lightmaps) -> f64 {
	let mut n = 0;
	let mut diff_squared = 0.0;

	for (handle, a) in la {
		let a = a.sum_all_diffuse().to_srgb();
		let b = b[handle].sum_all_diffuse().to_srgb();

		for (&a, &b) in a.pixels().zip(b.pixels()) {
			let a = Vector3::from(a.0).map(|v| (v as f32) / 255.0);
			let b = Vector3::from(b.0).map(|v| (v as f32) / 255.0);
			n += 3; // x, y, z components
			diff_squared += (b - a).len2() as f64;
		}
	}
	(diff_squared / (n as f64)).sqrt() // root-mean-square
}
*/