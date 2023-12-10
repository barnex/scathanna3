use super::internal::*;
use crate::prelude::*;

/// A game map (serializable data + acceleration structures for physics).
pub struct Map {
	pub name: String,

	pub sky_color: vec3,
	pub sun_color: vec3,
	pub sky_box: Option<Handle>,
	pub sun_dir: vec3,

	pub gravity: f32, // downward gravity (i.e. positive number) in m/s^2.

	pub jump_pads: Vec<JumpPad>,
	pub spawn_points: Vec<SpawnPoint>,
	pub pickup_points: Vec<PickupPoint>,

	collider_mesh: Node<ColliderFace>,

	pub material_palette: MaterialPalette,
	pub volumetric_light_cache: VolumetricLight, // <<< clear on map switch?
}

impl Map {
	/// Load a Map from directory `assets/maps/map_name.hx`.
	pub fn load(map_name: &str) -> Result<Self> {
		let map_dir = &assets_dir().find_map_dir(map_name);

		let face_tree = load_collider_mesh(map_dir)?;

		let metadata = Metadata::load(map_dir)?;
		let sky_box = metadata.sky_box.map(|s| Handle::from_str(&s)).transpose()?;

		Ok(Self {
			name: map_name.into(),

			sky_color: metadata.sky_color,
			sky_box,
			// ⚠️ Unlike metadata, Map always has sun. No sun represented by color BLACK.
			sun_color: metadata.sun_def.as_ref().map(|sun| sun.color).unwrap_or_default(),
			sun_dir: metadata.sun_def.as_ref().map(|sun| sun.dir.normalized()).unwrap_or(-vec3::EY),
			gravity: STD_GRAVITY,
			jump_pads: metadata.jump_pads,
			spawn_points: metadata.spawn_points,
			pickup_points: metadata.pickup_points,
			collider_mesh: face_tree,
			volumetric_light_cache: default(),
			material_palette: metadata.materials,
		})
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	/// Where does a ray intersect the map, if any.
	pub fn intersect_t_64(&self, ray: &Ray64) -> Option<f64> {
		let ray = Ray::new(ray.start.to_f32(), ray.dir.to_f32());
		self.collider_mesh.intersection(&ray).maybe_t().map(|t| t as f64)
	}

	/// Where does a ray intersect the map, if any
	pub fn intersect_t(&self, ray: &Ray32) -> Option<f32> {
		self.collider_mesh.intersection(ray).maybe_t()
	}

	pub fn intersects(&self, ray: &Ray32) -> bool {
		self.collider_mesh.intersection(ray).maybe_t().is_some()
	}

	pub fn intersect(&self, ray: &Ray32) -> HitRecord<f32, (Vector3<f32>, Vector2<f32>, Handle)> {
		self.collider_mesh.intersection(ray)
	}

	pub fn bumps(&self, bounds: &BoundingBox<f32>) -> bool {
		// Probe intersection with 4 diagonal line segments in the bounding box.
		// TODO: use more accurate & robust method, e.g. capsule.
		// +-----+
		// |\   /|
		// | \ / |
		// +--*--+
		let p = bounds.center_bottom();
		let (dx, dy, dz) = bounds.size().into();

		self.intersects_segment(p, p + vec3(dx, dy, dz))
			|| self.intersects_segment(p, p + vec3(dx, dy, -dz))
			|| self.intersects_segment(p, p + vec3(-dx, dy, dz))
			|| self.intersects_segment(p, p + vec3(-dx, dy, dz))
	}

	fn intersects_segment(&self, start: vec3, end: vec3) -> bool {
		let delta = end - start;
		let dir = delta.normalized();
		let len = delta.len();
		let ray = Ray::new(start, dir);
		self.collider_mesh.intersection(&ray).t <= len
	}
}

fn load_collider_mesh(map_dir: &MapDir) -> Result<Node<ColliderFace>> {
	load_bincode_gz(&map_dir.collider_mesh_file())
}

// does a wall obstruct the line of sight between two positions?
pub(crate) fn is_obstructed(map: &Map, pos1: vec3, pos2: vec3) -> bool {
	let dir = (pos2 - pos1).normalized();
	let len = (pos2 - pos1).len();
	let ray = Ray64::new(pos1.into(), dir.into());
	let t = map.intersect_t_64(&ray).unwrap_or(f64::INFINITY) as f32;
	t < len
}
