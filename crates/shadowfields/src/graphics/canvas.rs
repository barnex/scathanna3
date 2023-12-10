use super::buffer_uniforms::*;
use super::internal::*;
use crate::graphics::wgpu_util as gpu;
use crate::graphics::wgpu_util::COUNTERS;
use winit::window::Window;

/// A `Canvas` can be drawn on.
/// It hides a WGPU Surface, depth texture and context
/// and provides higher-level drawing functionality.
pub(crate) struct Canvas {
	// Render target
	config: wgpu::SurfaceConfiguration,
	surface: wgpu::Surface,
	depth_texture: Texture,
	msaa_fb: Option<MSAAFB>, // multi-sampled framebuffer, populated if MSAA is enabled

	// See globals.wgsl
	global_uniforms: GlobalsBuffer,
	instance_storage: StorageBuffer<InstanceData>,
}

/// Size of instance buffer.
pub(crate) const MAX_ENTITIES: usize = 1024;

impl Canvas {
	pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

	/// A `Canvas` that will present to `window`.
	/// NOTE: `resize` must be called whenever the window is resized.
	pub fn new(opts: &GraphicsOpts, window: &Window) -> Result<Self> {
		pollster::block_on(Self::new_async(opts, window))
	}

	async fn new_async(opts: &GraphicsOpts, window: &Window) -> Result<Self> {
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
			gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
			flags: wgpu::InstanceFlags::default(),
		});

		let surface = unsafe { instance.create_surface(window)? };
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.ok_or(anyhow!("No graphics adapter found"))?;
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: Some(file!()),
					features: default(),
					limits: wgpu::Limits::default(),
				},
				None, // Trace path
			)
			.await?;

		LOG.write(format!("Graphics adapter: {:?}", &adapter.get_info().name));

		let size = window.inner_size();

		let caps = surface.get_capabilities(&adapter);
		let format = caps.formats.get(0 /*the preferred format*/).copied().ok_or(anyhow!("No graphics adapter found"))?;
		let present_modes = caps.present_modes; // TODO
		println!("supported present modes {present_modes:?}");
		//let alpha_modes = caps.alpha_modes; // TODO

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format,
			view_formats: vec![format],
			width: size.width,
			height: size.height,
			present_mode: match opts.vsync {
				true => wgpu::PresentMode::Fifo,
				false => wgpu::PresentMode::Immediate,
			},

			alpha_mode: wgpu::CompositeAlphaMode::Auto,
		};
		surface.configure(&device, &config);

		let depth_texture = gpu::create_depth_texture(&device, opts, uvec2(config.width, config.height));
		let msaa_fb = gpu::maybe_create_msaa_fb(&device, opts, &config);

		let global_uniforms = GlobalsBuffer::new(&device); // TODO: we should be able to just make this a StorageBuffer?
		let instance_storage = StorageBuffer::with_capacity(&device, MAX_ENTITIES);

		GraphicsCtx::init(opts, device, queue, config.format);

		Ok(Self {
			surface,
			config,
			depth_texture,
			global_uniforms,
			instance_storage,
			msaa_fb,
		})
	}

	/// Must be called whenever the corresponding Window got resized,
	/// so that the Canvas size fits.
	pub fn resize(&mut self, new_size: uvec2) {
		let ctx = ctx();
		// It is rare but possible that a ridiculous size (-1) is intermittently requested.
		// This causes WGPU to checkfail:
		//    In Device::create_texture
		//    Dimension X value 4294967295 exceeds the limit of 8192
		// Normally a sensible size is requested soon after so ignoring + logging is okay.
		if new_size.iter().all(|v| v > 0 && v <= MAX_TEXTURE_SIZE) {
			self.config.width = new_size.x();
			self.config.height = new_size.y();
			self.surface.configure(ctx.device(), &self.config);
			self.depth_texture = ctx.create_depth_texture(&ctx.opts, uvec2(self.config.width, self.config.height));
			self.msaa_fb = ctx.create_msaa_fb(&ctx.opts, &self.config);
		} else {
			log::error!("canvas: resize: invalid size: {}", new_size);
		}
	}

	/// The Canvas' current size (width, height) in pixels.
	pub fn viewport_size(&self) -> uvec2 {
		uvec2(self.config.width, self.config.height)
	}
}

// --------------------------------------------------------------------------------  rendering

impl Canvas {
	pub fn render(&mut self, scene: &SceneGraph) {
		let surface_tex = match self.surface.get_current_texture() {
			Ok(v) => v,
			Err(wgpu::SurfaceError::Lost) => return self.handle_surface_lost(),
			Err(wgpu::SurfaceError::OutOfMemory) => panic!("out of memory"),
			Err(wgpu::SurfaceError::Outdated) => return, // Should be resolved by the next frame
			Err(wgpu::SurfaceError::Timeout) => return,  // Should be resolved by the next frame
		};
		self.render_to_surface(scene, &surface_tex);
		surface_tex.present();
	}

	fn render_to_surface(&mut self, sg: &SceneGraph, surface_tex: &wgpu::SurfaceTexture) {
		// upload uniforms: camera, sky color, sun direction, etc.
		let ctx = ctx();
		ctx.upload_buffer(&self.global_uniforms.buffer, &[GlobalsHostData::from(&sg.camera, self.viewport_size(), sg.sun_dir, sg.sun_color)]);

		// upload instance data (transforms, light, ...) for all instanced shaders.
		self.upload_instance_data(sg);

		let shaders = &ctx.shader_pack;

		let surface_view = surface_tex.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.command_encoder();
		{
			let mut render_pass = self.begin_render_pass(&mut encoder, &surface_view, sg.bg_color);

			// TODO: re-order for minimal shader / texture switching
			let mut entity_instance_counter = 0;

			for obj in &sg.objects {
				let mut entity_instance_id: u32 = 0; // here be mut dragons. Set to counter++ by Entity shader to mirror upload_entity_buffer
				let mut advance_instance_id = || {
					entity_instance_id = entity_instance_counter;
					entity_instance_counter += 1;
				};
				use Shader::*;
				match &obj.shader {
					Text(texture_bindings) => {
						render_pass.set_pipeline(&shaders.text_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
					}
					Flat(texture_bindings) => {
						render_pass.set_pipeline(&shaders.flat_texture_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
					}
					Lines(texture_bindings) => {
						render_pass.set_pipeline(&shaders.flat_lines_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
					}
					Highlight(texture_bindings) => {
						render_pass.set_pipeline(&shaders.highlight_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
					}
					Lightmap(texture_bindings) => {
						render_pass.set_pipeline(&shaders.lightmap_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
					}
					Normalmap(texture_bindings) => {
						render_pass.set_pipeline(&shaders.normalmap_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
					}
					Sph(texture_bindings) => {
						render_pass.set_pipeline(&shaders.sph_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
					}
					Entity(texture_bindings, _) => {
						render_pass.set_pipeline(&shaders.entity_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
						// ! keep the pace with the transforms copied by upload_instance_buffer().
						advance_instance_id()
					}
					Particles(texture_bindings, _) => {
						render_pass.set_pipeline(&shaders.particles_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
						// ! keep the pace with the transforms copied by upload_instance_buffer().
						advance_instance_id();
					}
					Debris(texture_bindings, _) => {
						render_pass.set_pipeline(&shaders.debris_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
						// ! keep the pace with the transforms copied by upload_instance_buffer().
						advance_instance_id();
					}
					Animation(texture_bindings, _) => {
						render_pass.set_pipeline(&shaders.animation_pipeline.pipeline);
						render_pass.set_bind_group(0, texture_bindings, &[]);
						// ! keep the pace with the transforms copied by upload_instance_buffer().
						advance_instance_id();
					}
				}

				// TODO: handle gracefully (e.g. do not draw: `continue`). clamping will still draw, just use previous entity transform
				if entity_instance_id >= MAX_ENTITIES as u32 {
					#[cfg(debug_assertions)]
					panic!("too many entities")
				}
				//let entity_instance_id = entity_instance_id.clamp(0, MAX_ENTITIES as u32 - 1);

				// All our shaders have the same bind groups 1 (global uniforms) and 2 (instance storage).
				render_pass.set_bind_group(1, &self.global_uniforms.bind_group, &[]);
				render_pass.set_bind_group(2, &self.instance_storage.bindings, &[]);

				let index_range = obj.index_range.clone().unwrap_or(0..obj.vao.num_indices);
				let num_indices = index_range.end - index_range.start;
				let vao_slice = obj.vao.vertex_buffer.slice(..);

				render_pass.set_vertex_buffer(0, vao_slice); // used by all shaders
				render_pass.set_index_buffer(obj.vao.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

				let instances = entity_instance_id..(entity_instance_id + 1);
				let num_instances = instances.clone().count() as u64;

				COUNTERS.draw_calls.inc();
				COUNTERS.draw_instances.add(num_instances);
				COUNTERS.vertices.add(num_instances * (num_indices as u64));

				render_pass.draw_indexed(index_range, 0, instances);
			}
			drop(render_pass);
		}

		ctx.queue().submit(std::iter::once(encoder.finish()));
	}

	// Copy entity transforms into storage buffer (`globals.wgsl: var instance_data`)
	fn upload_instance_data(&mut self, scene: &SceneGraph) {
		let ctx = ctx();
		self.instance_storage.host_data.clear();

		// push to vector until MAX_ENTITIES reached
		let try_push = |instance_light_data: &mut Vec<InstanceData>, light: InstanceData| {
			if instance_light_data.len() < MAX_ENTITIES {
				instance_light_data.push(light);
			} else {
				#[cfg(debug_assertions)]
				log::error!("max entities reached");
			}
		};

		use Shader::*;
		for obj in &scene.objects {
			match &obj.shader {
				Entity(_, data) | Particles(_, data) | Debris(_, data) | Animation(_, data) => try_push(&mut self.instance_storage.host_data, data.clone()),
				Flat(_) => (),
				Lines(_) => (),
				Lightmap(_) => (),
				Sph(_) => (),
				Normalmap(_) => (),
				Text(_) => (),
				Highlight(_) => (),
			}
		}

		// upload storage buffer to GPU (only the used part, we usually have far fewer than `MAX_ENTITIES` entities).
		if self.instance_storage.host_data.len() != 0 {
			debug_assert!(self.instance_storage.host_data.len() <= MAX_ENTITIES);
			self.instance_storage.upload(ctx);
		}
	}

	fn command_encoder(&self) -> wgpu::CommandEncoder {
		ctx().device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(file!()) })
	}

	fn begin_render_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, color_view: &'a wgpu::TextureView, clear_color: vec3) -> wgpu::RenderPass<'a> {
		// ! switch based on MSAA
		let (view, resolve_target) = match &self.msaa_fb {
			None => (color_view, None),
			Some(MSAAFB { fb_view, .. }) => (fb_view, Some(color_view)),
		};

		encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some(file!()),
			color_attachments: &[
				//
				Some(wgpu::RenderPassColorAttachment {
					view,           // ! depends on MSAA
					resolve_target, //  ! depends on MSAA
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: clear_color.x().into(),
							g: clear_color.y().into(),
							b: clear_color.z().into(),
							a: 1.0,
						}),
						store: wgpu::StoreOp::Store,
					},
				}),
			],
			depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
				view: &self.depth_texture.view,
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Clear(1.0),
					store: wgpu::StoreOp::Store,
				}),
				stencil_ops: None,
			}),
			timestamp_writes: None,
			occlusion_query_set: None,
		})
	}

	// To be called if a winit redraw returns SurfaceError::Lost.
	fn handle_surface_lost(&mut self) {
		info!("handle_surface_lost");
		self.resize((self.config.width, self.config.height).into())
	}
}

const MAX_TEXTURE_SIZE: u32 = 8192;
