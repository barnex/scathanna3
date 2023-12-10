use crate::prelude::*;

/// Options for lightmap baking.
/// Stored per game map (map_dir/baking.ron).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BakeOpts {
	/// Maximum lightmap size (width and height) in pixels.
	/// E.g. 4096 for max size of 4096x4096.
	#[serde(default = "default_max_lightmap_size")]
	pub max_lightmap_size: u32,

	/// Lightmap resolution: approximate lightmap pixels per meter.
	#[serde(default = "default_lightmap_pix_per_m")]
	pub lightmap_pix_per_m: f32,

	/// Tiny amount of fake ambient light to help with extremely dark areas.
	#[serde(default = "default_fake_ambient")]
	pub fake_ambient: vec3,

	#[serde(default = "default_max_samples")]
	pub max_samples: u32,

	#[serde(default = "default_min_samples")]
	pub min_samples: u32,

	#[serde(default = "default_target_error")]
	pub target_error: f32,

	/// Sun size in degrees. Unfortunately GLTF does not provide light sizes.
	#[serde(default = "default_sun_diam_deg")]
	pub sun_diam_deg: f32,

	/// Scale diffuse reflectivity by this factor (0..1)
	/// for indirect (scattered) lighting.
	/// E.g. real-world white plaster only reflects only ~50% of light,
	/// but would be represented as a 100% white texture
	/// -- hence the need for this factor.
	/// 0.5 is a good value.
	/// Lower yields stronger ambient occlusion,
	/// higher yields a flatter, lighter look (good for e.g. deep corridors).
	#[serde(default = "default_reflectivity_factor")]
	pub reflectivity_factor: f32,

	#[serde(default = "default_indirect_depth")]
	pub indirect_depth: u32,

	#[serde(default = "default_sky_color")]
	pub sky_color: vec3,

	#[serde(default)]
	pub sky_box: Option<String>,


	#[serde(default = "default_true")]
	pub filter: bool,

	#[serde(default = "default_false")]
	pub spherical_harmonics: bool,
}

// Load map_dir/baking.ron, or create default if it does not yet exist.
pub fn load_bake_opts(map_dir: &MapDir) -> Result<BakeOpts> {
	if !map_dir.baking_file().exists() {
		save_ron(&map_dir.baking_file(), &BakeOpts::default())?;
	}
	load_ron(&map_dir.baking_file())
}

fn default_sun_diam_deg() -> f32 {
	0.54 /* https://en.wikipedia.org/wiki/Sun */
}

fn default_max_samples() -> u32 {
	300
}

fn default_min_samples() -> u32 {
	25
}

fn default_target_error() -> f32 {
	0.05
}

fn default_max_lightmap_size() -> u32 {
	2048
}

fn default_lightmap_pix_per_m() -> f32 {
	8.0
}

fn default_reflectivity_factor() -> f32 {
	0.5
}

fn default_fake_ambient() -> vec3 {
	0.001 * vec3::ONES
}

fn default_indirect_depth() -> u32 {
	3
}

fn default_sky_color() -> vec3 {
	vec3(0.0, 0.0, 0.0)
}

fn default_true() -> bool {
	true
}

fn default_false() -> bool {
	false
}

impl Default for BakeOpts {
	fn default() -> Self {
		// parse so we that we use serde defaults
		ron::from_str("()").expect("BUG: default BakeOpts RON does not parse")
	}
}
