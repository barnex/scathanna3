use super::internal::*;

#[derive(Default)]
pub struct VolumetricLight {
	field: RefCell<HashMap<ivec3, vec4>>,
}

impl VolumetricLight {
	/// xyz: ambient, w: sun
	pub fn at(&self, map: &Map, pos: vec3) -> vec4 {
		let mut c = [[[vec4::ZERO; 2]; 2]; 2];

		for (dz, dy, dx) in cross3(0..=1, 0..=1, 0..=1) {
			let key = key_for(pos) + ivec3(dx, dy, dz);
			c[dz as usize][dy as usize][dx as usize] = *self.field.borrow_mut().entry(key).or_insert_with(|| sample_light_at(map, key.to_f32() + vec3::repeat(0.5)));
		}

		let t = pos - key_for(pos).to_f32(); // <<< TODO: div by stride if not 1.0.
		debug_assert!(t.iter().all(|v| v >= 0.0 && v <= 1.0), "pos = {pos}, key = {}, t = {t}", key_for(pos).to_f32());

		trilinear_interp(&c, t)
	}

	pub fn lightbox_for(&self, map: &Map, bb: &BoundingBox32) -> LightBox {
		let mut lightbox = LightBox::default();
		for z in 0..=1 {
			for y in 0..=1 {
				for x in 0..=1 {
					let pos = bb.min + ivec3(x, y, z).to_f32() * bb.size();
					lightbox.vertex_light[z as usize][y as usize][x as usize] = self.at(map, pos);
				}
			}
		}
		lightbox
	}
}

/// xyz: ambient, w: sun factor
fn sample_light_at(map: &Map, pos: vec3) -> vec4 {
	let fake_ambient = vec4(0.2, 0.2, 0.10, 0.02);
	sample_sun_at(map, pos) + sample_sky_at(map, pos) + fake_ambient
}

fn sample_sun_at(map: &Map, pos: vec3) -> vec4 {
	let to_sun = -map.sun_dir;
	let ray = Ray::new(pos, to_sun);
	match map.intersect_t(&ray) {
		None => vec4(0.0, 0.0, 0.0, 1.0),
		Some(_) => vec4::ZERO,
	}
}

fn sample_sky_at(map: &Map, pos: vec3) -> vec4 {
	let mut acc = vec4::ZERO;

	let mut n = 0;
	for (dx, dz) in cross(-2..=2, -2..=2) {
		n += 1;
		let to_sky = vec3(dx as f32, 2.0, dz as f32).normalized();
		let ray = Ray::new(pos, to_sky);
		acc += match map.intersect_t(&ray) {
			None => map.sky_color.append(0.0),
			Some(_) => vec4::ZERO,
		}
	}

	0.5 * acc / (n as f32)
}

fn key_for(pos: vec3) -> ivec3 {
	pos.map(|v| v.floor() as i32)
}
