use super::internal::*;

/// A shader
pub(crate) struct SphShadowPipeline {
	pub pipeline: wgpu::RenderPipeline,
	texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl SphShadowPipeline {
	pub fn new(opts: &GraphicsOpts, device: &wgpu::Device, surface_format: wgpu::TextureFormat, global_uniforms_layout: &wgpu::BindGroupLayout) -> Self {
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some(file!()),
			source: wgpu::ShaderSource::Wgsl(include_str!("sph_shadow.wgsl").into()),
		});

		let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				// diffuse texture
				wgpu::BindGroupLayoutEntry {
					binding: 0, // Fragment shader: t_base_color;
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1, // Fragment shader: s_base_color;
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				// lightmap texture
				wgpu::BindGroupLayoutEntry {
					binding: 2, // Fragment shader: t_sphz
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 3, // Fragment shader: s_sphz
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				// normal texture
				wgpu::BindGroupLayoutEntry {
					binding: 4, // Fragment shader: t_normalmap
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 5, // Fragment shader: s_lightmap
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				// direct light visibility
				wgpu::BindGroupLayoutEntry {
					binding: 6, // Fragment shader: t_direct
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 7, // Fragment shader: s_direct
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},

				wgpu::BindGroupLayoutEntry {
					binding: 8, // Fragment shader: t_sphx
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 9, // Fragment shader: s_sphx
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},


				wgpu::BindGroupLayoutEntry {
					binding: 10, // Fragment shader: t_sphy
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 11, // Fragment shader: s_sphy
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},

			],
			label: Some(file!()),
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some(file!()),
			layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some(file!()),
				bind_group_layouts: &[&texture_bind_group_layout, global_uniforms_layout],
				push_constant_ranges: &[],
			})),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[VertexLM::desc()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_format,
					blend: Some(wgpu::BlendState {
						color: wgpu::BlendComponent::REPLACE,
						alpha: wgpu::BlendComponent::REPLACE,
					}),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Cw,
				cull_mode: Some(wgpu::Face::Front),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: Some(wgpu::DepthStencilState {
				format: Canvas::DEPTH_FORMAT,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default(),
			}),
			multisample: wgpu::MultisampleState {
				count: opts.msaa_sample_count(),
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		Self { pipeline, texture_bind_group_layout }
	}

	fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
		&self.texture_bind_group_layout
	}

	pub fn texture_bind_group(&self, device: &Device, texture: &Texture, sph: &[&Texture; 3], normalmap: &Texture, sun_mask: &Texture) -> wgpu::BindGroup {
		use wgpu::BindGroupEntry;
		use wgpu::BindingResource;

		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: self.texture_bind_group_layout(),
			entries: &[
				BindGroupEntry {
					binding: 0, //
					resource: BindingResource::TextureView(&texture.view),
				},
				BindGroupEntry {
					binding: 1,
					resource: BindingResource::Sampler(&texture.sampler),
				},
				BindGroupEntry {
					binding: 2, //
					resource: BindingResource::TextureView(&sph[2].view),
				},
				BindGroupEntry {
					binding: 3,
					resource: BindingResource::Sampler(&sph[2].sampler),
				},
				BindGroupEntry {
					binding: 4, //
					resource: BindingResource::TextureView(&normalmap.view),
				},
				BindGroupEntry {
					binding: 5,
					resource: BindingResource::Sampler(&normalmap.sampler),
				},
				BindGroupEntry {
					binding: 6, //
					resource: BindingResource::TextureView(&sun_mask.view),
				},
				BindGroupEntry {
					binding: 7,
					resource: BindingResource::Sampler(&sun_mask.sampler),
				},
				BindGroupEntry {
					binding: 8, //
					resource: BindingResource::TextureView(&sph[0].view),
				},
				BindGroupEntry {
					binding: 9,
					resource: BindingResource::Sampler(&sph[0].sampler),
				},
				BindGroupEntry {
					binding: 10, //
					resource: BindingResource::TextureView(&sph[1].view),
				},
				BindGroupEntry {
					binding: 11,
					resource: BindingResource::Sampler(&sph[1].sampler),
				},
			],
			label: Some(file!()),
		})
	}
}
