use super::buffer_uniforms::*;
use super::internal::*;
use crate::graphics::wgpu_util as gpu;

pub(crate) struct ShaderPack {
	device: Arc<Device>,
	pub text_pipeline: TextPipeline,
	pub flat_texture_pipeline: FlatTexturePipeline,
	pub lightmap_pipeline: LightmapPipeline,
	pub normalmap_pipeline: NormalmapPipeline,
	pub sph_pipeline: SphPipeline,
	pub sph_shadows_pipeline: SphShadowPipeline,
	pub entity_pipeline: EntityPipeline,
	pub particles_pipeline: ParticlesPipeline,
	pub debris_pipeline: ParticlesPipeline,
	pub animation_pipeline: AnimationPipeline,
	pub font_texture: Arc<Texture>,
}

impl ShaderPack {
	pub fn new(device: Arc<Device>, queue: &Queue, opts: &GraphicsOpts, surface_format: TextureFormat) -> Self {
		let font_texture = Arc::new(_embedded_font_texture((&device, queue)));
		let global_uniforms_layout = GlobalsBuffer::bind_group_layout(&device);

		let text_pipeline = TextPipeline::new(opts, &device, surface_format);
		let flat_texture_pipeline = FlatTexturePipeline::new(opts, &device, surface_format, &global_uniforms_layout, false /*lines*/);
		let lightmap_pipeline = LightmapPipeline::new(opts, &device, surface_format, &global_uniforms_layout);
		let normalmap_pipeline = NormalmapPipeline::new(opts, &device, surface_format, &global_uniforms_layout);
		let sph_pipeline = SphPipeline::new(opts, &device, surface_format, &global_uniforms_layout);
		let sph_shadows_pipeline = SphShadowPipeline::new(opts, &device, surface_format, &global_uniforms_layout);
		let entity_pipeline = EntityPipeline::new(opts, &device, surface_format, &global_uniforms_layout);
		let particles_pipeline = ParticlesPipeline::new(opts, &device, surface_format, &global_uniforms_layout, true /* additive blend */);
		let debris_pipeline = ParticlesPipeline::new(opts, &device, surface_format, &global_uniforms_layout, false /* additive blend */);
		let animation_pipeline = AnimationPipeline::new(opts, &device, surface_format, &global_uniforms_layout);

		Self {
			device,
			text_pipeline,
			lightmap_pipeline,
			normalmap_pipeline,
			sph_pipeline,
			sph_shadows_pipeline,
			flat_texture_pipeline,
			entity_pipeline,
			animation_pipeline,
			particles_pipeline,
			debris_pipeline,
			font_texture,
		}
	}

	pub fn text(&self) -> Shader {
		Shader::Text(Arc::new(self.text_pipeline.texture_bind_group(&self.device, &self.font_texture)))
	}

	pub fn flat(&self, texture: &Texture) -> Shader {
		Shader::Flat(Arc::new(self.flat_texture_pipeline.texture_bind_group(&self.device, texture)))
	}

	pub fn lines(&self, texture: &Texture) -> Shader {
		Shader::Lines(Arc::new(self.flat_texture_pipeline.texture_bind_group(&self.device, texture)))
	}

	pub fn lightmap(&self, base_color: &Texture, lightmap: &Texture) -> Shader {
		Shader::Lightmap(Arc::new(self.lightmap_pipeline.texture_bind_group(&self.device, base_color, lightmap)))
	}

	pub fn normalmap(&self, base_color: &Texture, lightmap: &Texture, normalmap: &Texture, sun_mask: &Texture) -> Shader {
		Shader::Normalmap(Arc::new(self.normalmap_pipeline.texture_bind_group(&self.device, base_color, lightmap, normalmap, sun_mask)))
	}

	pub fn sph(&self, base_color: &Texture, sph: &[&Texture; 3], normalmap: &Texture, sun_mask: &Texture) -> Shader {
		Shader::Sph(Arc::new(self.sph_pipeline.texture_bind_group(&self.device, base_color, sph, normalmap, sun_mask)))
	}

	pub fn sph_shadows(&self, base_color: &Texture, sph: &[&Texture; 3], normalmap: &Texture, sun_mask: &Texture) -> Shader {
		Shader::SphShadows(Arc::new(self.sph_shadows_pipeline.texture_bind_group(&self.device, base_color, sph, normalmap, sun_mask)))
	}

	pub fn entity(&self, texture: &Texture, transform: mat4, bounding_box: &BoundingBox32, lightbox: &LightBox) -> Shader {
		Shader::Entity(
			Arc::new(self.entity_pipeline.texture_bind_group(&self.device, texture)),
			InstanceData {
				bounding_box_size: bounding_box.size(),
				_padding2: default(),
				bounding_box_bottom: bounding_box.center_bottom(),
				_padding3: default(),
				lightbox: *lightbox,
				model_matrix: transform.into(),
				time: default(),
				unused: default(),
				_padding: default(),
			},
		)
	}

	pub fn particles(&self, texture: &Texture, transform: mat4, phase: f32) -> Shader {
		Shader::Particles(
			Arc::new(self.particles_pipeline.texture_bind_group(&self.device, texture)),
			InstanceData {
				bounding_box_size: default(), // unused
				_padding2: default(),
				bounding_box_bottom: default(), // unused
				_padding3: default(),
				lightbox: default(), // <<
				model_matrix: transform.into(),
				time: phase,
				unused: default(),
				_padding: default(),
			},
		)
	}

	pub fn debris(&self, texture: &Texture, transform: mat4, phase: f32) -> Shader {
		Shader::Debris(
			Arc::new(self.particles_pipeline.texture_bind_group(&self.device, texture)),
			InstanceData {
				bounding_box_size: default(), // unused
				_padding2: default(),
				bounding_box_bottom: default(), // unused
				_padding3: default(),
				lightbox: default(), // <<
				model_matrix: transform.into(),
				time: phase,
				unused: default(),
				_padding: default(),
			},
		)
	}

	pub fn animation(&self, texture: &Texture, transform: mat4, t: f32, bounding_box: &BoundingBox32, lightbox: &LightBox) -> Shader {
		assert!(t >= 0.0); // TODO: debug_assert or warn
		assert!(t <= 1.0);
		Shader::Animation(
			Arc::new(self.animation_pipeline.texture_bind_group(&self.device, texture)),
			InstanceData {
				bounding_box_size: bounding_box.size(),
				_padding2: default(),
				bounding_box_bottom: bounding_box.center_bottom(),
				_padding3: default(),
				lightbox: *lightbox,
				model_matrix: transform.into(),
				time: t,
				unused: default(),
				_padding: default(),
			},
		)
	}
}

fn _embedded_font_texture(ctx: (&Device, &Queue)) -> Texture {
	let image = &image::load_from_memory(include_bytes!("font.png")).expect("decode embedded font");
	let opts = GraphicsOpts::default();
	gpu::upload_rgba_mipmap(ctx, &opts, &[&image.to_rgba8()], image.dimensions().into(), &NEAREST)
}
