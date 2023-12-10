use super::internal::*;
use std::ffi::OsStr;

// Obsolete!  uses materials dir.  Should use MaterialDefs from metadata instead.
pub struct HostMaterial {
	pub diffuse: RgbImage,
	pub avg_diffuse: vec3,
	pub normal_vec: Option<Img<vec3>>,
	pub emissive: Option<RgbImage>,
	pub avg_emissive: vec3,
}

impl HostMaterial {
	// TODO: refuse to load non-power-of-two textures (crashes mipmap).
	pub fn load(opts: &GraphicsOpts, materials_dir: &Path, name: &str) -> Result<Self> {
		LOG.write(format!("loading material {name}"));

		// scan files for any of the patterns (e.g. "-normal"),
		// upload the first match GPU.
		fn find_texture(mat_dir: &Path, files: &[PathBuf], patterns: &[&str]) -> Result<RgbImage> {
			find_pattern(files, patterns) //
				.map(|file| mat_dir.join(file))
				.and_then(|path| Ok((image::open(&path)?.into_rgb8(), path)))
				.and_then(|(img, path)| match uvec2::from(img.dimensions()).iter().all(|v| v.is_power_of_two()) {
					true => Ok(img),
					false => Err(anyhow!("non-power-of-two image size: {path:?}")),
				})
		}

		// find a file containing any of the given patterns. E.g.:
		//	"-normal" => "brick_wall-normal.png"
		fn find_pattern<'f>(files: &'f [PathBuf], patterns: &[&str]) -> Result<&'f OsStr> {
			files //
				.iter()
				.filter_map(|file| file.file_name())
				.find(|name| contains_any_ignore_case(name, patterns))
				.ok_or_else(|| anyhow!("material texture not found"))
		}

		fn contains_any_ignore_case(p: &OsStr, patterns: &[&str]) -> bool {
			patterns //
				.iter()
				.any(|pat| p.to_string_lossy().to_ascii_lowercase().contains(pat))
		}

		let mat_dir = materials_dir.join(name);
		let files = read_dir_names(&mat_dir)?.collect::<Vec<_>>();

		let diffuse = match opts.textures {
			true => find_texture(&mat_dir, &files, &["_basecolor", "-basecolor", "_diffuse", "-diffuse", "_base color", "base_color", "_albedo", "_d."])?,
			false => RgbImage::from_pixel(1, 1, Rgb([160, 160, 160])),
		};

		let avg_diffuse = average(&diffuse);
		let normal_map = match opts.normal_maps {
			true => find_texture(&mat_dir, &files, &["_normal", "-normal", "_n."]).ok(),
			false => None,
		};
		let normal_vec = normal_map.as_ref().map(decode_normal_map);
		let emissive = find_texture(&mat_dir, &files, &["_emissive", "-emissive"]).ok();
		let avg_emissive = emissive.as_ref().map(average).unwrap_or_default();

		Ok(Self {
			diffuse,
			avg_diffuse,
			normal_vec,
			emissive,
			avg_emissive,
		})
	}

	pub fn fallback() -> Self {
		Self {
			diffuse: fallback_image(),
			avg_diffuse: vec3::ONES,
			normal_vec: None,
			emissive: None,
			avg_emissive: vec3::ZERO,
		}
	}
}

fn average(img: &RgbImage) -> vec3 {
	let sum = img.pixels().map(|&Rgb([r, g, b])| Vector3::new(r, g, b).map(srgb_to_linear)).sum::<vec3>();
	let n = img.pixels().len() as f32;
	sum / n
}
