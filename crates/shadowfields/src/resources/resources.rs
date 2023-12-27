use super::internal::*;

/// Asset loader and cache.
pub(crate) struct Resources {
	pub vaos: Cache<VAO>,
	pub animations: Cache<AnimVAOs>,
	pub textures: Cache<Texture>,

	pub effects: EffectPack,

	pub default_texture: Arc<Texture>,
}

impl Resources {
	pub fn new() -> Result<Self> {
		let effect_pack = EffectPack::new()?;
		let animations = Cache::<AnimVAOs>::default();
		let default_texture = Arc::new(uniform_texture(vec4(0.5, 0.5, 0.5, 1.0)));

		Ok(Self {
			effects: effect_pack,
			vaos: default(),
			animations,
			textures: default(),
			default_texture,
		})
	}

	pub fn vao(&self, handle: Handle) -> Option<Arc<VAO>> {
		self.vaos.load_sync(handle)
	}
}

impl Load for Texture {
	/// Load texture by name or use #rrggbb for uniform color
	fn load(name: &str) -> Result<Self> {
		let ctx = ctx();
		if name.starts_with('#') {
			Ok(uniform_texture(parse_hex_color(name)?.append(1.0)))
		} else {
			match ctx.opts.textures {
				true => Ok(ctx.upload_image_mip(&load_image(name)?, &TextureOpts::DEFAULT)),
				false => Ok(uniform_texture(vec4::ONES)),
			}
		}
	}
}

pub fn parse_hex_color(color: &str) -> Result<vec3> {
	let color = color.strip_prefix('#').unwrap_or(color);
	if color.len() != 6 {
		bail!("parse hex colour `{color}`: need 6 characters")
	}
	let r = u8::from_str_radix(&color[0..2], 16)?;
	let g = u8::from_str_radix(&color[2..4], 16)?;
	let b = u8::from_str_radix(&color[4..6], 16)?;
	Ok(Vector3::new(r, g, b).map(srgb_to_linear))
}

impl Load for Prop {
	fn load(name: &str) -> Result<Self> {
		Self::from_str(name)
	}
}

impl Load for VAO {
	fn load(name: &str) -> Result<Self> {
		let ctx = ctx();
		let meshbuf = load_wavefront_merged(name)?;
		//let meshbuf = rescale_mesh(meshbuf);
		Ok(ctx.upload_meshbuffer(&meshbuf))
	}
}

impl Load for AnimVAOs {
	fn load(name: &str) -> Result<Self> {
		load_anim_vaos(name)
	}
}

fn rescale_mesh(mesh: MeshBuffer) -> MeshBuffer {
	let bounds = mesh.bounds().unwrap_or(BoundingBox::new(default(), default()));
	let scale = 1.0 / bounds.size().y();
	mesh.map_positions(|p| p * scale)
}
