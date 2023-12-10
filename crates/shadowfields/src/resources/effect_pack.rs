use super::internal::*;

pub(crate) struct EffectPack {
	pub particle_beam: Arc<VAO>,
	pub particle_explosion: Arc<VAO>,
	pub debris: Arc<VAO>,
}

/// Number of particles per unit of particle beam length.
pub const PARTICLE_BEAM_DENSITY: u32 = 8;
pub const PARTICLE_EXPLOSION_N: u32 = 1000;
pub const DEBRIS_N: u32 = 100;
pub const PARTICLE_SCALE: f32 = 0.2;

impl EffectPack {
	pub fn new() -> Result<Self> {
		let ctx = ctx();

		Ok(Self {
			particle_beam: Arc::new(ctx.upload_meshbuffer(&Self::particle_beam_vao())),
			particle_explosion: Arc::new(ctx.upload_meshbuffer(&Self::particle_explosion_vao())),
			debris: Arc::new(ctx.upload_meshbuffer(&Self::debris_vao())),
		})
	}

	// A VertexArray containing a "particle explosion" consisting of `n` triangles
	// with random orientations, and random velocities pointing away from the origin.
	// To be rendered with `shaders::Particles`.
	fn particle_explosion_vao() -> MeshBuffer {
		let pos = |_i| vec3(0.0, 0.0, 0.0);
		let vel = |_i| (3.0 + 1.0 * rand::thread_rng().gen::<f32>());
		Self::triangle_particles_vao(PARTICLE_EXPLOSION_N, pos, vel, || 999.9 /*per-triangle TTL */)
	}

	fn particle_beam_vao() -> MeshBuffer {
		let max_dist = 500;
		let n = PARTICLE_BEAM_DENSITY * max_dist;
		let pos = |i| {
			let mut rng = rand::thread_rng();
			let dist = (i as f32) / (PARTICLE_BEAM_DENSITY as f32);
			let rand = vec2(rng.gen(), rng.gen());
			const JITTER: f32 = 0.14;
			let rand = JITTER * uniform_disk(rand);
			vec3(rand.x(), dist, rand.y())
		};
		let vel = |_| rand::thread_rng().gen_range(0.1..1.5);
		Self::triangle_particles_vao(n, pos, vel, || rand::thread_rng().gen_range(PARTICLE_BEAM_TTL / 2.0..PARTICLE_BEAM_TTL))
	}

	fn debris_vao() -> MeshBuffer {
		let mut rng = rand::thread_rng();
		let n = DEBRIS_N as usize;

		let mut triangle = MeshBuffer::triangle(&[VertexLM::default(); 3]);
		let mut buf = MeshBuffer::new();

		let tex_coords = [vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(0.5, 0.0)];
		for tri_idx in 0..n {
			let tri_norm = sample_isotropic_direction(&mut rng);
			let basis = make_basis(tri_norm);
			let v_dir = sample_isotropic_direction(&mut rng);
			let vel = 1.0 * rng.gen::<f32>() * v_dir + 2.5 * rng.gen::<f32>() * vec3::EY;
			let tex_offset = vec2(rng.gen(), rng.gen());
			const SIZE: f32 = 1.5 * PARTICLE_SCALE;

			for (vert_idx, &vert) in TRIANGLE_VERTICES.iter().enumerate() {
				triangle.vertices[vert_idx].texcoords = 0.05 * tex_coords[vert_idx] + tex_offset;
				triangle.vertices[vert_idx].position = basis * vert * SIZE * rng.gen::<f32>();
				triangle.vertices[vert_idx].normal = vel; // !! hack: reusing normal as velocity :(
				triangle.vertices[vert_idx].lightcoords[1] = 1024.0; // !! hack: reusing lightcoords as TTL for culling. TTL handled by reducing indices over time
			}

			buf.append(&triangle)
		}

		buf
	}

	fn triangle_particles_vao(n: u32, pos: impl Fn(usize) -> vec3, vel: impl Fn(usize) -> f32, mut ttl: impl FnMut() -> f32) -> MeshBuffer {
		let mut rng = rand::thread_rng();
		let n = n as usize;

		let mut triangle = MeshBuffer::triangle(&[VertexLM::default(); 3]);
		let mut buf = MeshBuffer::new();

		let tex_coords = [vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(0.5, 0.0)];

		for tri_idx in 0..n {
			let norm = sample_isotropic_direction(&mut rng);
			let basis = make_basis(norm);
			let v_dir = sample_isotropic_direction(&mut rng); // - vel_y * vec3::EY; // minus moves particles away from player
			let vel = vel(tri_idx) * v_dir;

			let ttl = ttl(); // 👈 TTL is per-triangle

			for (vert_idx, &vert) in TRIANGLE_VERTICES.iter().enumerate() {
				triangle.vertices[vert_idx].texcoords = tex_coords[vert_idx];
				triangle.vertices[vert_idx].position = basis * vert * PARTICLE_SCALE + pos(tri_idx);
				triangle.vertices[vert_idx].normal = vel; // !! hack: reusing normal as velocity :(
				triangle.vertices[vert_idx].lightcoords[1] = ttl; // !! hack: reusing lightcoords as TTL for culling
			}

			buf.append(&triangle)
		}

		buf
	}
}

// Vertices of an equilateral triangle centered at (0,0,0).
// "Prototype" for all particle triangles.
const TRIANGLE_VERTICES: [vec3; 3] = [
	vec3(-0.5, -SIN_60 / 2.0, 0.0), //.
	vec3(0.5, -SIN_60 / 2.0, 0.0),
	vec3(0.0, SIN_60 / 2.0, 0.0),
];

const SIN_60: f32 = 0.86602540378;

/// Sample a random unit vector with isotropically distributed direction.
fn sample_isotropic_direction(rng: &mut impl rand::Rng) -> vec3 {
	let norm = rand_distr::StandardNormal;
	vec3(rng.sample(norm), rng.sample(norm), rng.sample(norm)).normalized()
}
