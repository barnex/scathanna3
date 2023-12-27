//! Find and load asset files (textures, meshes).
use super::internal::*;
use std::sync::OnceLock;

static ASSETS_DIR: OnceLock<AssetsDir> = OnceLock::new();

/// Find the "assets" directory.
/// Must be called before `assets()`.
pub fn init_assets_dir(assets_base_name: &str) -> Result<()> {
	match ASSETS_DIR.set(AssetsDir::find(assets_base_name)?) {
		Ok(()) => Ok(()),
		Err(_prev) => bail!("BUG: init_assets_dir called more than once"),
	}
}

/// Absolute path to the "assets" directory.
/// `init_assets_dir` must be called prior.
pub fn assets_dir() -> &'static AssetsDir {
	ASSETS_DIR.get().expect("BUG: init_assets_dir has not been called")
}

// Path to the `assets/` directory.
// Typestate pattern ensures correct use.
#[derive(Clone)]
pub struct AssetsDir(PathBuf);

// Path to a map directory (e.g. `deck.hx`).
// Typestate pattern ensures correct use.
#[derive(Debug)]
pub struct MapDir(PathBuf);

impl AssetsDir {
	/// Find the absolute path of the `assets_base_name` (default: "assets") directory.
	/// Search in the current working directory and the executable's directory.
	fn find(assets_base_name: &str) -> Result<Self> {
		if let Ok(dir) = std::env::current_dir() {
			log::info!("searching for assets in working directory: {}", dir.to_string_lossy());
			let abs = dir.join(assets_base_name);
			if abs.exists() {
				return Ok(Self(abs));
			}
		}

		let exe = std::env::current_exe()?;
		if let Some(dir) = exe.parent() {
			log::info!("searching for assets in executable directory: {}", dir.to_string_lossy());
			let abs = dir.join(assets_base_name);
			if abs.exists() {
				return Ok(Self(abs));
			}
		}

		Err(anyhow!("assets directory not found.\nBe sure to run this program form a directory that contains 'assets/'."))
	}

	// fuzzy find map: accept name (e.g. "deck") or full path (e.g. "assets/maps/deck.map")
	pub fn find_map_dir(&self, map_name: &str) -> MapDir {
		// map name is full path (e.g. "assets/maps/deck.map")
		{
			let map_dir = Path::new(map_name);
			if map_dir.exists() {
				return MapDir(map_dir.to_owned());
			}
		}

		// no match, return canonical map dir, which will eventually trigger a "file not found".
		self.map_dir(map_name)
	}

	fn map_dir(&self, map_name: &str) -> MapDir {
		MapDir(self.maps_dir().join(map_name.to_string() + ".map"))
	}

	pub fn find_all_maps(&self) -> Result<Vec<String>> {
		Ok(self.iter_all_maps()?.sorted().collect())
	}

	fn iter_all_maps(&self) -> Result<impl Iterator<Item = String>> {
		Ok(read_dir_names(&self.maps_dir())? //
			.filter_map(|f| f.file_name().map(|n| n.to_owned()))
			.map(|n| n.to_string_lossy().to_string())
			.filter_map(|n| n.strip_suffix(".map").map(|n| n.to_owned())))
	}

	fn maps_dir(&self) -> PathBuf {
		self.dir().join("maps")
	}

	// find an `obj` or `obj.gz` file in the assets directory.
	pub fn find_mesh(&self, base: &str) -> Result<PathBuf> {
		Self::find_asset(&self.0.join("meshes"), base, &["obj", "obj.gz"])
	}

	pub fn find_object(&self, base: &str) -> Result<PathBuf> {
		Self::find_asset(&self.0.join("objects"), base, &["ron"])
	}

	/// Find absolute path to a texture file with `base` name. E.g.:
	///   "lava" => "/path/to/textures/lava.png"
	fn find_texture(&self, base: &str) -> Result<PathBuf> {
		Self::find_asset(&self.textures_dir(), base, &["png", "jpg", "jpeg"])
	}

	/// Find the absolute path of an asset file. E.g.:
	///   find_asset("/path/to/assets/textures", "lava", &["png", "jpg"])? =>  /path/to/assets/textures/lava.jpg
	fn find_asset(dir: &Path, base: &str, extensions: &[&str]) -> Result<PathBuf> {
		for ext in extensions {
			let file = dir.join(base.to_owned() + "." + ext); // note: do not use .with_extension, *replaces* extension.
			if file.exists() {
				return Ok(file);
			}
		}
		Err(anyhow!("asset not found: {:?} with extension {}", dir.join(base), extensions.join(", ")))
	}

	pub fn audio_dir(&self) -> PathBuf {
		self.0.join("audio")
	}

	pub fn music_dir(&self) -> PathBuf {
		self.0.join("music")
	}

	pub fn materials_dir(&self, resolution: u32) -> PathBuf {
		self.0.join("materials").join(resolution.to_string())
	}

	pub fn settings_file(&self, file: &str) -> Result<PathBuf> {
		Ok(self.0.parent().ok_or(anyhow!("assets parent directory not found"))?.join(file))
	}

	// TODO: resolution
	fn textures_dir(&self) -> PathBuf {
		self.0.join("textures")
	}

	fn dir(&self) -> &Path {
		&self.0
	}
}

impl MapDir {
	pub fn gltf_file(&self) -> PathBuf {
		self.0.join("map.gltf")
	}

	pub fn gltf_blob(&self) -> PathBuf {
		self.0.join("map.bin")
	}

	pub fn scene_mesh_file(&self) -> PathBuf {
		self.0.join("scene_mesh.bincode.gz")
	}

	pub fn collider_mesh_file(&self) -> PathBuf {
		self.0.join("collider_mesh.bincode.gz")
	}

	pub fn baking_file(&self) -> PathBuf {
		self.0.join("baking.ron")
	}

	pub fn metadata_file(&self) -> PathBuf {
		self.0.join("metadata.ron")
	}

	pub fn lightmap_dir(&self) -> PathBuf {
		self.0.join("lm")
	}

	// For tests only
	pub fn lightmap_gold_dir(&self) -> PathBuf {
		self.0.join("lm_gold")
	}

	/// Lightmap file for object + channel. E.g.
	/// channel: sun, object: Cube.001 => `my_map.map/lm/Cube.001_sun.png`
	pub fn lm_file(&self, channel: &str, object: &str) -> PathBuf {
		// Note: don't use `Path::with_extension`, this messes up blender names (e.g. `Cube.001` -> `Cube.png`:( )
		self.lightmap_dir().join(object.to_string() + "_" + channel + ".png")
	}

	pub fn exists(&self) -> bool {
		self.0.exists()
	}

	pub fn mkdir(&self) -> Result<()> {
		mkdir(&self.0)
	}
}

/// Find and load a wavefront OBJ file by base name (no extension, e.g. "rocket").
/// Searches `{assets}/obj` for `{base}.obj`, `{base.obj.gz}`.
/// All Objects (in the wavefront sense, e.g. 'Cube.001', 'Cube.002')  are merged into one.
/// Not cached.
pub(crate) fn load_wavefront_merged(base: &str) -> Result<MeshBuffer> {
	convert_wavefront_all(&parse_wavefront(base)?)
}

/// Find and parse a wavefront file by name (no extension, e.g. "rocket").
fn parse_wavefront(base: &str) -> Result<wavefrontobj::ObjSet> {
	let path = assets_dir().find_mesh(base)?;
	log_loading(&path);
	match path.extension().unwrap_or_default().to_string_lossy().as_ref() {
		"obj" => wavefrontobj::parse(open(&path)?),
		"gz" => wavefrontobj::parse(GzDecoder::new(open(&path)?)),
		_ => Err(anyhow!("unsupported obj file format: {}", path.to_string_lossy())),
	}
}

pub(crate) fn upload_image(base: &str, sampling: &TextureOpts) -> Result<Texture> {
	Ok(ctx().upload_image_mip(&load_image(base)?, sampling))
}

/// Find and load an image file by base name (no extension, e.g. "lava").
/// Searches `{assets}/textures` for `{base}.png`, `{base.jpg}`, `{base.jpeg}`.
/// Not cached.
//todo resolution
pub(crate) fn load_image(base: &str) -> Result<DynamicImage> {
	let path = assets_dir().find_texture(base).log_err()?;
	log_loading(&path);
	Ok(image::open(&path)?)
}

/// Load image file from `assets/textures/{handle}`,
/// or return uniform color if the handle is a hex color code (e.g. `#FF0077`).
pub fn load_image_or_color(handle: Handle) -> Result<RgbImage> {
	let handle = handle.as_str();
	if handle.starts_with('#') {
		Ok(uniform_image(parse_hex_color(handle)?))
	} else {
		load_image(handle).map(|img| img.into_rgb8())
	}
}

fn uniform_image(color: vec3) -> RgbImage {
	RgbImage::from_pixel(2, 2, image::Rgb(color.map(linear_to_srgb).into()))
}

pub(crate) fn log_loading(path: &Path) {
	LOG.replace_if_prefix("loading", path.file_name().unwrap_or(path.as_os_str()).to_string_lossy());
}

// Convert a wavefront object to Vertices + Indices that can be uploaded to the GPU.
fn convert_wavefront_all(obj_set: &wavefrontobj::ObjSet) -> Result<MeshBuffer> {
	let shards = convert_wavefront_shards(obj_set)?;
	Ok(MeshBuffer::collect(&shards))
}

/// Convert an Object set into one Meshbuffer per Object.
/// Intended for loading Block models, generating one `Shard` for each surface that needs an independent lightmap.
/// E.g., the 6 faces of a cube would be independent Shards, as the lighting should not
/// be continuous between one face and the next. But the many faces of a icosphere would be a single Shard,
/// because we want the lighting to be continuous over the sphere.
fn convert_wavefront_shards(obj_set: &wavefrontobj::ObjSet) -> Result<Vec<MeshBuffer>> {
	//LOG.write("TODO: tangents for wavefront obj");
	let mut shards = vec![];

	for obj in &obj_set.objects {
		let mut buf = MeshBuffer::new();
		for face in &obj.faces {
			if face.len() != 3 {
				return Err(anyhow!("only triangular faces supported, got face with {} vertices", face.len()));
			}
			for v in face {
				let vertex = VertexLM {
					position: v.position,
					normal: v.normal,
					texcoords: flip_v(v.texture),
					lightcoords: flip_v(v.texture),
					tangent_u: vec3::ZERO,
					tangent_v: vec3::ZERO,
					/*TODO: tangents*/
				};
				buf.push(vertex)
			}
		}
		shards.push(buf)
	}
	Ok(shards)
}

pub(crate) fn fallback_image() -> RgbImage {
	RgbImage::from_pixel(2, 2, Rgb([200, 200, 200]))
}

// Flip the orientation of the V texture coordinate.
// Used to convert from Blender's "up" definition or our "up".
fn flip_v(t: vec2) -> vec2 {
	vec2(t.x(), 1.0 - t.y())
}
